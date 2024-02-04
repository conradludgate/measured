use bytes::{Bytes, BytesMut};
use memchr::memchr3_iter;

use crate::label::{LabelGroup, LabelVisitor};

pub struct TextEncoder {
    b: BytesMut,
}

pub enum MetricType {
    Counter,
    Histogram,
    Gauge,
    Summary,
    Untyped,
}

pub enum MetricValue {
    Int(i64),
    Float(f64),
}

impl TextEncoder {
    pub fn finish(&mut self) -> Bytes {
        self.b.split().freeze()
    }

    pub fn write_help(&mut self, name: &impl MetricName, help: &str) {
        self.b.extend_from_slice(b"# HELP ");
        name.encode_text(&mut self.b);
        self.b.extend_from_slice(b" ");
        self.b.extend_from_slice(help.as_bytes());
        self.b.extend_from_slice(b"\n");
    }

    pub fn write_type(&mut self, name: &impl MetricName, typ: MetricType) {
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

    pub fn write_metric<L: LabelGroup>(
        &mut self,
        name: &impl MetricName,
        labels: L,
        value: MetricValue,
    ) {
        name.encode_text(&mut self.b);
        struct Visitor<'a, I> {
            first: bool,
            iter: I,
            b: &'a mut BytesMut,
        }
        impl<I: Iterator<Item = &'static str>> LabelVisitor for Visitor<'_, I> {
            fn write_int(&mut self, x: u64) {
                self.write_str(itoa::Buffer::new().format(x))
            }

            fn write_float(&mut self, x: f64) {
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

            fn write_str(&mut self, x: &str) {
                if self.first {
                    self.first = false;
                    self.b.extend_from_slice(b"{");
                } else {
                    self.b.extend_from_slice(b",");
                }
                let label_name = self.iter.next().expect("missing label name");
                write_str(label_name, &mut *self.b);
                self.b.extend_from_slice(b"=\"");
                write_str(x, &mut *self.b);
                self.b.extend_from_slice(b"\"");
            }
        }

        let mut visitor = Visitor {
            first: true,
            iter: L::label_names().into_iter(),
            b: &mut self.b,
        };
        labels.label_values(&mut visitor);
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

pub trait MetricName {
    fn encode_text(&self, b: &mut BytesMut);

    fn in_namespace(self, ns: &'static str) -> WithNamespace<Self>
    where
        Self: Sized,
    {
        WithNamespace {
            namespace: ns,
            metric_name: self,
        }
    }

    fn with_suffix<S: Suffix>(self, suffix: S) -> WithSuffix<S, Self>
    where
        Self: Sized,
    {
        WithSuffix {
            suffix,
            metric_name: self,
        }
    }

    fn by_ref(&self) -> &Self {
        self
    }
}

fn write_str(s: &str, b: &mut BytesMut) {
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

pub trait Suffix {
    fn encode_text(&self, b: &mut BytesMut);
}

impl MetricName for str {
    fn encode_text(&self, b: &mut BytesMut) {
        write_str(self, b);
    }
}

impl<T: MetricName + ?Sized> MetricName for &T {
    fn encode_text(&self, b: &mut BytesMut) {
        T::encode_text(self, b)
    }
}

pub struct WithNamespace<T: ?Sized> {
    namespace: &'static str,
    metric_name: T,
}

impl<T: MetricName + ?Sized> MetricName for WithNamespace<T> {
    fn encode_text(&self, b: &mut BytesMut) {
        write_str(self.namespace, b);
        b.extend_from_slice(b"_");
        self.metric_name.encode_text(b)
    }
}

pub struct WithSuffix<S, T: ?Sized> {
    suffix: S,
    metric_name: T,
}

impl<S: Suffix, T: MetricName + ?Sized> MetricName for WithSuffix<S, T> {
    fn encode_text(&self, b: &mut BytesMut) {
        self.metric_name.encode_text(b);
        self.suffix.encode_text(b);
    }
}

pub struct Total;
pub struct Created;
pub struct Count;
pub struct Sum;
pub struct Bucket;

impl Suffix for Total {
    fn encode_text(&self, b: &mut BytesMut) {
        b.extend_from_slice(b"_total");
    }
}

impl Suffix for Created {
    fn encode_text(&self, b: &mut BytesMut) {
        b.extend_from_slice(b"_created");
    }
}
impl Suffix for Count {
    fn encode_text(&self, b: &mut BytesMut) {
        b.extend_from_slice(b"_count");
    }
}
impl Suffix for Sum {
    fn encode_text(&self, b: &mut BytesMut) {
        b.extend_from_slice(b"_sum");
    }
}
impl Suffix for Bucket {
    fn encode_text(&self, b: &mut BytesMut) {
        b.extend_from_slice(b"_bucket");
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use crate::{
        label::{FixedCardinalityLabel, LabelValue},
        CounterVec, Histogram, Thresholds,
    };

    use super::{write_str, MetricName, TextEncoder, Total};

    #[test]
    fn write_encoded_str() {
        let mut b = BytesMut::new();
        write_str(
            r#"Hello \ "World"
This is on a new line"#,
            &mut b,
        );

        assert_eq!(&b, &br#"Hello \\ \"World\"\nThis is on a new line"#[..]);
    }

    #[derive(Clone, Copy, PartialEq, Debug, measured_derive::LabelGroup)]
    #[label(crate = crate, set = RequestLabelSet)]
    struct RequestLabels {
        #[label(fixed)]
        method: Method,
        #[label(fixed)]
        code: StatusCode,
    }

    #[derive(Clone, Copy, PartialEq, Debug)]
    enum Method {
        Post,
        Get,
    }

    #[derive(Clone, Copy, PartialEq, Debug)]
    enum StatusCode {
        Ok,
        BadRequest,
    }

    #[test]
    fn text_encoding() {
        let counters = CounterVec::new_sparse_counter_vec(RequestLabelSet {});

        let post_ok = RequestLabels {
            method: Method::Post,
            code: StatusCode::Ok,
        };
        counters.get_metric(counters.with_labels(post_ok).unwrap(), |c| c.inc_by(1027));
        let get_bad = RequestLabels {
            method: Method::Get,
            code: StatusCode::BadRequest,
        };
        counters.get_metric(counters.with_labels(get_bad).unwrap(), |c| c.inc_by(3));

        let mut encoder = TextEncoder { b: BytesMut::new() };

        let name = "http_request".with_suffix(Total);
        encoder.write_help(&name, "The total number of HTTP requests.");
        counters.collect_into(name, &mut encoder);

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

        let mut encoder = TextEncoder { b: BytesMut::new() };

        let name = "http_request_duration_seconds";
        encoder.write_help(&name, "A histogram of the request duration.");
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
http_request_duration_seconds_bucket{le="+Inf"} 4
http_request_duration_seconds_sum 12.4
http_request_duration_seconds_count 4
"#
        );
    }

    // TODO: macro

    impl FixedCardinalityLabel for Method {
        fn cardinality() -> usize {
            2
        }

        fn encode(&self) -> usize {
            match self {
                Method::Post => 0,
                Method::Get => 1,
            }
        }

        fn decode(value: usize) -> Self {
            match value {
                0 => Method::Post,
                1 => Method::Get,
                _ => unreachable!(),
            }
        }
    }
    impl LabelValue for Method {
        fn visit(&self, v: &mut impl crate::label::LabelVisitor) {
            match self {
                Method::Post => v.write_str("post"),
                Method::Get => v.write_str("get"),
            }
        }
    }

    impl FixedCardinalityLabel for StatusCode {
        fn cardinality() -> usize {
            2
        }

        fn encode(&self) -> usize {
            match self {
                StatusCode::Ok => 0,
                StatusCode::BadRequest => 1,
            }
        }

        fn decode(value: usize) -> Self {
            match value {
                0 => StatusCode::Ok,
                1 => StatusCode::BadRequest,
                _ => unreachable!(),
            }
        }
    }
    impl LabelValue for StatusCode {
        fn visit(&self, v: &mut impl crate::label::LabelVisitor) {
            match self {
                StatusCode::Ok => v.write_int(200),
                StatusCode::BadRequest => v.write_int(400),
            }
        }
    }
}
