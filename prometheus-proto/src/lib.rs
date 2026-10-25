#![allow(clippy::cast_precision_loss)]

use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use encoding::WireType::{self, LengthDelimited};
use encoding::{encode_key, encode_varint, encoded_len_varint, key_len};
use measured::LabelGroup;
use measured::label::{LabelGroupVisitor, LabelName, LabelValue, LabelVisitor};
use measured::metric::exemplar::Exemplar;
use measured::metric::group::{Encoding, HistogramExtras, MetricKind, SampleExtras};
use measured::metric::name::MetricNameEncoder;

mod encoding;

/// The prometheus protobuf encoder.
///
/// Encodes metrics as a stream of length-delimited `io.prometheus.client.MetricFamily` messages.
pub struct ProtoEncoder<W> {
    state: State,
    untyped: bool,
    writer: W,
    buf: Vec<u8>,
}

impl<W: Write> ProtoEncoder<W> {
    /// Create a new protobuf encoder.
    ///
    /// This should ideally be cached and re-used between collections to reduce re-allocating
    pub fn new(w: W) -> Self {
        Self {
            state: State::Init,
            untyped: true,
            writer: w,
            buf: Vec::new(),
        }
    }

    /// Finish this collection, writing out the final metric family and flushing the inner writer.
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.flush_buf()?;
        self.writer.flush()
    }

    /// Borrow the inner writer.
    pub fn writer(&self) -> &W {
        &self.writer
    }

    /// Mutably borrow the inner writer.
    ///
    /// Writing to the inner writer directly in the middle of a collection can produce invalid output.
    pub fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Return the inner writer. Call [`ProtoEncoder::flush`] first, or the final metric family will be lost.
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Writes out the buffered metric family, if it has any metrics, and resets the buffer.
    ///
    /// Metric families with no metrics are dropped. The buffer is reset even if the write fails,
    /// so that a failed collection does not leak into the next one.
    fn flush_buf(&mut self) -> Result<(), std::io::Error> {
        let res = if self.state == State::Metrics {
            let len = self.buf.len() - 10;
            let varint_len = encoded_len_varint(len as u64);
            let offset = 10 - varint_len;
            encode_varint(len as u64, &mut &mut self.buf[offset..]);
            self.writer.write_all(&self.buf[offset..])
        } else {
            Ok(())
        };

        self.state = State::Init;
        self.untyped = true;
        self.buf.clear();
        self.buf.resize(10, 0);
        res
    }

    fn start_family(
        &mut self,
        name: impl MetricNameEncoder,
        help: Option<&str>,
        kind: MetricKind,
    ) -> Result<(), std::io::Error> {
        self.flush_buf()?;

        // optional string     name   = 1;
        encode_key(1, LengthDelimited, &mut self.buf);
        encode_varint(name.encode_len() as u64, &mut self.buf);
        name.encode_utf8(&mut self.buf)?;

        // optional string     help   = 2;
        if let Some(help) = help {
            encoding::encode_str(2, help, &mut self.buf);
        }

        // optional MetricType type   = 3;
        // Unsupported types fall back to untyped samples.
        let metric_type = match kind {
            MetricKind::Counter => 0,
            // protobuf has no state sets or info metrics, but their samples are gauges
            MetricKind::Gauge | MetricKind::StateSet | MetricKind::Info => 1,
            MetricKind::Summary => 2,
            MetricKind::Histogram => 4,
            MetricKind::GaugeHistogram => 5,
            _ => 3,
        };
        self.untyped = metric_type == 3;
        encoding::encode_i32(3, metric_type, &mut self.buf);

        self.state = State::Header;
        Ok(())
    }

    /// Write a metric into the current family, with the value in the given `Metric` field.
    fn write_metric(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value_tag: u32,
        value_len: usize,
        value: impl FnOnce(&mut Vec<u8>),
    ) -> Result<(), std::io::Error> {
        // samples should always be in a family, but don't produce an invalid message if not
        if self.state == State::Init {
            self.start_family(name, None, MetricKind::Untyped)?;
        }
        self.state = State::Metrics;

        let mut label_pairs_len = GroupLenVisitor { len: 0 };
        labels.visit_values(&mut label_pairs_len);
        let metric_len = label_pairs_len.len + message_len(value_tag, value_len);

        // repeated Metric     metric = 4;
        encode_message(4, metric_len, &mut self.buf, |buf| {
            labels.visit_values(&mut GroupVisitor { buf });
            encode_message(value_tag, value_len, buf, value);
        });

        Ok(())
    }

    fn write_double(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value_tag: u32,
        value: f64,
    ) -> Result<(), std::io::Error> {
        // optional Untyped untyped = 5;
        let value_tag = if self.untyped { 5 } else { value_tag };
        let value_len = encoding::encoded_len_f64(1, value);
        self.write_metric(name, labels, value_tag, value_len, |buf| {
            // optional double   value    = 1;
            encoding::encode_f64(1, value, buf);
        })
    }

    /// Write a `Counter` message
    fn write_counter_message(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: f64,
        extras: SampleExtras<'_>,
    ) -> Result<(), std::io::Error> {
        if self.untyped {
            // Untyped samples have no exemplar or created timestamp fields.
            return self.write_double(name, labels, 5, value);
        }

        let value_len = encoding::encoded_len_f64(1, value)
            + extras
                .exemplar
                .map_or(0, |e| message_len(2, exemplar_len(e)))
            + extras
                .created
                .map_or(0, |t| message_len(3, timestamp_len(t)));

        // optional Counter   counter      = 3;
        self.write_metric(name, labels, 3, value_len, |buf| {
            // optional double   value    = 1;
            encoding::encode_f64(1, value, buf);
            // optional Exemplar exemplar = 2;
            if let Some(exemplar) = extras.exemplar {
                encode_exemplar(2, exemplar, buf);
            }
            // optional google.protobuf.Timestamp created_timestamp = 3;
            if let Some(created) = extras.created {
                encode_timestamp(3, created, buf);
            }
        })
    }
}

/// Split a time into the seconds and nanos of a `google.protobuf.Timestamp`, where nanos are never negative.
fn timestamp_parts(t: SystemTime) -> (i64, i32) {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => (d.as_secs() as i64, d.subsec_nanos() as i32),
        Err(e) => {
            let d = e.duration();
            match d.subsec_nanos() {
                0 => (-(d.as_secs() as i64), 0),
                n => (-(d.as_secs() as i64) - 1, (1_000_000_000 - n) as i32),
            }
        }
    }
}

fn timestamp_len(t: SystemTime) -> usize {
    // proto3 fields are omitted when they are zero
    let (seconds, nanos) = timestamp_parts(t);
    let seconds_len = if seconds == 0 {
        0
    } else {
        key_len(1) + encoded_len_varint(seconds as u64)
    };
    let nanos_len = if nanos == 0 {
        0
    } else {
        key_len(2) + encoded_len_varint(nanos as u64)
    };
    seconds_len + nanos_len
}

fn encode_timestamp(tag: u32, t: SystemTime, buf: &mut Vec<u8>) {
    let (seconds, nanos) = timestamp_parts(t);
    encode_message(tag, timestamp_len(t), buf, |buf| {
        // int64 seconds = 1;
        if seconds != 0 {
            encode_key(1, WireType::Varint, buf);
            encode_varint(seconds as u64, buf);
        }
        // int32 nanos = 2;
        if nanos != 0 {
            encode_key(2, WireType::Varint, buf);
            encode_varint(nanos as u64, buf);
        }
    });
}

fn exemplar_len(exemplar: &Exemplar) -> usize {
    let mut labels_len = GroupLenVisitor { len: 0 };
    exemplar.labels().visit_values(&mut labels_len);
    labels_len.len
        + encoding::encoded_len_f64(2, exemplar.value())
        + exemplar
            .timestamp()
            .map_or(0, |t| message_len(3, timestamp_len(t)))
}

fn encode_exemplar(tag: u32, exemplar: &Exemplar, buf: &mut Vec<u8>) {
    encode_message(tag, exemplar_len(exemplar), buf, |buf| {
        // repeated LabelPair label = 1;
        exemplar.labels().visit_values(&mut GroupVisitor { buf });
        // optional double value = 2;
        encoding::encode_f64(2, exemplar.value(), buf);
        // optional google.protobuf.Timestamp timestamp = 3;
        if let Some(timestamp) = exemplar.timestamp() {
            encode_timestamp(3, timestamp, buf);
        }
    });
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Init,
    Header,
    Metrics,
}

impl<W: Write> Encoding for ProtoEncoder<W> {
    type Err = std::io::Error;

    const MIME_TYPE: &'static str = "application/vnd.google.protobuf; proto=io.prometheus.client.MetricFamily; encoding=delimited";

    fn write_family(
        &mut self,
        name: impl MetricNameEncoder,
        help: Option<&str>,
        kind: MetricKind,
    ) -> Result<(), std::io::Error> {
        self.start_family(name, help, kind)
    }

    fn write_counter(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: u64,
        extras: SampleExtras<'_>,
    ) -> Result<(), std::io::Error> {
        // protobuf counters are doubles, so this loses precision above 2^53
        self.write_counter_message(name, labels, value as f64, extras)
    }

    fn write_gauge(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: i64,
    ) -> Result<(), std::io::Error> {
        // optional Gauge     gauge        = 2;
        // protobuf gauges are doubles, so this loses precision above 2^53
        self.write_double(name, labels, 2, value as f64)
    }

    fn write_float_gauge(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: f64,
    ) -> Result<(), std::io::Error> {
        // optional Gauge     gauge        = 2;
        self.write_double(name, labels, 2, value)
    }

    fn write_float_counter(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: f64,
        extras: SampleExtras<'_>,
    ) -> Result<(), std::io::Error> {
        self.write_counter_message(name, labels, value, extras)
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
        // gauge histograms use the same message as histograms, in a GAUGE_HISTOGRAM family.
        // the counts should never be negative, but clamp them in case they are.
        let counts: Vec<u64> = counts.iter().map(|&c| c.max(0) as u64).collect();
        self.write_histogram(
            name,
            labels,
            le,
            &counts,
            inf.max(0) as u64,
            sum,
            HistogramExtras::NONE,
        )
    }

    fn write_histogram(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        le: &[f64],
        counts: &[u64],
        inf: u64,
        sum: f64,
        extras: HistogramExtras<'_>,
    ) -> Result<(), std::io::Error> {
        fn bucket_len(
            cumulative_count: u64,
            upper_bound: f64,
            exemplar: Option<&Exemplar>,
        ) -> usize {
            encoding::encoded_len_u64(1, cumulative_count)
                + encoding::encoded_len_f64(2, upper_bound)
                + exemplar.map_or(0, |e| message_len(3, exemplar_len(e)))
        }

        let count = counts.iter().sum::<u64>() + inf;

        let mut histogram_len = encoding::encoded_len_u64(1, count)
            + encoding::encoded_len_f64(2, sum)
            + extras
                .created
                .map_or(0, |t| message_len(15, timestamp_len(t)));
        // The +Inf count is normally implied by sample_count, but an explicit
        // bucket is needed to carry its exemplar.
        let buckets = le
            .iter()
            .copied()
            .zip(counts.iter().copied())
            .chain(extras.exemplar(le.len()).map(|_| (f64::INFINITY, inf)));
        let mut cumulative = 0;
        for (i, (upper_bound, n)) in buckets.clone().enumerate() {
            cumulative += n;
            histogram_len +=
                message_len(3, bucket_len(cumulative, upper_bound, extras.exemplar(i)));
        }

        // optional Histogram histogram    = 7;
        self.write_metric(name, labels, 7, histogram_len, |buf| {
            // optional uint64 sample_count = 1;
            encoding::encode_u64(1, count, buf);
            // optional double sample_sum   = 2;
            encoding::encode_f64(2, sum, buf);

            // repeated Bucket bucket       = 3;
            let mut cumulative = 0;
            for (i, (upper_bound, n)) in buckets.enumerate() {
                cumulative += n;
                let exemplar = extras.exemplar(i);
                encode_message(
                    3,
                    bucket_len(cumulative, upper_bound, exemplar),
                    buf,
                    |buf| {
                        // optional uint64 cumulative_count = 1;
                        encoding::encode_u64(1, cumulative, buf);
                        // optional double upper_bound      = 2;
                        encoding::encode_f64(2, upper_bound, buf);
                        // optional Exemplar exemplar       = 3;
                        if let Some(exemplar) = exemplar {
                            encode_exemplar(3, exemplar, buf);
                        }
                    },
                );
            }

            // optional google.protobuf.Timestamp created_timestamp = 15;
            if let Some(created) = extras.created {
                encode_timestamp(15, created, buf);
            }
        })
    }

    fn write_summary(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        quantiles: &[(f64, f64)],
        count: u64,
        sum: f64,
        extras: SampleExtras<'_>,
    ) -> Result<(), std::io::Error> {
        fn quantile_len(quantile: f64, value: f64) -> usize {
            encoding::encoded_len_f64(1, quantile) + encoding::encoded_len_f64(2, value)
        }

        let mut summary_len = encoding::encoded_len_u64(1, count)
            + encoding::encoded_len_f64(2, sum)
            + extras
                .created
                .map_or(0, |t| message_len(4, timestamp_len(t)));
        for &(quantile, value) in quantiles {
            summary_len += message_len(3, quantile_len(quantile, value));
        }

        // optional Summary   summary      = 4;
        self.write_metric(name, labels, 4, summary_len, |buf| {
            // optional uint64   sample_count = 1;
            encoding::encode_u64(1, count, buf);
            // optional double   sample_sum   = 2;
            encoding::encode_f64(2, sum, buf);

            // repeated Quantile quantile     = 3;
            for &(quantile, value) in quantiles {
                encode_message(3, quantile_len(quantile, value), buf, |buf| {
                    // optional double quantile = 1;
                    encoding::encode_f64(1, quantile, buf);
                    // optional double value    = 2;
                    encoding::encode_f64(2, value, buf);
                });
            }

            // optional google.protobuf.Timestamp created_timestamp = 4;
            if let Some(created) = extras.created {
                encode_timestamp(4, created, buf);
            }
        })
    }
}

struct LenVisitor {}
impl LabelVisitor for LenVisitor {
    type Output = usize;
    fn write_int(self, x: i64) -> usize {
        encoding::encoded_len_str(2, x.format_into(&mut core::fmt::NumBuffer::new()))
    }

    fn write_float(self, x: f64) -> usize {
        if x.is_infinite() {
            if x.is_sign_positive() {
                encoding::encoded_len_str(2, "+Inf")
            } else {
                encoding::encoded_len_str(2, "-Inf")
            }
        } else if x.is_nan() {
            encoding::encoded_len_str(2, "NaN")
        } else {
            encoding::encoded_len_str(2, zmij::Buffer::new().format(x))
        }
    }

    fn write_str(self, x: &str) -> usize {
        encoding::encoded_len_str(2, x)
    }
}

struct GroupLenVisitor {
    len: usize,
}
impl LabelGroupVisitor for GroupLenVisitor {
    type Output = ();
    fn write_value(&mut self, name: &LabelName, x: &impl LabelValue) {
        let mut label_pair_len = 0;

        label_pair_len += encoding::encoded_len_str(1, name.as_str());
        label_pair_len += x.visit(LenVisitor {});

        self.len += message_len(1, label_pair_len);
    }
}

struct Visitor<'a> {
    buf: &'a mut Vec<u8>,
}
impl LabelVisitor for Visitor<'_> {
    type Output = ();
    fn write_int(self, x: i64) {
        self.write_str(x.format_into(&mut core::fmt::NumBuffer::new()));
    }

    fn write_float(self, x: f64) {
        if x.is_infinite() {
            if x.is_sign_positive() {
                self.write_str("+Inf");
            } else {
                self.write_str("-Inf");
            }
        } else if x.is_nan() {
            self.write_str("NaN");
        } else {
            self.write_str(zmij::Buffer::new().format(x));
        }
    }

    fn write_str(self, x: &str) {
        // optional string value = 2;
        encoding::encode_str(2, x, self.buf);
    }
}

fn encode_message(
    tag: u32,
    len: usize,
    buf: &mut Vec<u8>,
    message: impl for<'a> FnOnce(&'a mut Vec<u8>),
) {
    encode_key(tag, LengthDelimited, buf);
    encode_varint(len as u64, buf);

    {
        let offset = buf.len();
        message(buf);
        debug_assert_eq!(buf.len() - offset, len);
    }
}

fn message_len(tag: u32, len: usize) -> usize {
    key_len(tag) + encoded_len_varint(len as u64) + len
}

struct GroupVisitor<'a> {
    buf: &'a mut Vec<u8>,
}
impl LabelGroupVisitor for GroupVisitor<'_> {
    type Output = ();
    fn write_value(&mut self, name: &LabelName, x: &impl LabelValue) {
        let mut label_pair_len = 0;
        label_pair_len += encoding::encoded_len_str(1, name.as_str());
        label_pair_len += x.visit(LenVisitor {});

        // repeated LabelPair label        = 1;
        encode_message(1, label_pair_len, self.buf, |buf| {
            // optional string name  = 1;
            encoding::encode_str(1, name.as_str(), buf);

            x.visit(Visitor { buf });
        });
    }
}

#[cfg(test)]
mod generated;

#[cfg(test)]
mod tests {
    use std::vec;

    use bytes::{BufMut, BytesMut};
    use measured::metric::MetricFamilyEncoding;
    use measured::metric::histogram::Thresholds;
    use measured::metric::name::{MetricName, Total};
    use measured::{CounterVec, GaugeVec, HistogramVec};
    use prost::Message;

    use crate::ProtoEncoder;
    use crate::generated::{
        Bucket, Counter, Gauge, Histogram, LabelPair, Metric, MetricFamily, MetricType, Quantile,
        Summary,
    };

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
    fn counters() {
        let requests = CounterVec::<RequestLabelSet>::new();

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

        let mut enc = ProtoEncoder::new(BytesMut::new().writer());

        let name = MetricName::from_str("http_request").with_suffix(Total);
        requests
            .collect_family_into(&name, Some("The total number of HTTP requests."), &mut enc)
            .unwrap();
        enc.flush().unwrap();
        let actual_msg = enc.into_inner().into_inner();

        let expected = MetricFamily {
            name: Some("http_request_total".to_string()),
            help: Some("The total number of HTTP requests.".to_string()),
            r#type: Some(MetricType::Counter as i32),
            metric: vec![
                Metric {
                    label: vec![
                        LabelPair {
                            name: Some("method".to_owned()),
                            value: Some("post".to_owned()),
                        },
                        LabelPair {
                            name: Some("code".to_owned()),
                            value: Some("200".to_owned()),
                        },
                    ],
                    gauge: None,
                    counter: Some(Counter {
                        value: Some(1027.0),
                        exemplar: None,
                        created_timestamp: None,
                    }),
                    summary: None,
                    untyped: None,
                    histogram: None,
                    timestamp_ms: None,
                },
                Metric {
                    label: vec![
                        LabelPair {
                            name: Some("method".to_owned()),
                            value: Some("get".to_owned()),
                        },
                        LabelPair {
                            name: Some("code".to_owned()),
                            value: Some("400".to_owned()),
                        },
                    ],
                    gauge: None,
                    counter: Some(Counter {
                        value: Some(3.0),
                        exemplar: None,
                        created_timestamp: None,
                    }),
                    summary: None,
                    untyped: None,
                    histogram: None,
                    timestamp_ms: None,
                },
            ],
            unit: None,
        };
        let mut expected_msg = BytesMut::new();
        expected.encode_length_delimited(&mut expected_msg).unwrap();

        assert_eq!(actual_msg, expected_msg);

        let actual = MetricFamily::decode_length_delimited(actual_msg).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn gauge() {
        let requests = GaugeVec::<RequestLabelSet>::new();

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

        let mut enc = ProtoEncoder::new(BytesMut::new().writer());

        let name = MetricName::from_str("http_request").with_suffix(Total);
        requests
            .collect_family_into(&name, Some("The total number of HTTP requests."), &mut enc)
            .unwrap();
        enc.flush().unwrap();
        let actual_msg = enc.into_inner().into_inner();

        let expected = MetricFamily {
            name: Some("http_request_total".to_string()),
            help: Some("The total number of HTTP requests.".to_string()),
            r#type: Some(MetricType::Gauge as i32),
            metric: vec![
                Metric {
                    label: vec![
                        LabelPair {
                            name: Some("method".to_owned()),
                            value: Some("post".to_owned()),
                        },
                        LabelPair {
                            name: Some("code".to_owned()),
                            value: Some("200".to_owned()),
                        },
                    ],
                    gauge: Some(Gauge {
                        value: Some(1027.0),
                    }),
                    counter: None,
                    summary: None,
                    untyped: None,
                    histogram: None,
                    timestamp_ms: None,
                },
                Metric {
                    label: vec![
                        LabelPair {
                            name: Some("method".to_owned()),
                            value: Some("get".to_owned()),
                        },
                        LabelPair {
                            name: Some("code".to_owned()),
                            value: Some("400".to_owned()),
                        },
                    ],
                    gauge: Some(Gauge { value: Some(3.0) }),
                    counter: None,
                    summary: None,
                    untyped: None,
                    histogram: None,
                    timestamp_ms: None,
                },
            ],
            unit: None,
        };
        let mut expected_msg = BytesMut::new();
        expected.encode_length_delimited(&mut expected_msg).unwrap();

        assert_eq!(actual_msg, expected_msg);

        let actual = MetricFamily::decode_length_delimited(actual_msg).unwrap();
        assert_eq!(actual, expected);
    }

    fn decode_all(mut msg: bytes::Bytes) -> Vec<MetricFamily> {
        let mut families = vec![];
        while !msg.is_empty() {
            families.push(MetricFamily::decode_length_delimited(&mut msg).unwrap());
        }
        families
    }

    fn write_family<W: std::io::Write>(
        enc: &mut ProtoEncoder<W>,
        name: &'static str,
        family: &CounterVec<RequestLabelSet>,
    ) -> std::io::Result<()> {
        let name = MetricName::from_str(name).with_suffix(Total);
        family.collect_family_into(&name, Some("help"), enc)
    }

    #[test]
    fn empty_families_are_dropped() {
        let empty = CounterVec::<RequestLabelSet>::new();
        let full = CounterVec::<RequestLabelSet>::new();
        full.inc(RequestLabels {
            method: Method::Get,
            code: StatusCode::Ok,
        });

        let mut enc = ProtoEncoder::new(BytesMut::new().writer());
        write_family(&mut enc, "a", &empty).unwrap();
        write_family(&mut enc, "b", &full).unwrap();
        write_family(&mut enc, "c", &empty).unwrap();
        write_family(&mut enc, "d", &full).unwrap();
        write_family(&mut enc, "e", &empty).unwrap();
        enc.flush().unwrap();

        let families = decode_all(enc.into_inner().into_inner().freeze());
        let names: Vec<_> = families.iter().map(|f| f.name()).collect();
        assert_eq!(names, ["b_total", "d_total"]);
        assert!(families.iter().all(|f| f.metric.len() == 1));
    }

    #[test]
    fn type_without_help() {
        let empty = CounterVec::<RequestLabelSet>::new();
        let full = CounterVec::<RequestLabelSet>::new();
        full.inc(RequestLabels {
            method: Method::Get,
            code: StatusCode::Ok,
        });

        let mut enc = ProtoEncoder::new(BytesMut::new().writer());
        let a = MetricName::from_str("a").with_suffix(Total);
        let b = MetricName::from_str("b").with_suffix(Total);
        empty.collect_family_into(&a, None, &mut enc).unwrap();
        full.collect_family_into(&b, None, &mut enc).unwrap();
        enc.flush().unwrap();

        let families = decode_all(enc.into_inner().into_inner().freeze());
        assert_eq!(families.len(), 1);
        assert_eq!(families[0].name(), "b_total");
        assert_eq!(families[0].help, None);
        assert_eq!(families[0].r#type, Some(MetricType::Counter as i32));
    }

    struct FailingWriter {
        fail: bool,
        out: Vec<u8>,
    }

    impl std::io::Write for FailingWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            if self.fail {
                return Err(std::io::ErrorKind::BrokenPipe.into());
            }
            self.out.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn reuse_after_write_error() {
        let full = CounterVec::<RequestLabelSet>::new();
        full.inc(RequestLabels {
            method: Method::Get,
            code: StatusCode::Ok,
        });

        let mut enc = ProtoEncoder::new(FailingWriter {
            fail: true,
            out: vec![],
        });
        write_family(&mut enc, "a", &full).unwrap();
        enc.flush().unwrap_err();

        enc.writer_mut().fail = false;
        write_family(&mut enc, "b", &full).unwrap();
        enc.flush().unwrap();

        let families = decode_all(enc.into_inner().out.into());
        assert_eq!(families.len(), 1);
        assert_eq!(families[0].name(), "b_total");
        assert_eq!(families[0].metric.len(), 1);
    }

    #[test]
    fn histogram() {
        let histograms =
            HistogramVec::<RequestLabelSet, 2>::with_metadata(Thresholds::with_buckets([0.1, 1.0]));
        let labels = RequestLabels {
            method: Method::Get,
            code: StatusCode::Ok,
        };
        histograms.observe(labels, 0.05);
        histograms.observe(labels, 0.5);
        histograms.observe(labels, 0.5);
        histograms.observe(labels, 5.0);

        let mut enc = ProtoEncoder::new(BytesMut::new().writer());
        let name = MetricName::from_str("latency_seconds");
        histograms
            .collect_family_into(name, Some("request latency"), &mut enc)
            .unwrap();
        enc.flush().unwrap();

        let families = decode_all(enc.into_inner().into_inner().freeze());
        assert_eq!(
            families,
            [MetricFamily {
                name: Some("latency_seconds".to_owned()),
                help: Some("request latency".to_owned()),
                r#type: Some(MetricType::Histogram as i32),
                metric: vec![Metric {
                    label: vec![
                        LabelPair {
                            name: Some("method".to_owned()),
                            value: Some("get".to_owned()),
                        },
                        LabelPair {
                            name: Some("code".to_owned()),
                            value: Some("200".to_owned()),
                        },
                    ],
                    histogram: Some(Histogram {
                        sample_count: Some(4),
                        sample_sum: Some(6.05),
                        bucket: vec![
                            Bucket {
                                cumulative_count: Some(1),
                                upper_bound: Some(0.1),
                                ..Default::default()
                            },
                            Bucket {
                                cumulative_count: Some(3),
                                upper_bound: Some(1.0),
                                ..Default::default()
                            },
                        ],
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                unit: None,
            }]
        );
    }

    #[test]
    fn summary() {
        use measured::metric::group::{Encoding, MetricKind, SampleExtras};

        let mut enc = ProtoEncoder::new(BytesMut::new().writer());
        let name = MetricName::from_str("latency");
        enc.write_family(name, None, MetricKind::Summary).unwrap();
        enc.write_summary(
            name,
            measured::label::NoLabels,
            &[(0.5, 1.0), (0.99, 3.0)],
            10,
            12.5,
            SampleExtras::NONE,
        )
        .unwrap();
        enc.write_family(
            MetricName::from_str("flag"),
            None,
            MetricKind::Custom("stateset"),
        )
        .unwrap();
        enc.write_gauge(MetricName::from_str("flag"), measured::label::NoLabels, 1)
            .unwrap();
        enc.flush().unwrap();

        let families = decode_all(enc.into_inner().into_inner().freeze());
        assert_eq!(families.len(), 2);
        assert_eq!(families[0].r#type, Some(MetricType::Summary as i32));
        assert_eq!(
            families[0].metric[0].summary,
            Some(Summary {
                sample_count: Some(10),
                sample_sum: Some(12.5),
                quantile: vec![
                    Quantile {
                        quantile: Some(0.5),
                        value: Some(1.0),
                    },
                    Quantile {
                        quantile: Some(0.99),
                        value: Some(3.0),
                    },
                ],
                ..Default::default()
            })
        );
        assert_eq!(families[1].r#type, Some(MetricType::Untyped as i32));
    }

    #[test]
    fn untyped_samples_use_untyped_payloads() {
        use measured::label::NoLabels;
        use measured::metric::group::{Encoding, MetricKind, SampleExtras};

        let mut enc = ProtoEncoder::new(Vec::new());
        let name = MetricName::from_str("value");
        // Also cover the implicit untyped family before any header is written.
        enc.write_counter(name, NoLabels, 7, SampleExtras::NONE)
            .unwrap();
        enc.flush().unwrap();
        for kind in [MetricKind::Untyped, MetricKind::Custom("flag")] {
            enc.write_family(name, None, kind).unwrap();
            enc.write_gauge(name, NoLabels, -2).unwrap();
            enc.write_float_gauge(name, NoLabels, 1.5).unwrap();
            let exemplar = measured::metric::exemplar::Exemplar::new();
            let extras = SampleExtras::NONE
                .with_created(Some(std::time::UNIX_EPOCH))
                .with_exemplar(Some(&exemplar));
            enc.write_counter(name, NoLabels, 3, extras).unwrap();
            enc.write_float_counter(name, NoLabels, 4.5, extras)
                .unwrap();
        }
        enc.write_family(name, None, MetricKind::Counter).unwrap();
        enc.write_counter(name, NoLabels, 9, SampleExtras::NONE)
            .unwrap();
        enc.flush().unwrap();
        // Flushing a typed family must reset the implicit fallback too.
        enc.write_float_gauge(name, NoLabels, 2.5).unwrap();
        enc.flush().unwrap();

        let families = decode_all(enc.into_inner().into());
        assert_eq!(families.len(), 5);
        for (family, values) in [
            (&families[0], &[7.0][..]),
            (&families[1], &[-2.0, 1.5, 3.0, 4.5][..]),
            (&families[2], &[-2.0, 1.5, 3.0, 4.5][..]),
            (&families[4], &[2.5][..]),
        ] {
            assert_eq!(family.r#type, Some(MetricType::Untyped as i32));
            assert_eq!(family.metric.len(), values.len());
            for (metric, value) in family.metric.iter().zip(values) {
                assert_eq!(metric.untyped.as_ref().unwrap().value, Some(*value));
                assert!(metric.gauge.is_none());
                assert!(metric.counter.is_none());
            }
        }
        assert_eq!(
            families[3].metric[0].counter.as_ref().unwrap().value,
            Some(9.0)
        );
        assert!(families[3].metric[0].untyped.is_none());
    }

    #[test]
    fn histogram_retains_infinite_bucket_exemplar() {
        use measured::label::LabelName;
        use measured::metric::exemplar::Exemplar;

        // Include a histogram whose only bucket is +Inf.
        for bounds in [vec![], vec![1.0]] {
            let mut enc = ProtoEncoder::new(Vec::new());
            use measured::metric::group::{Encoding, HistogramExtras, MetricKind};
            let name = MetricName::from_str("latency");
            enc.write_family(name, None, MetricKind::Histogram).unwrap();
            let exemplar = Exemplar::new()
                .with_label(LabelName::from_str("trace_id"), "overflow")
                .with_value(5.0)
                .with_timestamp(std::time::UNIX_EPOCH);
            let mut exemplars = vec![None; bounds.len()];
            exemplars.push(Some(exemplar));
            enc.write_histogram(
                name,
                measured::label::NoLabels,
                &bounds,
                &vec![2; bounds.len()],
                1,
                5.0,
                HistogramExtras::NONE.with_exemplars(&exemplars),
            )
            .unwrap();
            enc.flush().unwrap();
            let families = decode_all(enc.into_inner().into());
            let histogram = families[0].metric[0].histogram.as_ref().unwrap();
            assert_eq!(histogram.bucket.len(), bounds.len() + 1);
            let bucket = histogram.bucket.last().unwrap();
            assert_eq!(bucket.upper_bound, Some(f64::INFINITY));
            assert_eq!(bucket.cumulative_count, histogram.sample_count);
            let exemplar = bucket.exemplar.as_ref().unwrap();
            assert_eq!(exemplar.value, Some(5.0));
            assert_eq!(exemplar.label[0].value.as_deref(), Some("overflow"));
            assert_eq!(
                exemplar.timestamp,
                Some(prost_types::Timestamp {
                    seconds: 0,
                    nanos: 0
                })
            );
        }
    }

    #[test]
    fn new_kinds() {
        use measured::FixedCardinalityLabel;
        use measured::label::NoLabels;
        use measured::metric::group::{Encoding, MetricKind, SampleExtras};

        #[derive(Clone, Copy, PartialEq, Debug, FixedCardinalityLabel)]
        enum Status {
            Starting,
            Ready,
        }

        let mut enc = ProtoEncoder::new(BytesMut::new().writer());

        let cpu = MetricName::from_str("cpu_seconds_total");
        enc.write_family(cpu, None, MetricKind::Counter).unwrap();
        enc.write_float_counter(cpu, NoLabels, 1.5, SampleExtras::NONE)
            .unwrap();

        let ages = MetricName::from_str("queue_age_seconds");
        enc.write_family(ages, None, MetricKind::GaugeHistogram)
            .unwrap();
        enc.write_gauge_histogram(ages, NoLabels, &[1.0], &[1], 1, 2.5)
            .unwrap();

        let status = MetricName::from_str("status");
        enc.write_family(status, None, MetricKind::StateSet)
            .unwrap();
        enc.write_state_set(
            status,
            NoLabels,
            [(Status::Starting, false), (Status::Ready, true)],
        )
        .unwrap();

        let info = MetricName::from_str("build_info");
        enc.write_family(info, None, MetricKind::Info).unwrap();
        enc.write_gauge(info, NoLabels, 1).unwrap();
        enc.flush().unwrap();

        let families = decode_all(enc.into_inner().into_inner().freeze());
        let types: Vec<_> = families.iter().map(|f| f.r#type).collect();
        assert_eq!(
            types,
            [
                Some(MetricType::Counter as i32),
                Some(MetricType::GaugeHistogram as i32),
                Some(MetricType::Gauge as i32),
                Some(MetricType::Gauge as i32),
            ]
        );

        assert_eq!(
            families[0].metric[0].counter.as_ref().unwrap().value,
            Some(1.5)
        );

        let histogram = families[1].metric[0].histogram.as_ref().unwrap();
        assert_eq!(histogram.sample_count, Some(2));
        assert_eq!(histogram.sample_sum, Some(2.5));
        assert_eq!(histogram.bucket[0].cumulative_count, Some(1));

        let states: Vec<_> = families[2]
            .metric
            .iter()
            .map(|m| (m.label[0].clone(), m.gauge.as_ref().unwrap().value))
            .collect();
        assert_eq!(
            states,
            [
                (
                    LabelPair {
                        name: Some("status".to_owned()),
                        value: Some("starting".to_owned()),
                    },
                    Some(0.0)
                ),
                (
                    LabelPair {
                        name: Some("status".to_owned()),
                        value: Some("ready".to_owned()),
                    },
                    Some(1.0)
                ),
            ]
        );
    }

    #[test]
    fn exemplars_and_created() {
        use std::time::{Duration, UNIX_EPOCH};

        use measured::label::LabelName;
        use measured::metric::MetricFamilyEncoding;
        use measured::metric::exemplar::Exemplar;
        use measured::metric::histogram::Thresholds;
        use measured::{CreatedCounter, ExemplarCounter, ExemplarHistogram};

        const TRACE_ID: &LabelName = LabelName::from_str("trace_id");
        let at = UNIX_EPOCH + Duration::from_millis(1_700_000_000_500);

        let requests = ExemplarCounter::new();
        requests.inc_with_exemplar(
            3,
            Exemplar::new()
                .with_label(TRACE_ID, "abc")
                .with_timestamp(at),
        );
        let latency = ExemplarHistogram::with_metadata(Thresholds::with_buckets([1.0]));
        latency.observe_with_exemplar(0.5, Exemplar::new().with_label(TRACE_ID, "def"));
        let created = CreatedCounter::new();
        created.inc();

        let mut enc = ProtoEncoder::new(BytesMut::new().writer());
        requests
            .collect_family_into(MetricName::from_str("requests_total"), None, &mut enc)
            .unwrap();
        latency
            .collect_family_into(MetricName::from_str("latency_seconds"), None, &mut enc)
            .unwrap();
        created
            .collect_family_into(MetricName::from_str("created_total"), None, &mut enc)
            .unwrap();
        enc.flush().unwrap();

        let families = decode_all(enc.into_inner().into_inner().freeze());

        let counter = families[0].metric[0].counter.clone().unwrap();
        assert_eq!(counter.value, Some(3.0));
        let exemplar = counter.exemplar.unwrap();
        assert_eq!(exemplar.value, Some(3.0));
        assert_eq!(exemplar.label[0].value.as_deref(), Some("abc"));
        assert_eq!(
            exemplar.timestamp,
            Some(prost_types::Timestamp {
                seconds: 1_700_000_000,
                nanos: 500_000_000,
            })
        );
        assert_eq!(counter.created_timestamp, None);

        let bucket = &families[1].metric[0].histogram.as_ref().unwrap().bucket[0];
        let exemplar = bucket.exemplar.as_ref().unwrap();
        assert_eq!(exemplar.value, Some(0.5));
        assert_eq!(exemplar.label[0].value.as_deref(), Some("def"));

        let counter = families[2].metric[0].counter.clone().unwrap();
        let created_at = created.created().duration_since(UNIX_EPOCH).unwrap();
        assert_eq!(
            counter.created_timestamp,
            Some(prost_types::Timestamp {
                seconds: created_at.as_secs() as i64,
                nanos: created_at.subsec_nanos() as i32,
            })
        );
    }
}
