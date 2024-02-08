//! Prometheus Text based exporter

use bytes::{BufMut, Bytes, BytesMut};
use memchr::memchr3_iter;

use crate::{
    label::{LabelGroup, LabelGroupVisitor, LabelName, LabelValue, LabelVisitor},
    metric::{
        counter::CounterState,
        gauge::GaugeState,
        group::Encoding,
        histogram::{HistogramState, Thresholds},
        name::{Bucket, Count, MetricNameEncoder, Sum},
        MetricEncoding,
    },
};

/// The prometheus text encoder helper
pub struct TextEncoder {
    state: State,
    b: BytesMut,
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

/// Values that prometheus supports in the text format
#[derive(Clone, Copy, Debug)]
pub enum MetricValue {
    Int(i64),
    Float(f64),
}

impl Default for TextEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Encoding for TextEncoder {
    /// Write the help line for a metric
    fn write_help(&mut self, name: impl MetricNameEncoder, help: &str) {
        if self.state == State::Metrics {
            self.write_line();
        }
        self.state = State::Info;

        self.b.extend_from_slice(b"# HELP ");
        name.encode_text(&mut self.b);
        self.b.extend_from_slice(b" ");
        self.b.extend_from_slice(help.as_bytes());
        self.b.extend_from_slice(b"\n");
    }
}

impl TextEncoder {
    /// Create a new text encoder.
    ///
    /// This should ideally be cached and re-used between collections to reduce re-allocating
    pub fn new() -> Self {
        Self {
            state: State::Info,
            b: BytesMut::new(),
        }
    }

    /// Finish the text encoding and extract the bytes to send in a HTTP response.
    pub fn finish(&mut self) -> Bytes {
        self.state = State::Info;
        self.b.split().freeze()
    }

    fn write_line(&mut self) {
        self.b.put_u8(b'\n');
    }

    /// Write the type line for a metric
    pub fn write_type(&mut self, name: &impl MetricNameEncoder, typ: MetricType) {
        if self.state == State::Metrics {
            self.write_line();
        }
        self.state = State::Info;

        self.b.extend_from_slice(b"# TYPE ");
        name.encode_text(&mut self.b);
        match typ {
            MetricType::Counter => self.b.extend_from_slice(b" counter\n"),
            MetricType::Histogram => self.b.extend_from_slice(b" histogram\n"),
            MetricType::Gauge => self.b.extend_from_slice(b" gauge\n"),
            MetricType::Summary => self.b.extend_from_slice(b" summary\n"),
            MetricType::Untyped => self.b.extend_from_slice(b" untyped\n"),
        }
    }

    /// Write the metric data
    pub fn write_metric<L: LabelGroup>(
        &mut self,
        name: impl MetricNameEncoder,
        labels: L,
        value: MetricValue,
    ) {
        struct Visitor<'a> {
            b: &'a mut BytesMut,
        }
        impl LabelVisitor for Visitor<'_> {
            type Output = ();
            fn write_int(self, x: u64) {
                self.write_str(itoa::Buffer::new().format(x))
            }

            fn write_float(self, x: f64) {
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

            fn write_str(self, x: &str) {
                self.b.extend_from_slice(b"=\"");
                write_label_str_value(x, &mut *self.b);
                self.b.extend_from_slice(b"\"");
            }
        }
        struct GroupVisitor<'a> {
            first: bool,
            b: &'a mut BytesMut,
        }
        impl LabelGroupVisitor for GroupVisitor<'_> {
            fn write_value(&mut self, name: &LabelName, x: &impl LabelValue) {
                if self.first {
                    self.first = false;
                    self.b.extend_from_slice(b"{");
                } else {
                    self.b.extend_from_slice(b",");
                }
                self.b.extend_from_slice(name.as_str().as_bytes());
                x.visit(Visitor { b: self.b });
            }
        }

        self.state = State::Metrics;
        name.encode_text(&mut self.b);

        let mut visitor = GroupVisitor {
            first: true,
            b: &mut self.b,
        };
        labels.visit_values(&mut visitor);
        if !visitor.first {
            self.b.extend_from_slice(b"}");
        }
        self.b.extend_from_slice(b" ");
        match value {
            MetricValue::Int(x) => self
                .b
                .extend_from_slice(itoa::Buffer::new().format(x).as_bytes()),
            MetricValue::Float(x) => self
                .b
                .extend_from_slice(ryu::Buffer::new().format(x).as_bytes()),
        }
        self.b.extend_from_slice(b"\n");
    }
}

impl<const N: usize> MetricEncoding<TextEncoder> for HistogramState<N> {
    fn write_type(name: impl MetricNameEncoder, enc: &mut TextEncoder) {
        enc.write_type(&name, MetricType::Histogram);
    }
    fn collect_into(
        &self,
        metadata: &Thresholds<N>,
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut TextEncoder,
    ) {
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
                const LE: &LabelName = LabelName::from_static("le");
                v.write_value(LE, &F64(self.le));
            }
        }

        let state = *self.inner.lock();
        let mut val = 0;

        for i in 0..N {
            let le = metadata.get()[i];
            val += state.buckets[i];
            enc.write_metric(
                &name.by_ref().with_suffix(Bucket),
                labels.by_ref().compose_with(HistogramLabelLe { le }),
                MetricValue::Int(val as i64),
            );
        }
        let count = val + state.inf;
        enc.write_metric(
            &name.by_ref().with_suffix(Bucket),
            labels
                .by_ref()
                .compose_with(HistogramLabelLe { le: f64::INFINITY }),
            MetricValue::Int(count as i64),
        );
        enc.write_metric(
            &name.by_ref().with_suffix(Sum),
            labels.by_ref(),
            MetricValue::Float(state.sum),
        );
        enc.write_metric(
            &name.by_ref().with_suffix(Count),
            labels,
            MetricValue::Int(count as i64),
        );
    }
}

impl MetricEncoding<TextEncoder> for CounterState {
    fn write_type(name: impl MetricNameEncoder, enc: &mut TextEncoder) {
        enc.write_type(&name, MetricType::Counter);
    }
    fn collect_into(
        &self,
        _m: &(),
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut TextEncoder,
    ) {
        enc.write_metric(
            &name,
            labels,
            MetricValue::Int(self.count.load(core::sync::atomic::Ordering::Relaxed) as i64),
        );
    }
}

impl MetricEncoding<TextEncoder> for GaugeState {
    fn write_type(name: impl MetricNameEncoder, enc: &mut TextEncoder) {
        enc.write_type(&name, MetricType::Gauge);
    }
    fn collect_into(
        &self,
        _m: &(),
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut TextEncoder,
    ) {
        enc.write_metric(
            &name,
            labels,
            MetricValue::Int(self.count.load(core::sync::atomic::Ordering::Relaxed) as i64),
        );
    }
}

pub(crate) fn write_label_str_value(s: &str, b: &mut BytesMut) {
    let mut i = 0;
    for j in memchr3_iter(b'\\', b'"', b'\n', s.as_bytes()) {
        b.extend_from_slice(&s.as_bytes()[i..j]);
        match s.as_bytes()[j] {
            b'\\' => b.extend_from_slice(b"\\\\"),
            b'"' => b.extend_from_slice(b"\\\""),
            b'\n' => b.extend_from_slice(b"\\n"),
            _ => unreachable!(),
        }
        i = j + 1;
    }
    b.extend_from_slice(&s.as_bytes()[i..]);
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

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

    use super::{write_label_str_value, TextEncoder};

    #[test]
    fn write_encoded_str() {
        let mut b = BytesMut::new();
        write_label_str_value(
            r#"Hello \ "World"
This is on a new line"#,
            &mut b,
        );

        assert_eq!(b, r#"Hello \\ \"World\"\nThis is on a new line"#);
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
        let requests = CounterVec::new_sparse(RequestLabelSet {
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

        let mut encoder = TextEncoder::default();

        let name = MetricName::from_static("http_request").with_suffix(Total);
        encoder.write_help(&name, "The total number of HTTP requests.");
        requests.collect_into(name, &mut encoder);

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
        let histogram = Histogram::new_metric(thresholds);

        histogram.get_metric().observe(0.7);
        histogram.get_metric().observe(2.5);
        histogram.get_metric().observe(1.2);
        histogram.get_metric().observe(8.0);

        let mut encoder = TextEncoder::default();

        let name = MetricName::from_static("http_request_duration_seconds");
        encoder.write_help(name, "A histogram of the request duration.");
        histogram.collect_into(name, &mut encoder);

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
