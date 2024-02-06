use bytes::BytesMut;

/// `MetricName` represents a type that can be encoded into the name of a metric when collected.
pub trait MetricNameEncoder {
    /// Encoded this name into the given bytes buffer according to the Prometheus metric name encoding specification.
    ///
    /// See <https://prometheus.io/docs/concepts/data_model/#metric-names-and-labels>
    fn encode_text(&self, b: &mut BytesMut);

    /// Adds a semantic suffix to this metric name.
    fn with_suffix<S: Suffix>(self, suffix: S) -> WithSuffix<S, Self>
    where
        Self: Sized,
    {
        WithSuffix {
            suffix,
            metric_name: self,
        }
    }

    /// Get a reference to this metric name
    fn by_ref(&self) -> &Self {
        self
    }
}

/// Error returned by [`CheckedMetricName::try_from`]
#[derive(Debug)]
pub enum InvalidMetricName {
    InvalidChars,
    Empty,
    StartsWithNumber,
}

impl core::fmt::Display for InvalidMetricName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InvalidMetricName::InvalidChars => {
                f.write_str("metric name contained invalid characters")
            }
            InvalidMetricName::Empty => f.write_str("metric name was empty"),
            InvalidMetricName::StartsWithNumber => f.write_str("metric name started with a number"),
        }
    }
}

impl std::error::Error for InvalidMetricName {}

/// Represents a string-based [`MetricName`]
pub struct MetricName(str);

pub(crate) const fn assert_metric_name(name: &'static str) {
    // > Metric names may contain ASCII letters, digits, underscores, and colons. It must match the regex [a-zA-Z_:][a-zA-Z0-9_:]*
    if name.is_empty() {
        panic!("string should not be empty")
    }

    let mut i = 0;
    while i < name.len() {
        match name.as_bytes()[i] {
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'_' | b':' => {}
            _ => panic!("string should only contain [a-zA-Z0-9_:]"),
        }
        i += 1;
    }

    if name.as_bytes()[0].is_ascii_digit() {
        panic!("string should not start with a digit")
    }
}

impl MetricName {
    /// Construct a [`MetricNameEncoder`] from a static string, can be used in const expressions.
    ///
    /// # Panics
    /// Will panic if the string contains invalid characters
    pub const fn from_static(value: &'static str) -> &'static Self {
        assert_metric_name(value);

        // SAFETY: `MetricName` is transparent over `str`. There's no way to do this safely.
        // I could use bytemuck::TransparentWrapper, but the trait enabled users to skip this validation function.
        unsafe { &*(value as *const str as *const MetricName) }
    }

    /// Add a namespace prefix to this metric name.
    pub const fn in_namespace(&self, ns: &'static str) -> WithNamespace<&'_ Self> {
        WithNamespace {
            namespace: MetricName::from_static(ns),
            metric_name: self,
        }
    }

    /// Adds a semantic suffix to this metric name.
    pub const fn with_suffix<S: Suffix>(&self, suffix: S) -> WithSuffix<S, &'_ Self> {
        WithSuffix {
            suffix,
            metric_name: self,
        }
    }
}

impl<'a> TryFrom<&'a str> for &'a MetricName {
    type Error = InvalidMetricName;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        // > Metric names may contain ASCII letters, digits, underscores, and colons. It must match the regex [a-zA-Z_:][a-zA-Z0-9_:]*
        if value.is_empty() {
            return Err(InvalidMetricName::Empty);
        }

        value.bytes().try_fold((), |(), b| match b {
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'_' | b':' => Ok(()),
            _ => Err(InvalidMetricName::InvalidChars),
        })?;

        if value.as_bytes()[0].is_ascii_digit() {
            return Err(InvalidMetricName::StartsWithNumber);
        }

        // SAFETY: `CheckedMetricName` is transparent over `str`. There's no way to do this safely.
        // I could use bytemuck::TransparentWrapper, but the trait enabled users to skip this validation function.
        Ok(unsafe { &*(value as *const str as *const MetricName) })
    }
}

impl MetricNameEncoder for MetricName {
    fn encode_text(&self, b: &mut BytesMut) {
        b.extend_from_slice(self.0.as_bytes());
    }
}

/// `Suffix` defines semantic suffixes as suggested by Prometheus
///
/// Included suffixes:
/// * [`Total`] - Good for counters
/// * [`Count`] - Used internally for histograms
/// * [`Sum`] - Used internally for histograms
/// * [`Bucket`] - Used internally for histograms
pub trait Suffix {
    fn encode_text(&self, b: &mut BytesMut);
}

impl<T: MetricNameEncoder + ?Sized> MetricNameEncoder for &T {
    fn encode_text(&self, b: &mut BytesMut) {
        T::encode_text(self, b)
    }
}

/// See [`MetricName::in_namespace`]
pub struct WithNamespace<T: ?Sized> {
    namespace: &'static MetricName,
    metric_name: T,
}

impl<T> WithNamespace<T> {
    /// Adds a semantic suffix to this metric name.
    pub const fn with_suffix<S: Suffix>(self, suffix: S) -> WithSuffix<S, Self> {
        WithSuffix {
            suffix,
            metric_name: self,
        }
    }
}

impl<T: MetricNameEncoder + ?Sized> MetricNameEncoder for WithNamespace<T> {
    fn encode_text(&self, b: &mut BytesMut) {
        b.extend_from_slice(self.namespace.0.as_bytes());
        b.extend_from_slice(b"_");
        self.metric_name.encode_text(b)
    }
}

pub struct WithSuffix<S, T: ?Sized> {
    suffix: S,
    metric_name: T,
}

impl<S: Suffix, T: MetricNameEncoder + ?Sized> MetricNameEncoder for WithSuffix<S, T> {
    fn encode_text(&self, b: &mut BytesMut) {
        self.metric_name.encode_text(b);
        self.suffix.encode_text(b);
    }
}

/// A [`Suffix`] that is good for counters
pub struct Total;
/// A [`Suffix`] that is used internally for histograms
pub struct Count;
/// A [`Suffix`] that is used internally for histograms
pub struct Sum;
/// A [`Suffix`] that is used internally for histograms
pub struct Bucket;

impl Suffix for Total {
    fn encode_text(&self, b: &mut BytesMut) {
        b.extend_from_slice(b"_total");
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