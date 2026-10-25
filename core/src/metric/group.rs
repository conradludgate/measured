//! Groups of metrics

use std::{sync::Arc, time::SystemTime};

pub use crate::label::ComposedGroup;

use crate::label::{LabelGroup, LabelGroupVisitor, LabelName, LabelValue};

use super::{
    exemplar::Exemplar,
    name::{Bucket, Count, GCount, GSum, MetricNameEncoder, Sum, WithNamespace},
};

/// The type of a metric family
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MetricKind {
    /// A monotonically increasing value
    Counter,
    /// A value that can go up and down
    Gauge,
    /// A distribution of observations, counted in buckets
    Histogram,
    /// Quantiles of observations, with a sum and count
    Summary,
    /// A histogram of values that can decrease, eg the age of items currently in a queue.
    ///
    /// The prometheus text format does not support this type, so it is written as untyped.
    GaugeHistogram,
    /// A set of boolean states, eg which state an enum is in.
    ///
    /// The prometheus text and protobuf formats do not support this type, so it is written as a gauge.
    StateSet,
    /// Information exposed as labels on a gauge that is always 1, eg a build version.
    ///
    /// The prometheus text and protobuf formats do not support this type, so it is written as a gauge.
    Info,
    /// A metric with no specific type
    Untyped,
    /// A metric type not listed here, eg `"gaugehistogram"` from OpenMetrics.
    ///
    /// Encoders that support the type write its name as-is. Encoders that do not, such as the
    /// prometheus text and protobuf encoders, write the family as untyped.
    Custom(&'static str),
}

impl MetricKind {
    /// The name of this metric type, as used in the prometheus text format
    pub const fn as_str(&self) -> &'static str {
        match self {
            MetricKind::Counter => "counter",
            MetricKind::Gauge => "gauge",
            MetricKind::Histogram => "histogram",
            MetricKind::Summary => "summary",
            MetricKind::GaugeHistogram => "gaugehistogram",
            MetricKind::StateSet => "stateset",
            MetricKind::Info => "info",
            MetricKind::Untyped => "untyped",
            MetricKind::Custom(name) => name,
        }
    }
}

/// Optional data written with a counter or summary sample.
///
/// Encoders that do not support some of this data, such as the prometheus text encoder, ignore it.
#[derive(Clone, Copy, Debug, Default)]
#[non_exhaustive]
pub struct SampleExtras<'a> {
    /// When the series was created. See [`Created`](super::created::Created).
    pub created: Option<SystemTime>,
    /// An example of what was counted, eg with a trace ID. Summaries do not support exemplars.
    pub exemplar: Option<&'a Exemplar>,
}

impl<'a> SampleExtras<'a> {
    /// No extra data
    pub const NONE: Self = Self {
        created: None,
        exemplar: None,
    };

    /// Set when the series was created
    #[must_use]
    pub fn with_created(self, created: Option<SystemTime>) -> Self {
        Self { created, ..self }
    }

    /// Set the exemplar
    #[must_use]
    pub fn with_exemplar(self, exemplar: Option<&'a Exemplar>) -> Self {
        Self { exemplar, ..self }
    }
}

/// Optional data written with a histogram sample.
///
/// Encoders that do not support some of this data, such as the prometheus text encoder, ignore it.
#[derive(Clone, Copy, Debug, Default)]
#[non_exhaustive]
pub struct HistogramExtras<'a> {
    /// When the series was created. See [`Created`](super::created::Created).
    pub created: Option<SystemTime>,
    /// An exemplar for each bucket. Index `i` is for bucket `le[i]`, and the last is for `+Inf`.
    /// This can be empty if there are no exemplars.
    pub exemplars: &'a [Option<Exemplar>],
}

impl<'a> HistogramExtras<'a> {
    /// No extra data
    pub const NONE: Self = Self {
        created: None,
        exemplars: &[],
    };

    /// Set when the series was created
    #[must_use]
    pub fn with_created(self, created: Option<SystemTime>) -> Self {
        Self { created, ..self }
    }

    /// Set the exemplars for each bucket
    #[must_use]
    pub fn with_exemplars(self, exemplars: &'a [Option<Exemplar>]) -> Self {
        Self { exemplars, ..self }
    }

    /// The exemplar for the bucket at `index`, where `le.len()` is the `+Inf` bucket.
    pub fn exemplar(&self, index: usize) -> Option<&'a Exemplar> {
        self.exemplars.get(index).and_then(Option::as_ref)
    }
}

/// A metric encoder, eg for the prometheus text format.
///
/// Encoders only need to know how to write each kind of sample. Metric types are written with
/// [`MetricEncoding`](super::MetricEncoding), so any metric can be written into any encoder.
///
/// A collection is a sequence of metric families. Each family starts with [`Encoding::write_family`],
/// followed by any number of samples for that family.
pub trait Encoding {
    /// The error type that this encoder might produce when encoding metric values.
    type Err;

    /// The prometheus Content-Type that this encoder exports as
    /// <https://prometheus.io/docs/instrumenting/content_negotiation/#protocol-headers>
    const MIME_TYPE: &'static str;

    /// Start a new metric family. Following samples belong to this family.
    fn write_family(
        &mut self,
        name: impl MetricNameEncoder,
        help: Option<&str>,
        kind: MetricKind,
    ) -> Result<(), Self::Err>;

    /// Write a counter sample
    fn write_counter(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: u64,
        extras: SampleExtras<'_>,
    ) -> Result<(), Self::Err>;

    /// Write an integer gauge sample
    fn write_gauge(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: i64,
    ) -> Result<(), Self::Err>;

    /// Write a float gauge sample
    fn write_float_gauge(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: f64,
    ) -> Result<(), Self::Err>;

    /// Write a float counter sample
    fn write_float_counter(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: f64,
        extras: SampleExtras<'_>,
    ) -> Result<(), Self::Err>;

    /// Write a histogram sample.
    ///
    /// `counts[i]` is the number of observations in bucket `le[i]`. These are *not* cumulative.
    /// `inf` counts the observations greater than every bucket.
    ///
    /// By default, this writes the cumulative `_bucket` counters (including `le="+Inf"`), the `_sum` gauge
    /// and the `_count` counter, as in the prometheus text format. Encoders with a native histogram
    /// representation, such as protobuf, should override this.
    #[allow(clippy::too_many_arguments)]
    fn write_histogram(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        le: &[f64],
        counts: &[u64],
        inf: u64,
        sum: f64,
        extras: HistogramExtras<'_>,
    ) -> Result<(), Self::Err> {
        let mut cumulative = 0;
        for (i, (&le, &count)) in le.iter().zip(counts).enumerate() {
            cumulative += count;
            self.write_counter(
                name.by_ref().with_suffix(Bucket),
                labels.by_ref().compose_with(HistogramLe(le)),
                cumulative,
                SampleExtras::NONE.with_exemplar(extras.exemplar(i)),
            )?;
        }
        let count = cumulative + inf;
        self.write_counter(
            name.by_ref().with_suffix(Bucket),
            labels.by_ref().compose_with(HistogramLe(f64::INFINITY)),
            count,
            SampleExtras::NONE.with_exemplar(extras.exemplar(le.len())),
        )?;
        self.write_float_gauge(name.by_ref().with_suffix(Sum), labels.by_ref(), sum)?;
        self.write_counter(
            name.by_ref().with_suffix(Count),
            labels,
            count,
            SampleExtras::NONE.with_created(extras.created),
        )
    }

    /// Write a summary sample.
    ///
    /// `quantiles` is a list of `(quantile, value)` pairs, eg `(0.99, 1.2)` for a p99 of 1.2.
    ///
    /// By default, this writes the `quantile` gauges, the `_sum` gauge and the `_count` counter,
    /// as in the prometheus text format. Encoders with a native summary representation,
    /// such as protobuf, should override this.
    fn write_summary(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        quantiles: &[(f64, f64)],
        count: u64,
        sum: f64,
        extras: SampleExtras<'_>,
    ) -> Result<(), Self::Err> {
        for &(quantile, value) in quantiles {
            self.write_float_gauge(
                name.by_ref(),
                labels.by_ref().compose_with(SummaryQuantile(quantile)),
                value,
            )?;
        }
        self.write_float_gauge(name.by_ref().with_suffix(Sum), labels.by_ref(), sum)?;
        self.write_counter(
            name.by_ref().with_suffix(Count),
            labels,
            count,
            SampleExtras::NONE.with_created(extras.created),
        )
    }

    /// Write a gauge histogram sample.
    ///
    /// `counts[i]` is the number of values currently in bucket `le[i]`. These are *not* cumulative.
    /// `inf` counts the values greater than every bucket.
    ///
    /// By default, this writes the cumulative `_bucket` gauges (including `le="+Inf"`), the `_gsum` gauge
    /// and the `_gcount` gauge, as in the OpenMetrics text format. Encoders with a native gauge histogram
    /// representation, such as protobuf, should override this.
    fn write_gauge_histogram(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        le: &[f64],
        counts: &[i64],
        inf: i64,
        sum: f64,
    ) -> Result<(), Self::Err> {
        let mut cumulative = 0;
        for (&le, &count) in le.iter().zip(counts) {
            cumulative += count;
            self.write_gauge(
                name.by_ref().with_suffix(Bucket),
                labels.by_ref().compose_with(HistogramLe(le)),
                cumulative,
            )?;
        }
        let count = cumulative + inf;
        self.write_gauge(
            name.by_ref().with_suffix(Bucket),
            labels.by_ref().compose_with(HistogramLe(f64::INFINITY)),
            count,
        )?;
        self.write_gauge(name.by_ref().with_suffix(GCount), labels.by_ref(), count)?;
        self.write_float_gauge(name.by_ref().with_suffix(GSum), labels, sum)
    }

    /// Write a state set sample, with whether each state is currently set.
    ///
    /// By default, this writes a gauge per state that is 1 if set, and 0 if not. As in the OpenMetrics
    /// text format, the state is a label with the same name as the metric. Any `:` in the metric name
    /// is replaced with `_`, as `:` is not valid in label names.
    fn write_state_set<S: LabelValue>(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        states: impl IntoIterator<Item = (S, bool)>,
    ) -> Result<(), Self::Err> {
        let mut label_name = Vec::with_capacity(name.encode_len());
        name.encode_utf8(&mut label_name)
            .expect("writing to a vec should not fail");
        for b in &mut label_name {
            if *b == b':' {
                *b = b'_';
            }
        }
        let label_name =
            core::str::from_utf8(&label_name).expect("metric names are always valid utf8");
        let label_name = LabelName::from_str(label_name);

        for (state, set) in states {
            self.write_gauge(
                name.by_ref(),
                labels.by_ref().compose_with(State(label_name, state)),
                i64::from(set),
            )?;
        }
        Ok(())
    }
}

/// The state label of a state set
struct State<'a, S>(&'a LabelName, S);

impl<S: LabelValue> LabelGroup for State<'_, S> {
    fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
        v.write_value(self.0, &self.1);
    }
}

/// The `quantile` label of a summary
struct SummaryQuantile(f64);

impl LabelGroup for SummaryQuantile {
    fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
        const QUANTILE: &LabelName = LabelName::from_str("quantile");
        v.write_value(QUANTILE, &self.0);
    }
}

/// The `le` label of a histogram bucket
struct HistogramLe(f64);

impl LabelGroup for HistogramLe {
    fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
        const LE: &LabelName = LabelName::from_str("le");
        v.write_value(LE, &self.0);
    }
}

impl<E: Encoding> Encoding for &mut E {
    type Err = E::Err;
    const MIME_TYPE: &'static str = E::MIME_TYPE;

    fn write_family(
        &mut self,
        name: impl MetricNameEncoder,
        help: Option<&str>,
        kind: MetricKind,
    ) -> Result<(), Self::Err> {
        E::write_family(self, name, help, kind)
    }

    fn write_counter(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: u64,
        extras: SampleExtras<'_>,
    ) -> Result<(), Self::Err> {
        E::write_counter(self, name, labels, value, extras)
    }

    fn write_gauge(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: i64,
    ) -> Result<(), Self::Err> {
        E::write_gauge(self, name, labels, value)
    }

    fn write_float_gauge(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: f64,
    ) -> Result<(), Self::Err> {
        E::write_float_gauge(self, name, labels, value)
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
    ) -> Result<(), Self::Err> {
        E::write_histogram(self, name, labels, le, counts, inf, sum, extras)
    }

    fn write_summary(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        quantiles: &[(f64, f64)],
        count: u64,
        sum: f64,
        extras: SampleExtras<'_>,
    ) -> Result<(), Self::Err> {
        E::write_summary(self, name, labels, quantiles, count, sum, extras)
    }

    fn write_float_counter(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: f64,
        extras: SampleExtras<'_>,
    ) -> Result<(), Self::Err> {
        E::write_float_counter(self, name, labels, value, extras)
    }

    fn write_gauge_histogram(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        le: &[f64],
        counts: &[i64],
        inf: i64,
        sum: f64,
    ) -> Result<(), Self::Err> {
        E::write_gauge_histogram(self, name, labels, le, counts, inf, sum)
    }

    fn write_state_set<S: LabelValue>(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        states: impl IntoIterator<Item = (S, bool)>,
    ) -> Result<(), Self::Err> {
        E::write_state_set(self, name, labels, states)
    }
}

/// A `MetricGroup` defines a group of [`MetricFamilyEncoding`](super::MetricFamilyEncoding)s
pub trait MetricGroup<Enc: Encoding> {
    /// Collect the group of metric families into the encoder
    fn collect_group_into(&self, enc: &mut Enc) -> Result<(), Enc::Err>;
}

impl<G, E> MetricGroup<E> for &G
where
    G: MetricGroup<E>,
    E: Encoding,
{
    fn collect_group_into(&self, enc: &mut E) -> Result<(), E::Err> {
        G::collect_group_into(self, enc)
    }
}

impl<A, B, E> MetricGroup<E> for ComposedGroup<A, B>
where
    A: MetricGroup<E>,
    B: MetricGroup<E>,
    E: Encoding,
{
    fn collect_group_into(&self, enc: &mut E) -> Result<(), E::Err> {
        self.0.collect_group_into(enc)?;
        self.1.collect_group_into(enc)?;
        Ok(())
    }
}

impl<G, E> MetricGroup<E> for WithNamespace<G>
where
    G: for<'a> MetricGroup<WithNamespace<&'a mut E>>,
    E: Encoding,
{
    fn collect_group_into(&self, enc: &mut E) -> Result<(), E::Err> {
        self.inner.collect_group_into(&mut WithNamespace {
            namespace: self.namespace,
            inner: enc,
        })
    }
}

impl<M: MetricGroup<T>, T: Encoding> MetricGroup<T> for Option<M> {
    fn collect_group_into(&self, enc: &mut T) -> Result<(), T::Err> {
        if let Some(this) = self {
            this.collect_group_into(enc)?;
        }
        Ok(())
    }
}

impl<M: MetricGroup<T>, T: Encoding> MetricGroup<T> for Arc<M> {
    fn collect_group_into(&self, enc: &mut T) -> Result<(), T::Err> {
        M::collect_group_into(self, enc)
    }
}

impl<E: Encoding> Encoding for WithNamespace<E> {
    type Err = E::Err;

    const MIME_TYPE: &'static str = E::MIME_TYPE;

    fn write_family(
        &mut self,
        name: impl MetricNameEncoder,
        help: Option<&str>,
        kind: MetricKind,
    ) -> Result<(), Self::Err> {
        let name = WithNamespace {
            namespace: self.namespace,
            inner: name,
        };
        self.inner.write_family(name, help, kind)
    }

    fn write_counter(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: u64,
        extras: SampleExtras<'_>,
    ) -> Result<(), Self::Err> {
        let name = WithNamespace {
            namespace: self.namespace,
            inner: name,
        };
        self.inner.write_counter(name, labels, value, extras)
    }

    fn write_gauge(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: i64,
    ) -> Result<(), Self::Err> {
        let name = WithNamespace {
            namespace: self.namespace,
            inner: name,
        };
        self.inner.write_gauge(name, labels, value)
    }

    fn write_float_gauge(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: f64,
    ) -> Result<(), Self::Err> {
        let name = WithNamespace {
            namespace: self.namespace,
            inner: name,
        };
        self.inner.write_float_gauge(name, labels, value)
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
    ) -> Result<(), Self::Err> {
        let name = WithNamespace {
            namespace: self.namespace,
            inner: name,
        };
        self.inner
            .write_histogram(name, labels, le, counts, inf, sum, extras)
    }

    fn write_summary(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        quantiles: &[(f64, f64)],
        count: u64,
        sum: f64,
        extras: SampleExtras<'_>,
    ) -> Result<(), Self::Err> {
        let name = WithNamespace {
            namespace: self.namespace,
            inner: name,
        };
        self.inner
            .write_summary(name, labels, quantiles, count, sum, extras)
    }

    fn write_float_counter(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: f64,
        extras: SampleExtras<'_>,
    ) -> Result<(), Self::Err> {
        let name = WithNamespace {
            namespace: self.namespace,
            inner: name,
        };
        self.inner.write_float_counter(name, labels, value, extras)
    }

    fn write_gauge_histogram(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        le: &[f64],
        counts: &[i64],
        inf: i64,
        sum: f64,
    ) -> Result<(), Self::Err> {
        let name = WithNamespace {
            namespace: self.namespace,
            inner: name,
        };
        self.inner
            .write_gauge_histogram(name, labels, le, counts, inf, sum)
    }

    fn write_state_set<S: LabelValue>(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        states: impl IntoIterator<Item = (S, bool)>,
    ) -> Result<(), Self::Err> {
        let name = WithNamespace {
            namespace: self.namespace,
            inner: name,
        };
        self.inner.write_state_set(name, labels, states)
    }
}
