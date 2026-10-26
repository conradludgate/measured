//! The Prometheus text exposition format for [`measured`].
//!
//! ```
//! use measured::{Counter, MetricGroup};
//! use measured_prometheus_text::BufferedTextEncoder;
//!
//! #[derive(MetricGroup, Default)]
//! struct Metrics {
//!     /// total number of requests
//!     requests_total: Counter,
//! }
//!
//! let metrics = Metrics::default();
//! metrics.requests_total.inc();
//!
//! // ideally, the encoder should be re-used between collections.
//! let mut encoder = BufferedTextEncoder::new();
//! metrics.collect_group_into(&mut encoder).unwrap();
//! let bytes = encoder.finish();
//!
//! assert_eq!(bytes, "# HELP requests_total total number of requests\n# TYPE requests_total counter\nrequests_total 1\n");
//! ```

use std::io::{self, Write};

use bytes::{BufMut, Bytes, BytesMut};
use measured::label::{LabelGroup, LabelGroupVisitor, LabelName, LabelValue, LabelVisitor};
use measured::metric::exemplar::Exemplar;
use measured::metric::group::{Encoding, FamilyMeta, MetricKind, SampleExtras};
use measured::metric::name::{Bucket, Count, MetricNameEncoder, Sum};
use memchr::memchr3_iter;

/// Values that prometheus supports in the text format
#[derive(Clone, Copy, Debug)]
enum MetricValue {
    Int(i64),
    Uint(u64),
    Float(f64),
}

/// Which text format an encoder writes
#[derive(Clone, Copy, Debug, PartialEq)]
enum Format {
    /// The prometheus text format, `version=0.0.4`
    Classic,
    /// The OpenMetrics text format, `version=1.0.0`
    OpenMetrics,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Info,
    Metrics,
}

/// The shared implementation of [`TextEncoder`] and [`OpenMetricsEncoder`].
///
/// The format is a const generic so that the checks for it are removed at compile time.
struct Inner<W, const OPEN_METRICS: bool> {
    state: State,
    writer: W,
    /// The OpenMetrics name of the current metric family, which may be different from the metric name
    family: Vec<u8>,
    family_kind: MetricKind,
    /// Scratch space for rendering sample names
    scratch: Vec<u8>,
}

impl<W: Write, const OPEN_METRICS: bool> Inner<W, OPEN_METRICS> {
    const FORMAT: Format = if OPEN_METRICS {
        Format::OpenMetrics
    } else {
        Format::Classic
    };

    fn new(writer: W) -> Self {
        Self {
            state: State::Info,
            writer,
            family: Vec::new(),
            family_kind: MetricKind::Untyped,
            scratch: Vec::new(),
        }
    }

    fn finish(&mut self) -> io::Result<()> {
        self.state = State::Info;
        if Self::FORMAT == Format::OpenMetrics {
            self.writer.write_all(b"# EOF\n")?;
        }
        Ok(())
    }

    fn write_family(&mut self, name: impl MetricNameEncoder, kind: MetricKind, meta: FamilyMeta<'_>) -> io::Result<()> {
        if meta.unit().is_some_and(|unit| {
            unit.is_empty()
                || !unit
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b':')
        }) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid metric unit",
            ));
        }
        let help = meta.help();
        let unit = meta.unit();
        match Self::FORMAT {
            Format::Classic => {
                if self.state == State::Metrics {
                    self.writer.write_all(b"\n")?;
                }

                if let Some(help) = help {
                    self.writer.write_all(b"# HELP ")?;
                    name.encode_utf8(&mut self.writer)?;
                    self.writer.write_all(b" ")?;
                    write_help_str(help, &mut self.writer)?;
                    self.writer.write_all(b"\n")?;
                }

                // the prometheus text format only supports these types
                let kind = match kind {
                    MetricKind::Counter
                    | MetricKind::Gauge
                    | MetricKind::Histogram
                    | MetricKind::Summary
                    | MetricKind::Untyped => kind,
                    MetricKind::StateSet | MetricKind::Info => MetricKind::Gauge,
                    // see `write_gauge_histogram`
                    MetricKind::GaugeHistogram => MetricKind::Histogram,
                    _ => MetricKind::Untyped,
                };
                self.writer.write_all(b"# TYPE ")?;
                name.encode_utf8(&mut self.writer)?;
                self.writer.write_all(b" ")?;
                self.writer.write_all(kind.as_str().as_bytes())?;
                self.writer.write_all(b"\n")?;
            }
            Format::OpenMetrics => {
                // OpenMetrics family names do not include the `_total` and `_info` sample suffixes
                self.family.clear();
                name.encode_utf8(&mut self.family)?;
                let suffix: &[u8] = match kind {
                    MetricKind::Counter => b"_total",
                    MetricKind::Info => b"_info",
                    _ => b"",
                };
                if !suffix.is_empty() && self.family.ends_with(suffix) {
                    self.family.truncate(self.family.len() - suffix.len());
                }
                self.family_kind = kind;

                let kind = match kind {
                    MetricKind::Counter
                    | MetricKind::Gauge
                    | MetricKind::Histogram
                    | MetricKind::GaugeHistogram
                    | MetricKind::StateSet
                    | MetricKind::Info
                    | MetricKind::Summary => kind.as_str(),
                    _ => "unknown",
                };
                self.writer.write_all(b"# TYPE ")?;
                self.writer.write_all(&self.family)?;
                self.writer.write_all(b" ")?;
                self.writer.write_all(kind.as_bytes())?;
                self.writer.write_all(b"\n")?;

                if let Some(unit) =
                    unit.filter(|_| !matches!(self.family_kind, MetricKind::Info | MetricKind::StateSet))
                {
                    self.writer.write_all(b"# UNIT ")?;
                    self.writer.write_all(&self.family)?;
                    self.writer.write_all(b" ")?;
                    self.writer.write_all(unit.as_bytes())?;
                    self.writer.write_all(b"\n")?;
                }

                if let Some(help) = help {
                    self.writer.write_all(b"# HELP ")?;
                    self.writer.write_all(&self.family)?;
                    self.writer.write_all(b" ")?;
                    write_label_str_value(help, &mut self.writer)?;
                    self.writer.write_all(b"\n")?;
                }
            }
        }
        self.state = State::Info;
        Ok(())
    }

    /// Write the name of a sample. OpenMetrics requires some samples to have a suffix.
    fn write_sample_name(&mut self, name: impl MetricNameEncoder) -> io::Result<()> {
        let suffix: &[u8] = match (Self::FORMAT, self.family_kind) {
            (Format::OpenMetrics, MetricKind::Counter) => b"_total",
            (Format::OpenMetrics, MetricKind::Info) => b"_info",
            _ => return name.encode_utf8(&mut self.writer),
        };

        self.scratch.clear();
        name.encode_utf8(&mut self.scratch)?;
        if self.scratch == self.family {
            self.scratch.extend_from_slice(suffix);
        }
        self.writer.write_all(&self.scratch)
    }

    fn write_labels(&mut self, labels: impl LabelGroup, braces: bool) -> io::Result<()> {
        struct Visitor<'a, W> {
            writer: &'a mut W,
        }
        impl<W: Write> LabelVisitor for Visitor<'_, W> {
            type Output = Result<(), std::io::Error>;
            fn write_int(self, x: i64) -> Result<(), std::io::Error> {
                self.write_str(x.format_into(&mut core::fmt::NumBuffer::new()))
            }

            fn write_float(self, x: f64) -> Result<(), std::io::Error> {
                self.write_str(format_float(&mut zmij::Buffer::new(), x))
            }

            fn write_str(self, x: &str) -> Result<(), std::io::Error> {
                self.writer.write_all(b"=\"")?;
                write_label_str_value(x, &mut *self.writer)?;
                self.writer.write_all(b"\"")?;
                Ok(())
            }
        }

        struct GroupVisitor<'a, W> {
            first: bool,
            writer: &'a mut W,
        }
        impl<W: Write> LabelGroupVisitor for GroupVisitor<'_, W> {
            type Output = Result<(), std::io::Error>;
            fn write_value(&mut self, name: &LabelName, x: &impl LabelValue) -> Result<(), std::io::Error> {
                if self.first {
                    self.first = false;
                    self.writer.write_all(b"{")?;
                } else {
                    self.writer.write_all(b",")?;
                }
                self.writer.write_all(name.as_str().as_bytes())?;
                x.visit(Visitor { writer: self.writer })
            }
        }

        let mut visitor = GroupVisitor { first: true, writer: &mut self.writer };
        labels.visit_values(&mut visitor);
        if !visitor.first {
            self.writer.write_all(b"}")?;
        } else if braces {
            self.writer.write_all(b"{}")?;
        }
        Ok(())
    }

    fn write_value(&mut self, value: MetricValue) -> io::Result<()> {
        match value {
            MetricValue::Int(x) => self
                .writer
                .write_all(x.format_into(&mut core::fmt::NumBuffer::new()).as_bytes()),
            MetricValue::Uint(x) => self
                .writer
                .write_all(x.format_into(&mut core::fmt::NumBuffer::new()).as_bytes()),
            MetricValue::Float(x) => self
                .writer
                .write_all(format_float(&mut zmij::Buffer::new(), x).as_bytes()),
        }
    }

    fn write_metric_value(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: MetricValue,
    ) -> io::Result<()> {
        self.write_sample(name, labels, value, None)
    }

    fn write_sample(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: MetricValue,
        exemplar: Option<&Exemplar>,
    ) -> io::Result<()> {
        self.state = State::Metrics;
        self.write_sample_name(name)?;
        self.write_labels(labels, false)?;
        self.writer.write_all(b" ")?;
        self.write_value(value)?;

        // OpenMetrics limits exemplar labels to 128 characters, and prometheus rejects longer ones.
        if let Some(exemplar) = exemplar.filter(|e| e.labels_len() <= 128) {
            self.writer.write_all(b" # ")?;
            self.write_labels(exemplar.labels(), true)?;
            self.writer.write_all(b" ")?;
            self.write_value(MetricValue::Float(exemplar.value()))?;
            if let Some(timestamp) = exemplar.timestamp() {
                self.writer.write_all(b" ")?;
                self.write_value(MetricValue::Float(unix_seconds(timestamp)))?;
            }
        }

        self.writer.write_all(b"\n")
    }

    /// Write a counter-like sample, with the exemplar and created timestamp if this is OpenMetrics
    fn write_counter_sample(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: MetricValue,
        extras: SampleExtras<'_>,
    ) -> io::Result<()> {
        if Self::FORMAT == Format::Classic {
            return self.write_sample(name, labels, value, None);
        }

        self.write_sample(name, labels.by_ref(), value, extras.exemplar)?;
        if let Some(created) = extras.created {
            self.writer.write_all(&self.family)?;
            self.writer.write_all(b"_created")?;
            self.write_labels(labels, false)?;
            self.writer.write_all(b" ")?;
            self.write_value(MetricValue::Float(unix_seconds(created)))?;
            self.writer.write_all(b"\n")?;
        }
        Ok(())
    }
}

/// The `le` label of a histogram bucket
struct Le(f64);

impl LabelGroup for Le {
    fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
        const LE: &LabelName = LabelName::from_str("le");
        v.write_value(LE, &self.0);
    }
}

/// Forwards everything except `write_gauge_histogram`, to use its default implementation.
struct DefaultGaugeHistogram<'a, E>(&'a mut E);

impl<E: Encoding> Encoding for DefaultGaugeHistogram<'_, E> {
    type Err = E::Err;
    const MIME_TYPE: &'static str = E::MIME_TYPE;

    fn write_family(
        &mut self,
        name: impl MetricNameEncoder,
        kind: MetricKind,
        meta: FamilyMeta<'_>,
    ) -> Result<(), E::Err> {
        self.0.write_family(name, kind, meta)
    }

    fn write_counter(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: u64,
        extras: SampleExtras<'_>,
    ) -> Result<(), E::Err> {
        self.0.write_counter(name, labels, value, extras)
    }

    fn write_float_counter(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: f64,
        extras: SampleExtras<'_>,
    ) -> Result<(), E::Err> {
        self.0.write_float_counter(name, labels, value, extras)
    }

    fn write_gauge(&mut self, name: impl MetricNameEncoder, labels: impl LabelGroup, value: i64) -> Result<(), E::Err> {
        self.0.write_gauge(name, labels, value)
    }

    fn write_float_gauge(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: f64,
    ) -> Result<(), E::Err> {
        self.0.write_float_gauge(name, labels, value)
    }
}

macro_rules! text_encoder {
    ($(#[$meta:meta])* $encoder:ident, $(#[$buffered_meta:meta])* $buffered:ident, $open_metrics:literal, $mime:literal) => {
        $(#[$meta])*
        pub struct $encoder<W>(Inner<W, $open_metrics>);

        impl<W: Write> $encoder<W> {
            /// Create a new encoder that writes into `w`.
            pub fn from_writer(w: W) -> Self {
                Self(Inner::new(w))
            }

            /// Finish this collection and flush the inner writer.
            ///
            /// The OpenMetrics encoder ends each collection with `# EOF`, so only call this once per collection.
            pub fn flush(&mut self) -> io::Result<()> {
                self.0.finish()?;
                self.0.writer.flush()
            }

            /// Borrow the inner writer.
            pub fn writer(&self) -> &W {
                &self.0.writer
            }

            /// Mutably borrow the inner writer.
            ///
            /// Writing to the inner writer directly in the middle of a collection can produce invalid output.
            pub fn writer_mut(&mut self) -> &mut W {
                &mut self.0.writer
            }

            /// Return the inner writer. Call `flush` first to finish the collection.
            pub fn into_inner(self) -> W {
                self.0.writer
            }
        }

        $(#[$buffered_meta])*
        pub type $buffered = $encoder<BytesWriter>;

        impl Default for $buffered {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $buffered {
            /// Create a new buffered encoder.
            ///
            /// This should ideally be cached and re-used between collections to reduce re-allocating
            pub fn new() -> Self {
                Self::from_writer(BytesWriter {
                    buf: BytesMut::new(),
                })
            }

            /// Finish the encoding and extract the bytes to send in a HTTP response.
            ///
            /// The OpenMetrics encoder ends each collection with `# EOF`, so only call this once per collection.
            pub fn finish(&mut self) -> Bytes {
                self.0
                    .finish()
                    .expect("writing to a buffer should not fail");
                self.0.writer.buf.split().freeze()
            }
        }

        impl<W: Write> Encoding for $encoder<W> {
            type Err = std::io::Error;
            const MIME_TYPE: &'static str = $mime;

            fn write_family(
                &mut self,
                name: impl MetricNameEncoder,
                kind: MetricKind,
                meta: FamilyMeta<'_>,
            ) -> Result<(), std::io::Error> {
                self.0.write_family(name, kind, meta)
            }

            fn write_counter(
                &mut self,
                name: impl MetricNameEncoder,
                labels: impl LabelGroup,
                value: u64,
                extras: SampleExtras<'_>,
            ) -> Result<(), std::io::Error> {
                self.0
                    .write_counter_sample(name, labels, MetricValue::Uint(value), extras)
            }

            fn write_float_counter(
                &mut self,
                name: impl MetricNameEncoder,
                labels: impl LabelGroup,
                value: f64,
                extras: SampleExtras<'_>,
            ) -> Result<(), std::io::Error> {
                self.0
                    .write_counter_sample(name, labels, MetricValue::Float(value), extras)
            }

            fn write_gauge(
                &mut self,
                name: impl MetricNameEncoder,
                labels: impl LabelGroup,
                value: i64,
            ) -> Result<(), std::io::Error> {
                self.0.write_metric_value(name, labels, MetricValue::Int(value))
            }

            fn write_float_gauge(
                &mut self,
                name: impl MetricNameEncoder,
                labels: impl LabelGroup,
                value: f64,
            ) -> Result<(), std::io::Error> {
                self.0.write_metric_value(name, labels, MetricValue::Float(value))
            }

            fn write_gauge_histogram(
                &mut self,
                name: impl MetricNameEncoder,
                labels: impl LabelGroup,
                le: &[f64],
                counts: &[i64],
                inf: i64,
                sum: f64,
            ) -> Result<(), std::io::Error> {
                match Inner::<W, $open_metrics>::FORMAT {
                    // the prometheus text format has no gauge histograms, so write it as a histogram,
                    // with `_sum` and `_count` instead of `_gsum` and `_gcount`.
                    Format::Classic => {
                        let mut cumulative = 0;
                        for (&le, &count) in le.iter().zip(counts) {
                            cumulative += count;
                            self.write_gauge(
                                name.by_ref().with_suffix(Bucket),
                                labels.by_ref().compose_with(Le(le)),
                                cumulative,
                            )?;
                        }
                        let count = cumulative + inf;
                        self.write_gauge(
                            name.by_ref().with_suffix(Bucket),
                            labels.by_ref().compose_with(Le(f64::INFINITY)),
                            count,
                        )?;
                        self.write_float_gauge(name.by_ref().with_suffix(Sum), labels.by_ref(), sum)?;
                        self.write_gauge(name.by_ref().with_suffix(Count), labels, count)
                    }
                    Format::OpenMetrics => {
                        DefaultGaugeHistogram(self).write_gauge_histogram(name, labels, le, counts, inf, sum)
                    }
                }
            }
        }
    };
}

text_encoder!(
    /// The prometheus text format encoder, writing into any [`Write`].
    ///
    /// This writes the classic prometheus text format (`version=0.0.4`), which supports counters,
    /// gauges, histograms and summaries. Other metric types are written as gauges or untyped.
    ///
    /// See [`BufferedTextEncoder`] for an encoder that writes into an in-memory buffer.
    TextEncoder,
    /// A [`TextEncoder`] that writes into an in-memory buffer.
    ///
    /// Writing into the buffer never fails.
    BufferedTextEncoder,
    false,
    "text/plain; version=0.0.4"
);

text_encoder!(
    /// The OpenMetrics text format encoder, writing into any [`Write`].
    ///
    /// This writes the OpenMetrics text format (`version=1.0.0`), which supports every [`MetricKind`].
    /// Following OpenMetrics, a `_total` suffix is removed from counter family names, and added to
    /// counter samples if it is missing. The same is done with `_info` for info metrics.
    ///
    /// Prometheus scrapes OpenMetrics if it is configured to. Remember to call `flush` or `finish`,
    /// which ends the output with `# EOF`.
    ///
    /// See [`BufferedOpenMetricsEncoder`] for an encoder that writes into an in-memory buffer.
    OpenMetricsEncoder,
    /// An [`OpenMetricsEncoder`] that writes into an in-memory buffer.
    ///
    /// Writing into the buffer never fails.
    BufferedOpenMetricsEncoder,
    true,
    "application/openmetrics-text; version=1.0.0; charset=utf-8"
);

fn unix_seconds(t: std::time::SystemTime) -> f64 {
    match t.duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_secs_f64(),
        Err(e) => -e.duration().as_secs_f64(),
    }
}

fn format_float(buf: &mut zmij::Buffer, x: f64) -> &str {
    if x.is_nan() {
        "NaN"
    } else if x == f64::INFINITY {
        "+Inf"
    } else if x == f64::NEG_INFINITY {
        "-Inf"
    } else {
        buf.format_finite(x)
    }
}

fn write_help_str(s: &str, b: &mut impl Write) -> io::Result<()> {
    let mut i = 0;
    for j in memchr::memchr2_iter(b'\\', b'\n', s.as_bytes()) {
        b.write_all(&s.as_bytes()[i..j])?;
        match s.as_bytes()[j] {
            b'\\' => b.write_all(b"\\\\")?,
            b'\n' => b.write_all(b"\\n")?,
            _ => unreachable!(),
        }
        i = j + 1;
    }
    b.write_all(&s.as_bytes()[i..])
}

pub(crate) fn write_label_str_value(s: &str, b: &mut impl Write) -> io::Result<()> {
    let mut i = 0;
    for j in memchr3_iter(b'\\', b'"', b'\n', s.as_bytes()) {
        b.write_all(&s.as_bytes()[i..j])?;
        match s.as_bytes()[j] {
            b'\\' => b.write_all(b"\\\\")?,
            b'"' => b.write_all(b"\\\"")?,
            b'\n' => b.write_all(b"\\n")?,
            _ => unreachable!(),
        }
        i = j + 1;
    }
    b.write_all(&s.as_bytes()[i..])
}

/// The in-memory writer used by [`BufferedTextEncoder`].
pub struct BytesWriter {
    buf: BytesMut,
}

impl Write for BytesWriter {
    #[inline]
    fn write(&mut self, src: &[u8]) -> io::Result<usize> {
        self.write_all(src)?;
        Ok(src.len())
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.buf.put(buf);
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};
    use measured::label::StaticLabelSet;
    use measured::metric::MetricFamilyEncoding;
    use measured::metric::group::{Encoding, FamilyMeta, MetricKind};
    use measured::metric::histogram::Thresholds;
    use measured::metric::name::{MetricName, Total};
    use measured::{Counter, CounterVec, FloatGauge, Histogram};

    use super::{BufferedOpenMetricsEncoder, BufferedTextEncoder, write_label_str_value};

    #[test]
    fn invalid_units_are_rejected_before_output() {
        let mut encoder = BufferedOpenMetricsEncoder::new();
        for unit in ["", "a\nb", "a b"] {
            let err = encoder
                .write_family(
                    MetricName::from_str("value"),
                    MetricKind::Gauge,
                    FamilyMeta::new().with_unit(unit),
                )
                .unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        }
        assert_eq!(encoder.finish(), "# EOF\n");
    }

    #[test]
    fn write_encoded_str() {
        let mut b = BytesMut::new().writer();
        write_label_str_value(
            r#"Hello \ "World"
This is on a new line"#,
            &mut b,
        )
        .unwrap();

        assert_eq!(
            b.into_inner(),
            r#"Hello \\ \"World\"\nThis is on a new line"#
        );
    }

    #[derive(Clone, Copy, PartialEq, Debug, measured::LabelGroup)]
    #[label(set = RequestLabelSet)]
    struct RequestLabels {
        method: Method,
        code: StatusCode,
    }

    #[derive(Clone, Copy, PartialEq, Debug, measured::FixedCardinalityLabel)]
    #[label(rename_all = "snake_case")]
    enum Method {
        Post,
        Get,
    }

    #[derive(Clone, Copy, PartialEq, Debug, measured::FixedCardinalityLabel)]
    enum StatusCode {
        Ok = 200,
        BadRequest = 400,
    }

    #[test]
    fn text_encoding() {
        let requests = CounterVec::with_label_set(RequestLabelSet {
            code: StaticLabelSet::new(),
            method: StaticLabelSet::new(),
        });

        let labels = RequestLabels {
            method: Method::Post,
            code: StatusCode::Ok,
        };
        requests.inc_by(labels, 1027);

        let labels = RequestLabels {
            method: Method::Get,
            code: StatusCode::BadRequest,
        };
        requests.inc_by(labels, 3);

        let mut encoder = BufferedTextEncoder::default();

        let name = MetricName::from_str("http_request").with_suffix(Total);
        requests
            .collect_family_into(
                name,
                FamilyMeta::new().with_help("The total number of HTTP requests."),
                &mut encoder,
            )
            .unwrap();

        let s = String::from_utf8(encoder.finish().to_vec()).unwrap();
        assert_eq!(
            s,
            r#"# HELP http_request_total The total number of HTTP requests.
# TYPE http_request_total counter
http_request_total{method="post",code="200"} 1027
http_request_total{method="get",code="400"} 3
"#
        );
    }

    #[test]
    fn text_histogram() {
        let thresholds = Thresholds::<8>::exponential_buckets(0.1, 2.0);
        let histogram = Histogram::with_metadata(thresholds);

        histogram.get_metric().observe(0.7);
        histogram.get_metric().observe(2.5);
        histogram.get_metric().observe(1.2);
        histogram.get_metric().observe(8.0);

        let mut encoder = BufferedTextEncoder::default();

        let name = MetricName::from_str("http_request_duration_seconds");
        histogram
            .collect_family_into(
                name,
                FamilyMeta::new().with_help("A histogram of the request duration."),
                &mut encoder,
            )
            .unwrap();

        let s = String::from_utf8(encoder.finish().to_vec()).unwrap();
        assert_eq!(
            s,
            r#"# HELP http_request_duration_seconds A histogram of the request duration.
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{le="0.1"} 0
http_request_duration_seconds_bucket{le="0.2"} 0
http_request_duration_seconds_bucket{le="0.4"} 0
http_request_duration_seconds_bucket{le="0.8"} 1
http_request_duration_seconds_bucket{le="1.6"} 2
http_request_duration_seconds_bucket{le="3.2"} 3
http_request_duration_seconds_bucket{le="6.4"} 3
http_request_duration_seconds_bucket{le="12.8"} 4
http_request_duration_seconds_bucket{le="+Inf"} 4
http_request_duration_seconds_sum 12.4
http_request_duration_seconds_count 4
"#
        );
    }

    /// See <https://github.com/conradludgate/measured/issues/8>
    #[test]
    fn text_encoding_rename() {
        #[derive(Clone, Copy, PartialEq, Debug, measured::LabelGroup)]
        #[label(set = RequestLabelSet)]
        struct RequestLabels {
            method: Method,
            code: StatusCode,
        }

        #[derive(Clone, Copy, PartialEq, Debug, measured::FixedCardinalityLabel)]
        enum StatusCode {
            #[label(rename = "ok")]
            Ok = 200,

            #[label(rename = "badrequest")]
            BadRequest = 400,
        }

        let requests = CounterVec::with_label_set(RequestLabelSet {
            code: StaticLabelSet::new(),
            method: StaticLabelSet::new(),
        });

        let labels = RequestLabels {
            method: Method::Post,
            code: StatusCode::Ok,
        };
        requests.inc_by(labels, 1027);

        let labels = RequestLabels {
            method: Method::Get,
            code: StatusCode::BadRequest,
        };
        requests.inc_by(labels, 3);

        let mut encoder = BufferedTextEncoder::default();

        let name = MetricName::from_str("http_request").with_suffix(Total);
        requests
            .collect_family_into(
                name,
                FamilyMeta::new().with_help("The total number of HTTP requests."),
                &mut encoder,
            )
            .unwrap();

        let s = String::from_utf8(encoder.finish().to_vec()).unwrap();
        assert_eq!(
            s,
            r#"# HELP http_request_total The total number of HTTP requests.
# TYPE http_request_total counter
http_request_total{method="post",code="ok"} 1027
http_request_total{method="get",code="badrequest"} 3
"#
        );
    }

    #[test]
    fn help_is_escaped() {
        let mut encoder = BufferedTextEncoder::default();
        let name = MetricName::from_str("foo");
        encoder
            .write_family(
                name,
                MetricKind::Gauge,
                FamilyMeta::new().with_help("line one\nline \\two \"quoted\""),
            )
            .unwrap();

        let s = String::from_utf8(encoder.finish().to_vec()).unwrap();
        assert_eq!(
            s,
            "# HELP foo line one\\nline \\\\two \"quoted\"\n# TYPE foo gauge\n"
        );
    }

    #[test]
    fn special_values() {
        let mut encoder = BufferedTextEncoder::default();

        let counter = Counter::new();
        counter.inc_by(u64::MAX);
        counter
            .collect_family_into(MetricName::from_str("big"), FamilyMeta::new(), &mut encoder)
            .unwrap();

        for (name, x) in [
            ("pos_inf", f64::INFINITY),
            ("neg_inf", f64::NEG_INFINITY),
            ("nan", f64::NAN),
        ] {
            let gauge = FloatGauge::new();
            gauge.set(x);
            gauge
                .collect_family_into(MetricName::from_str(name), FamilyMeta::new(), &mut encoder)
                .unwrap();
        }

        let s = String::from_utf8(encoder.finish().to_vec()).unwrap();
        assert_eq!(
            s,
            r#"# TYPE big counter
big 18446744073709551615

# TYPE pos_inf gauge
pos_inf +Inf

# TYPE neg_inf gauge
neg_inf -Inf

# TYPE nan gauge
nan NaN
"#
        );
    }

    #[test]
    fn histogram_nan() {
        let histogram = Histogram::with_metadata(Thresholds::with_buckets([1.0, 2.0]));
        histogram.get_metric().observe(f64::NAN);
        histogram.get_metric().observe(1.5);

        let mut encoder = BufferedTextEncoder::default();
        histogram
            .collect_family_into(MetricName::from_str("h"), FamilyMeta::new(), &mut encoder)
            .unwrap();

        let s = String::from_utf8(encoder.finish().to_vec()).unwrap();
        assert_eq!(
            s,
            r#"# TYPE h histogram
h_bucket{le="1.0"} 0
h_bucket{le="2.0"} 1
h_bucket{le="+Inf"} 2
h_sum NaN
h_count 2
"#
        );
    }

    #[test]
    fn empty_thresholds() {
        let histogram = Histogram::with_metadata(Thresholds::<0>::with_buckets([]));
        histogram.get_metric().observe(1.0);
    }

    #[test]
    #[should_panic = "must not be NaN"]
    fn nan_threshold() {
        Thresholds::with_buckets([f64::NAN]);
    }

    #[derive(Clone, Copy, PartialEq, Debug, measured::FixedCardinalityLabel)]
    enum Status {
        Starting,
        Ready,
    }

    #[derive(measured::LabelGroup)]
    struct Build {
        version: &'static str,
    }

    #[derive(measured::MetricGroup)]
    #[metric(new())]
    struct AllKinds {
        /// requests "served"
        requests_total: Counter,
        /// a counter without the suffix
        errors: Counter,
        /// cpu time
        cpu_seconds_total: measured::FloatCounter,
        /// build information
        #[metric(init = measured::InfoMetric::new(Build { version: "1.0" }))]
        build_info: measured::InfoMetric<Build>,
        /// server status
        #[metric(init = measured::StateSet::new(Status::Ready))]
        status: measured::StateSet<Status>,
        /// ages of queued items
        #[metric(metadata = Thresholds::with_buckets([1.0]))]
        queue_age_seconds: measured::GaugeHistogram<1>,
        /// latency
        #[metric(metadata = Thresholds::with_buckets([1.0]))]
        latency_seconds: Histogram<1>,
    }

    fn all_kinds() -> AllKinds {
        let metrics = AllKinds::new();
        metrics.requests_total.inc_by(3);
        metrics.errors.inc();
        metrics.cpu_seconds_total.inc_by(1.5);
        metrics.queue_age_seconds.observe(0.5);
        metrics.queue_age_seconds.observe(2.0);
        metrics.queue_age_seconds.observe(3.0);
        metrics.queue_age_seconds.remove(3.0);
        metrics.latency_seconds.observe(0.5);
        metrics
    }

    #[test]
    fn open_metrics() {
        use measured::MetricGroup;

        let mut encoder = BufferedOpenMetricsEncoder::new();
        all_kinds().collect_group_into(&mut encoder).unwrap();
        let s = String::from_utf8(encoder.finish().to_vec()).unwrap();
        assert_eq!(
            s,
            r#"# TYPE requests counter
# HELP requests requests \"served\"
requests_total 3
# TYPE errors counter
# HELP errors a counter without the suffix
errors_total 1
# TYPE cpu_seconds counter
# HELP cpu_seconds cpu time
cpu_seconds_total 1.5
# TYPE build info
# HELP build build information
build_info{version="1.0"} 1
# TYPE status stateset
# HELP status server status
status{status="starting"} 0
status{status="ready"} 1
# TYPE queue_age_seconds gaugehistogram
# HELP queue_age_seconds ages of queued items
queue_age_seconds_bucket{le="1.0"} 1
queue_age_seconds_bucket{le="+Inf"} 2
queue_age_seconds_gcount 2
queue_age_seconds_gsum 2.5
# TYPE latency_seconds histogram
# HELP latency_seconds latency
latency_seconds_bucket{le="1.0"} 1
latency_seconds_bucket{le="+Inf"} 1
latency_seconds_sum 0.5
latency_seconds_count 1
# EOF
"#
        );
    }

    #[test]
    fn classic_fallbacks() {
        use measured::MetricGroup;

        let mut encoder = BufferedTextEncoder::new();
        all_kinds().collect_group_into(&mut encoder).unwrap();
        let s = String::from_utf8(encoder.finish().to_vec()).unwrap();
        assert!(s.contains("# TYPE errors counter\nerrors 1\n"), "{s}");
        assert!(s.contains("# TYPE build_info gauge\nbuild_info{version=\"1.0\"} 1\n"));
        assert!(s.contains("# TYPE status gauge\nstatus{status=\"starting\"} 0\n"));
        assert!(s.contains(
            "# TYPE queue_age_seconds histogram\nqueue_age_seconds_bucket{le=\"1.0\"} 1\nqueue_age_seconds_bucket{le=\"+Inf\"} 2\nqueue_age_seconds_sum 2.5\nqueue_age_seconds_count 2\n"
        ));
        assert!(!s.contains("# EOF"));
    }

    #[derive(measured::MetricGroup)]
    #[metric(new())]
    struct Extras {
        /// requests
        requests_total: measured::ExemplarCounter,
        /// latency
        #[metric(metadata = Thresholds::with_buckets([1.0]))]
        latency_seconds: measured::ExemplarHistogram<1>,
        /// created
        created_total: measured::CreatedCounter,
        /// created latency
        #[metric(metadata = Thresholds::with_buckets([1.0]))]
        created_latency_seconds: measured::CreatedHistogram<1>,
    }

    fn extras() -> Extras {
        use measured::label::LabelName;
        use measured::metric::exemplar::Exemplar;
        const TRACE_ID: &LabelName = LabelName::from_str("trace_id");

        let at = std::time::UNIX_EPOCH + std::time::Duration::from_millis(1_700_000_000_500);
        let metrics = Extras::new();
        metrics.requests_total.inc_with_exemplar(
            2,
            Exemplar::new()
                .with_label(TRACE_ID, "abc")
                .with_timestamp(at),
        );
        metrics
            .latency_seconds
            .observe_with_exemplar(0.5, Exemplar::new().with_label(TRACE_ID, "def"));
        // too long for OpenMetrics, so this is not written
        metrics
            .latency_seconds
            .observe_with_exemplar(2.0, Exemplar::new().with_label(TRACE_ID, "x".repeat(200)));
        metrics.created_total.inc();
        metrics.created_latency_seconds.observe(0.5);
        metrics
    }

    #[test]
    fn open_metrics_extras() {
        use measured::MetricGroup;

        let before = super::unix_seconds(std::time::SystemTime::now());
        let metrics = extras();
        let mut encoder = BufferedOpenMetricsEncoder::new();
        metrics.collect_group_into(&mut encoder).unwrap();
        let s = String::from_utf8(encoder.finish().to_vec()).unwrap();
        let after = super::unix_seconds(std::time::SystemTime::now());

        assert!(
            s.contains("requests_total 2 # {trace_id=\"abc\"} 2.0 1700000000.5\n"),
            "{s}"
        );
        assert!(s.contains("latency_seconds_bucket{le=\"1.0\"} 1 # {trace_id=\"def\"} 0.5\n"));
        assert!(s.contains("latency_seconds_bucket{le=\"+Inf\"} 2\n"));

        let created: Vec<f64> = s
            .lines()
            .filter(|l| l.starts_with("created_created ") || l.starts_with("created_latency_seconds_created "))
            .map(|l| l.rsplit(' ').next().unwrap().parse().unwrap())
            .collect();
        assert_eq!(created.len(), 2, "{s}");
        assert!(created.iter().all(|&t| before <= t && t <= after));

        // the created line follows the counter sample, and the histogram count
        assert!(s.contains("created_total 1\ncreated_created "));
        assert!(s.contains("created_latency_seconds_count 1\ncreated_latency_seconds_created "));
    }

    #[test]
    fn classic_ignores_extras() {
        use measured::MetricGroup;

        let mut encoder = BufferedTextEncoder::new();
        extras().collect_group_into(&mut encoder).unwrap();
        let s = String::from_utf8(encoder.finish().to_vec()).unwrap();
        assert!(!s.contains(" # {"), "{s}");
        assert!(!s.contains("_created"), "{s}");
        assert!(s.contains("requests_total 2\n"));
    }
    #[test]
    fn summary_defaults_and_namespace_forwarding_preserve_extras() {
        use measured::label::NoLabels;
        use measured::metric::group::SampleExtras;
        use measured::metric::name::WithNamespace;
        macro_rules! check {
            ($encoder:ty, $expected:expr) => {{
                let mut encoder = <$encoder>::new();
                let mut enc = WithNamespace::new("http", &mut encoder);
                let name = MetricName::from_str("latency_seconds");
                enc.write_family(name, MetricKind::Summary, FamilyMeta::new())
                    .unwrap();
                enc.write_summary(
                    name,
                    NoLabels,
                    &[(0.5, 1.0)],
                    2,
                    3.0,
                    SampleExtras::NONE.with_created(Some(std::time::UNIX_EPOCH)),
                )
                .unwrap();
                assert_eq!(encoder.finish(), $expected);
            }};
        }
        check!(
            BufferedTextEncoder,
            "# TYPE http_latency_seconds summary\nhttp_latency_seconds{quantile=\"0.5\"} 1.0\nhttp_latency_seconds_sum 3.0\nhttp_latency_seconds_count 2\n"
        );
        check!(
            BufferedOpenMetricsEncoder,
            "# TYPE http_latency_seconds summary\nhttp_latency_seconds{quantile=\"0.5\"} 1.0\nhttp_latency_seconds_sum 3.0\nhttp_latency_seconds_count 2\nhttp_latency_seconds_created 0.0\n# EOF\n"
        );
    }

    #[test]
    fn labelled_extras_and_infinite_bucket_exemplars_survive_namespacing() {
        use measured::label::LabelName;
        use measured::metric::name::WithNamespace;
        use measured::{CreatedCounterVec, Exemplar, ExemplarCounterVec, ExemplarHistogram};
        let labels = RequestLabels {
            method: Method::Post,
            code: StatusCode::Ok,
        };
        let counter = ExemplarCounterVec::<RequestLabelSet>::new();
        counter.inc_with_exemplar(
            labels,
            1,
            Exemplar::new().with_label(LabelName::from_str("trace"), "a\"\\\n"),
        );
        let created = CreatedCounterVec::<RequestLabelSet>::new();
        created.inc(labels);
        let histogram = ExemplarHistogram::with_metadata(Thresholds::with_buckets([1.0]));
        histogram.observe_with_exemplar(
            f64::INFINITY,
            Exemplar::new().with_label(LabelName::from_str("trace"), "inf"),
        );
        let mut encoder = BufferedOpenMetricsEncoder::new();
        let mut enc = WithNamespace::new("http", &mut encoder);
        counter
            .collect_family_into(
                MetricName::from_str("requests_total"),
                FamilyMeta::new(),
                &mut enc,
            )
            .unwrap();
        created
            .collect_family_into(
                MetricName::from_str("created_total"),
                FamilyMeta::new(),
                &mut enc,
            )
            .unwrap();
        histogram
            .collect_family_into(MetricName::from_str("latency"), FamilyMeta::new(), &mut enc)
            .unwrap();
        let output = String::from_utf8(encoder.finish().to_vec()).unwrap();
        assert!(
            output.contains("http_requests_total{method=\"post\",code=\"200\"} 1 # {trace=\"a\\\"\\\\\\n\"} 1.0\n"),
            "{output}"
        );
        assert!(
            output.contains("http_created_created{method=\"post\",code=\"200\"} "),
            "{output}"
        );
        assert!(
            output.contains("http_latency_bucket{le=\"+Inf\"} 1 # {trace=\"inf\"} +Inf\n"),
            "{output}"
        );
    }
}
