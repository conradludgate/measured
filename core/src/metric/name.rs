//! Metric names and name encodings

use std::io::Write;

/// `MetricName` represents a type that can be encoded into the name of a metric when collected.
pub trait MetricNameEncoder {
    /// Encoded this name into the given bytes buffer according to the Prometheus metric name encoding specification.
    ///
    /// See <https://prometheus.io/docs/concepts/data_model/#metric-names-and-labels>
    fn encode_utf8(&self, b: &mut impl Write) -> std::io::Result<()>;

    /// The length of the utf8 string this metric name encodes to.
    fn encode_len(&self) -> usize;

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

/// Error returned by [`MetricName::try_from_str`]
#[derive(Debug)]
pub enum InvalidMetricName {
    /// The metric name contained invalid characters
    InvalidChars,
    /// The metric name was empty
    Empty,
    /// The metric name started with a number
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

/// Represents a string-based [`MetricNameEncoder`]
///
/// Metric names may contain ASCII letters, digits, underscores, and colons. It must match the regex `[a-zA-Z_:][a-zA-Z0-9_:]*`.
pub struct MetricName(str);

const fn const_assert_metric_name(name: &str) {
    assert!(!name.is_empty(), "string should not be empty");

    let mut i = 0;
    while i < name.len() {
        match name.as_bytes()[i] {
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'_' | b':' => {}
            _ => panic!("string should only contain [a-zA-Z0-9_:]"),
        }
        i += 1;
    }

    assert!(
        !name.as_bytes()[0].is_ascii_digit(),
        "string should not start with a digit"
    );
}

fn try_assert_metric_name(value: &str) -> Result<(), InvalidMetricName> {
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

    Ok(())
}

impl MetricName {
    /// Construct a [`MetricName`] from a string, can be used in const expressions.
    ///
    /// # Panics
    /// Will panic if the string contains invalid characters
    #[must_use]
    pub const fn from_str(value: &'static str) -> &Self {
        const_assert_metric_name(value);

        // SAFETY: `MetricName` is transparent over `str`. There's no way to do this safely.
        // I could use bytemuck::TransparentWrapper, but the trait enabled users to skip this validation function.
        unsafe { &*(value as *const str as *const MetricName) }
    }

    /// Construct a [`MetricName`] from a string
    ///
    /// # Errors
    /// Will panic if the string contains invalid characters
    pub fn try_from_str(value: &str) -> Result<&Self, InvalidMetricName> {
        try_assert_metric_name(value)?;

        // SAFETY: `MetricName` is transparent over `str`. There's no way to do this safely.
        // I could use bytemuck::TransparentWrapper, but the trait enabled users to skip this validation function.
        Ok(unsafe { &*(value as *const str as *const MetricName) })
    }

    /// Add a namespace prefix to this metric name.
    #[must_use]
    pub const fn in_namespace(&self, ns: &'static str) -> WithNamespace<&'_ Self> {
        WithNamespace {
            namespace: MetricName::from_str(ns),
            inner: self,
        }
    }

    /// Adds a semantic suffix to this metric name.
    #[must_use]
    pub const fn with_suffix<S: Suffix>(&self, suffix: S) -> WithSuffix<S, &'_ Self> {
        WithSuffix {
            suffix,
            metric_name: self,
        }
    }
}

impl MetricNameEncoder for MetricName {
    fn encode_utf8(&self, b: &mut impl Write) -> std::io::Result<()> {
        b.write_all(self.0.as_bytes())
    }
    fn encode_len(&self) -> usize {
        self.0.len()
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
    /// Write `_` followed by the suffix value with to the underlying writer
    fn encode_text(&self, b: &mut impl Write) -> std::io::Result<()>;

    /// The length of the utf8 string this suffix encodes to.
    fn encode_len(&self) -> usize;
}

impl<T: MetricNameEncoder + ?Sized> MetricNameEncoder for &T {
    fn encode_utf8(&self, b: &mut impl Write) -> std::io::Result<()> {
        T::encode_utf8(self, b)
    }
    fn encode_len(&self) -> usize {
        T::encode_len(self)
    }
}

/// See [`MetricName::in_namespace`]
pub struct WithNamespace<T: ?Sized> {
    pub(crate) namespace: &'static MetricName,
    pub(crate) inner: T,
}

impl<T> WithNamespace<T> {
    /// Create a new namespaced value.
    ///
    /// # Panics
    /// Will panic if the `ns` string contains invalid metric name characters
    pub const fn new(ns: &'static str, inner: T) -> Self {
        Self {
            namespace: MetricName::from_str(ns),
            inner,
        }
    }

    /// Adds a semantic suffix to this metric name.
    pub const fn with_suffix<S: Suffix>(self, suffix: S) -> WithSuffix<S, Self> {
        WithSuffix {
            suffix,
            metric_name: self,
        }
    }
}

impl<T: MetricNameEncoder + ?Sized> MetricNameEncoder for WithNamespace<T> {
    fn encode_utf8(&self, b: &mut impl Write) -> std::io::Result<()> {
        b.write_all(self.namespace.0.as_bytes())?;
        b.write_all(b"_")?;
        self.inner.encode_utf8(b)
    }
    fn encode_len(&self) -> usize {
        self.namespace.0.len() + 1 + self.inner.encode_len()
    }
}

/// See [`MetricName::with_suffix`]
pub struct WithSuffix<S, T: ?Sized> {
    suffix: S,
    metric_name: T,
}

impl<S: Suffix, T: MetricNameEncoder + ?Sized> MetricNameEncoder for WithSuffix<S, T> {
    fn encode_utf8(&self, b: &mut impl Write) -> std::io::Result<()> {
        self.metric_name.encode_utf8(b)?;
        self.suffix.encode_text(b)
    }
    fn encode_len(&self) -> usize {
        self.metric_name.encode_len() + self.suffix.encode_len()
    }
}

/// `_total`. A [`Suffix`] that is good for counters
pub struct Total;
/// `_count`. A [`Suffix`] that is used internally for histograms
pub struct Count;
/// `_sum`. A [`Suffix`] that is used internally for histograms
pub struct Sum;
/// `_bucket`. A [`Suffix`] that is used internally for histograms
pub struct Bucket;

impl Suffix for Total {
    fn encode_text(&self, b: &mut impl Write) -> std::io::Result<()> {
        b.write_all(b"_total")
    }
    fn encode_len(&self) -> usize {
        6
    }
}

impl Suffix for Count {
    fn encode_text(&self, b: &mut impl Write) -> std::io::Result<()> {
        b.write_all(b"_count")
    }
    fn encode_len(&self) -> usize {
        6
    }
}

impl Suffix for Sum {
    fn encode_text(&self, b: &mut impl Write) -> std::io::Result<()> {
        b.write_all(b"_sum")
    }
    fn encode_len(&self) -> usize {
        4
    }
}

impl Suffix for Bucket {
    fn encode_text(&self, b: &mut impl Write) -> std::io::Result<()> {
        b.write_all(b"_bucket")
    }
    fn encode_len(&self) -> usize {
        7
    }
}
