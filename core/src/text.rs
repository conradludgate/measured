//! Prometheus Text based exporter

use std::{
    convert::Infallible,
    io::{self, Write},
};

use bytes::{BufMut, Bytes, BytesMut};
use memchr::memchr3_iter;

use crate::{
    label::{LabelGroup, LabelGroupVisitor, LabelName, LabelValue, LabelVisitor},
    metric::{
        counter::CounterState,
        gauge::GaugeState,
        group::{Encoding, MetricValue},
        histogram::{HistogramState, Thresholds},
        name::{Bucket, Count, MetricNameEncoder, Sum},
        MetricEncoding,
    },
};

/// The prometheus text encoder helper
pub struct TextEncoder<W> {
    state: State,
    pub writer: W,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Info,
    Metrics,
}

/// Prometheus only supports these 5 types of metrics
#[derive(Clone, Copy, Debug)]
pub enum MetricType {
    Counter,
    Histogram,
    Gauge,
    Summary,
    Untyped,
}

impl<W: Write> Encoding for TextEncoder<W> {
    type Err = std::io::Error;

    /// Write the help line for a metric
    fn write_help(
        &mut self,
        name: impl MetricNameEncoder,
        help: &str,
    ) -> Result<(), std::io::Error> {
        if self.state == State::Metrics {
            self.write_line()?;
        }
        self.state = State::Info;

        self.writer.write_all(b"# HELP ")?;
        name.encode_text(&mut self.writer)?;
        self.writer.write_all(b" ")?;
        self.writer.write_all(help.as_bytes())?;
        self.writer.write_all(b"\n")?;
        Ok(())
    }

    /// Write the metric data
    fn write_metric_value(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: MetricValue,
    ) -> Result<(), std::io::Error> {
        struct Visitor<'a, W> {
            writer: &'a mut W,
        }
        impl<W: Write> LabelVisitor for Visitor<'_, W> {
            type Output = Result<(), std::io::Error>;
            fn write_int(self, x: i64) -> Result<(), std::io::Error> {
                self.write_str(itoa::Buffer::new().format(x))
            }

            fn write_float(self, x: f64) -> Result<(), std::io::Error> {
                if x.is_infinite() {
                    if x.is_sign_positive() {
                        self.write_str("+Inf")
                    } else {
                        self.write_str("-Inf")
                    }
                } else if x.is_nan() {
                    self.write_str("NaN")
                } else {
                    self.write_str(ryu::Buffer::new().format(x))
                }
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
            fn write_value(
                &mut self,
                name: &LabelName,
                x: &impl LabelValue,
            ) -> Result<(), std::io::Error> {
                if self.first {
                    self.first = false;
                    self.writer.write_all(b"{")?;
                } else {
                    self.writer.write_all(b",")?;
                }
                self.writer.write_all(name.as_str().as_bytes())?;
                x.visit(Visitor {
                    writer: self.writer,
                })
            }
        }

        self.state = State::Metrics;
        name.encode_text(&mut self.writer)?;

        let mut visitor = GroupVisitor {
            first: true,
            writer: &mut self.writer,
        };
        labels.visit_values(&mut visitor);
        if !visitor.first {
            self.writer.write_all(b"}")?;
        }
        self.writer.write_all(b" ")?;
        match value {
            MetricValue::Int(x) => self
                .writer
                .write_all(itoa::Buffer::new().format(x).as_bytes())?,
            MetricValue::Float(x) => self
                .writer
                .write_all(ryu::Buffer::new().format(x).as_bytes())?,
        }
        self.writer.write_all(b"\n")?;
        Ok(())
    }
}

impl<W: Write> TextEncoder<W> {
    /// Create a new text encoder.
    ///
    /// This should ideally be cached and re-used between collections to reduce re-allocating
    pub fn new(w: W) -> Self {
        Self {
            state: State::Info,
            writer: w,
        }
    }

    /// Finish the text encoding and extract the bytes to send in a HTTP response.
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.state = State::Info;
        self.writer.flush()
    }

    fn write_line(&mut self) -> std::io::Result<()> {
        self.writer.write_all(b"\n")
    }

    /// Write the type line for a metric
    pub fn write_type(
        &mut self,
        name: &impl MetricNameEncoder,
        typ: MetricType,
    ) -> Result<(), std::io::Error> {
        if self.state == State::Metrics {
            self.write_line()?;
        }
        self.state = State::Info;

        self.writer.write_all(b"# TYPE ")?;
        name.encode_text(&mut self.writer)?;
        match typ {
            MetricType::Counter => self.writer.write_all(b" counter\n"),
            MetricType::Histogram => self.writer.write_all(b" histogram\n"),
            MetricType::Gauge => self.writer.write_all(b" gauge\n"),
            MetricType::Summary => self.writer.write_all(b" summary\n"),
            MetricType::Untyped => self.writer.write_all(b" untyped\n"),
        }
    }
}

impl<W: Write, const N: usize> MetricEncoding<TextEncoder<W>> for HistogramState<N> {
    fn write_type(
        name: impl MetricNameEncoder,
        enc: &mut TextEncoder<W>,
    ) -> Result<(), std::io::Error> {
        enc.write_type(&name, MetricType::Histogram)
    }
    fn collect_into(
        &self,
        metadata: &Thresholds<N>,
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut TextEncoder<W>,
    ) -> Result<(), std::io::Error> {
        struct F64(f64);
        impl LabelValue for F64 {
            fn visit<V: LabelVisitor>(&self, v: V) -> V::Output {
                v.write_float(self.0)
            }
        }

        struct HistogramLabelLe {
            le: f64,
        }

        impl LabelGroup for HistogramLabelLe {
            fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
                const LE: &LabelName = LabelName::from_str("le");
                v.write_value(LE, &F64(self.le));
            }
        }

        let (buckets, inf, sum) = self.inner.write().sample();
        let mut val = 0;

        #[allow(clippy::needless_range_loop)]
        for i in 0..N {
            let le = metadata.get()[i];
            val += buckets[i];
            enc.write_metric_value(
                &name.by_ref().with_suffix(Bucket),
                labels.by_ref().compose_with(HistogramLabelLe { le }),
                MetricValue::Int(val as i64),
            )?;
        }
        let count = val + inf;
        enc.write_metric_value(
            &name.by_ref().with_suffix(Bucket),
            labels
                .by_ref()
                .compose_with(HistogramLabelLe { le: f64::INFINITY }),
            MetricValue::Int(count as i64),
        )?;
        enc.write_metric_value(
            &name.by_ref().with_suffix(Sum),
            labels.by_ref(),
            MetricValue::Float(sum),
        )?;
        enc.write_metric_value(
            &name.by_ref().with_suffix(Count),
            labels,
            MetricValue::Int(count as i64),
        )?;
        Ok(())
    }
}

impl<W: Write> MetricEncoding<TextEncoder<W>> for CounterState {
    fn write_type(
        name: impl MetricNameEncoder,
        enc: &mut TextEncoder<W>,
    ) -> Result<(), std::io::Error> {
        enc.write_type(&name, MetricType::Counter)
    }
    fn collect_into(
        &self,
        _m: &(),
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut TextEncoder<W>,
    ) -> Result<(), std::io::Error> {
        enc.write_metric_value(
            &name,
            labels,
            MetricValue::Int(self.count.load(core::sync::atomic::Ordering::Relaxed) as i64),
        )
    }
}

impl<W: Write> MetricEncoding<TextEncoder<W>> for GaugeState {
    fn write_type(
        name: impl MetricNameEncoder,
        enc: &mut TextEncoder<W>,
    ) -> Result<(), std::io::Error> {
        enc.write_type(&name, MetricType::Gauge)
    }
    fn collect_into(
        &self,
        _m: &(),
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut TextEncoder<W>,
    ) -> Result<(), std::io::Error> {
        enc.write_metric_value(
            &name,
            labels,
            MetricValue::Int(self.count.load(core::sync::atomic::Ordering::Relaxed)),
        )
    }
}

/// The prometheus text encoder helper
pub struct BufferedTextEncoder {
    inner: TextEncoder<BytesWriter>,
}

impl Default for BufferedTextEncoder {
    fn default() -> Self {
        Self::new()
    }
}

trait Unreachable<T> {
    fn unreachable(self) -> Result<T, Infallible>;
}

impl<T, E: std::fmt::Debug> Unreachable<T> for Result<T, E> {
    fn unreachable(self) -> Result<T, Infallible> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => unreachable!("{e:?}"),
        }
    }
}

impl Encoding for BufferedTextEncoder {
    type Err = Infallible;

    /// Write the help line for a metric
    fn write_help(&mut self, name: impl MetricNameEncoder, help: &str) -> Result<(), Infallible> {
        self.inner.write_help(name, help).unreachable()
    }

    /// Write the metric data
    fn write_metric_value(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: MetricValue,
    ) -> Result<(), Infallible> {
        self.inner
            .write_metric_value(name, labels, value)
            .unreachable()
    }
}

impl BufferedTextEncoder {
    /// Create a new text encoder.
    ///
    /// This should ideally be cached and re-used between collections to reduce re-allocating
    pub fn new() -> Self {
        Self {
            inner: TextEncoder::new(BytesWriter {
                buf: BytesMut::new(),
            }),
        }
    }

    /// Finish the text encoding and extract the bytes to send in a HTTP response.
    pub fn finish(&mut self) -> Bytes {
        self.inner.flush().unreachable().unwrap();
        self.inner.writer.buf.split().freeze()
    }
}

impl<const N: usize> MetricEncoding<BufferedTextEncoder> for HistogramState<N> {
    fn write_type(
        name: impl MetricNameEncoder,
        enc: &mut BufferedTextEncoder,
    ) -> Result<(), Infallible> {
        Self::write_type(name, &mut enc.inner).unreachable()
    }
    fn collect_into(
        &self,
        metadata: &Thresholds<N>,
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut BufferedTextEncoder,
    ) -> Result<(), Infallible> {
        self.collect_into(metadata, labels, name, &mut enc.inner)
            .unreachable()
    }
}

impl MetricEncoding<BufferedTextEncoder> for CounterState {
    fn write_type(
        name: impl MetricNameEncoder,
        enc: &mut BufferedTextEncoder,
    ) -> Result<(), Infallible> {
        Self::write_type(name, &mut enc.inner).unreachable()
    }
    fn collect_into(
        &self,
        metadata: &(),
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut BufferedTextEncoder,
    ) -> Result<(), Infallible> {
        self.collect_into(metadata, labels, name, &mut enc.inner)
            .unreachable()
    }
}

impl MetricEncoding<BufferedTextEncoder> for GaugeState {
    fn write_type(
        name: impl MetricNameEncoder,
        enc: &mut BufferedTextEncoder,
    ) -> Result<(), Infallible> {
        Self::write_type(name, &mut enc.inner).unreachable()
    }
    fn collect_into(
        &self,
        metadata: &(),
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut BufferedTextEncoder,
    ) -> Result<(), Infallible> {
        self.collect_into(metadata, labels, name, &mut enc.inner)
            .unreachable()
    }
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

struct BytesWriter {
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

    use crate::{
        label::StaticLabelSet,
        metric::{
            group::Encoding,
            histogram::Thresholds,
            name::{MetricName, Total},
            MetricFamilyEncoding,
        },
        CounterVec, Histogram,
    };

    use super::{write_label_str_value, BufferedTextEncoder};

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

    #[derive(Clone, Copy, PartialEq, Debug, measured_derive::LabelGroup)]
    #[label(crate = crate, set = RequestLabelSet)]
    struct RequestLabels {
        method: Method,
        code: StatusCode,
    }

    #[derive(Clone, Copy, PartialEq, Debug, measured_derive::FixedCardinalityLabel)]
    #[label(crate = crate, rename_all = "snake_case")]
    enum Method {
        Post,
        Get,
    }

    #[derive(Clone, Copy, PartialEq, Debug, measured_derive::FixedCardinalityLabel)]
    #[label(crate = crate)]
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
        encoder
            .write_help(&name, "The total number of HTTP requests.")
            .unwrap();
        requests.collect_family_into(name, &mut encoder).unwrap();

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
        encoder
            .write_help(name, "A histogram of the request duration.")
            .unwrap();
        histogram.collect_family_into(name, &mut encoder).unwrap();

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
}
