use bytes::BytesMut;

use crate::text::write_str;

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
