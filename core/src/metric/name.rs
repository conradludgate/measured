use std::io::Write;

/// `MetricName` represents a type that can be encoded into the name of a metric when collected.
pub trait MetricNameEncoder {
    /// Encoded this name into the given bytes buffer according to the Prometheus metric name encoding specification.
    ///
    /// See <https://prometheus.io/docs/concepts/data_model/#metric-names-and-labels>
    fn encode_text(&self, b: &mut impl Write) -> std::io::Result<()>;

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

/// Error returned by [`MetricName::try_from`]
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

const fn assert_metric_name(name: &str) {
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

impl MetricName {
    /// Construct a [`MetricName`] from a string, can be used in const expressions.
    ///
    /// # Panics
    /// Will panic if the string contains invalid characters
    #[must_use]
    pub const fn from_str(value: &str) -> &Self {
        assert_metric_name(value);

        // SAFETY: `MetricName` is transparent over `str`. There's no way to do this safely.
        // I could use bytemuck::TransparentWrapper, but the trait enabled users to skip this validation function.
        unsafe { &*(value as *const str as *const MetricName) }
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

        // SAFETY: `MetricName` is transparent over `str`. There's no way to do this safely.
        // I could use bytemuck::TransparentWrapper, but the trait enabled users to skip this validation function.
        Ok(unsafe { &*(value as *const str as *const MetricName) })
    }
}

impl MetricNameEncoder for MetricName {
    fn encode_text(&self, b: &mut impl Write) -> std::io::Result<()> {
        b.write_all(self.0.as_bytes())
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
    fn encode_text(&self, b: &mut impl Write) -> std::io::Result<()>;
}

impl<T: MetricNameEncoder + ?Sized> MetricNameEncoder for &T {
    fn encode_text(&self, b: &mut impl Write) -> std::io::Result<()> {
        T::encode_text(self, b)
    }
}

/// See [`MetricName::in_namespace`]
pub struct WithNamespace<T: ?Sized> {
    pub(crate) namespace: &'static MetricName,
    pub(crate) inner: T,
}

impl<T> WithNamespace<T> {
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
    fn encode_text(&self, b: &mut impl Write) -> std::io::Result<()> {
        b.write_all(self.namespace.0.as_bytes())?;
        b.write_all(b"_")?;
        self.inner.encode_text(b)
    }
}

pub struct WithSuffix<S, T: ?Sized> {
    suffix: S,
    metric_name: T,
}

impl<S: Suffix, T: MetricNameEncoder + ?Sized> MetricNameEncoder for WithSuffix<S, T> {
    fn encode_text(&self, b: &mut impl Write) -> std::io::Result<()> {
        self.metric_name.encode_text(b)?;
        self.suffix.encode_text(b)
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
    fn encode_text(&self, b: &mut impl Write) -> std::io::Result<()> {
        b.write_all(b"_total")
    }
}

impl Suffix for Count {
    fn encode_text(&self, b: &mut impl Write) -> std::io::Result<()> {
        b.write_all(b"_count")
    }
}

impl Suffix for Sum {
    fn encode_text(&self, b: &mut impl Write) -> std::io::Result<()> {
        b.write_all(b"_sum")
    }
}

impl Suffix for Bucket {
    fn encode_text(&self, b: &mut impl Write) -> std::io::Result<()> {
        b.write_all(b"_bucket")
    }
}
