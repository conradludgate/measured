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

    pub fn write_help(&mut self, help: &str, name: &impl MetricName) {
        self.b.extend_from_slice(b"# HELP ");
        name.encode_text(&mut self.b);
        self.b.extend_from_slice(b" ");
        self.b.extend_from_slice(help.as_bytes());
        self.b.extend_from_slice(b"\n");
    }

    pub fn write_type(&mut self, typ: MetricType, name: &impl MetricName) {
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
                self.write_str(ryu::Buffer::new().format(x))
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

struct Total;
struct Created;
struct Count;
struct Sum;
struct Bucket;

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

    use crate::label::{FixedCardinalityLabel, LabelGroup, LabelGroupSet, LabelValue};

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

    #[derive(Clone, Copy, PartialEq, Debug)]
    // #[derive(LabelGroup)] #[label_set(RequestLabelSet)]
    struct RequestLabels {
        method: Method,
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
        let mut encoder = TextEncoder { b: BytesMut::new() };
        let name = "http_request".with_suffix(Total);
        let metrics = [
            (
                RequestLabels {
                    method: Method::Post,
                    code: StatusCode::Ok,
                },
                1027,
            ),
            (
                RequestLabels {
                    method: Method::Get,
                    code: StatusCode::BadRequest,
                },
                3,
            ),
        ];

        encoder.write_help("The total number of HTTP requests.", &name);
        encoder.write_type(super::MetricType::Counter, &name);
        for (labels, count) in metrics {
            encoder.write_metric(&name, labels, super::MetricValue::Int(count));
        }
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

    // TODO: macro
    struct RequestLabelSet;
    impl LabelGroupSet for RequestLabelSet {
        type Group<'a> = RequestLabels;

        fn cardinality(&self) -> Option<usize> {
            Some(1usize)
                .and_then(|x| x.checked_mul(Method::cardinality()))
                .and_then(|x| x.checked_mul(StatusCode::cardinality()))
        }

        type Unique = usize;

        #[allow(unused_assignments)]
        fn encode<'a>(&'a self, value: Self::Group<'a>) -> Option<Self::Unique> {
            let mut mul = 1;
            let mut index = 0;

            index += value.method.encode() * mul;
            mul *= Method::cardinality();

            index += value.code.encode() * mul;
            mul *= StatusCode::cardinality();

            Some(index)
        }

        fn decode(&self, value: Self::Unique) -> Self::Group<'_> {
            let index = value;
            let (index, index1) = (index / Method::cardinality(), index % Method::cardinality());
            let method = Method::decode(index1);
            let (index, index1) = (
                index / StatusCode::cardinality(),
                index % StatusCode::cardinality(),
            );
            let code = StatusCode::decode(index1);
            debug_assert_eq!(index, 0);
            Self::Group { method, code }
        }

        fn encode_dense(&self, value: Self::Unique) -> Option<usize> {
            Some(value)
        }
    }

    impl LabelGroup for RequestLabels {
        fn label_names() -> impl IntoIterator<Item = &'static str> {
            ["method", "code"]
        }

        fn label_values(self, v: &mut impl super::LabelVisitor) {
            self.method.visit(v);
            self.code.visit(v);
        }
    }

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