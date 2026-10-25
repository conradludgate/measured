//! Metrics that record when their series was created. See [`Created`]

use core::ops::Deref;
use std::time::SystemTime;

use crate::{
    CreatedCounter, CreatedCounterVec, CreatedFloatCounter, CreatedFloatCounterVec,
    CreatedHistogram, CreatedHistogramVec,
    label::{LabelGroup, LabelGroupSet, LabelValue},
};

use super::{
    MetricEncoding, MetricLockGuard, MetricType,
    group::{Encoding, HistogramExtras, MetricKind, SampleExtras},
    histogram::{HistogramSample, HistogramState},
    name::MetricNameEncoder,
};

/// Wraps a metric to record when each series was created.
///
/// The created timestamp lets Prometheus tell when a counter was reset, even if it was not scraped in between.
/// It is written by the OpenMetrics and protobuf encoders for counters, histograms and summaries, and ignored
/// by the prometheus text encoder.
///
/// Each series stores an extra timestamp, which is why this is not done by default.
/// See [`CreatedCounter`], [`CreatedFloatCounter`] and [`CreatedHistogram`].
#[derive(Debug)]
pub struct Created<M> {
    metric: M,
    created: SystemTime,
}

impl<M: Default> Default for Created<M> {
    fn default() -> Self {
        Self {
            metric: M::default(),
            created: SystemTime::now(),
        }
    }
}

impl<M> Created<M> {
    /// When this series was created
    pub fn created(&self) -> SystemTime {
        self.created
    }
}

impl<M> Deref for Created<M> {
    type Target = M;

    fn deref(&self) -> &M {
        &self.metric
    }
}

impl<M: MetricType> MetricType for Created<M> {
    type Metadata = M::Metadata;
}

impl<M: MetricEncoding> MetricEncoding for Created<M> {
    const KIND: MetricKind = M::KIND;

    fn collect_into<E: Encoding>(
        &self,
        metadata: &M::Metadata,
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut E,
    ) -> Result<(), E::Err> {
        let mut enc = WithCreated {
            inner: enc,
            created: self.created,
        };
        self.metric.collect_into(metadata, labels, name, &mut enc)
    }
}

/// Adds the created timestamp to every sample that supports it
struct WithCreated<'a, E> {
    inner: &'a mut E,
    created: SystemTime,
}

impl<E: Encoding> Encoding for WithCreated<'_, E> {
    type Err = E::Err;
    const MIME_TYPE: &'static str = E::MIME_TYPE;

    fn write_family(
        &mut self,
        name: impl MetricNameEncoder,
        help: Option<&str>,
        kind: MetricKind,
    ) -> Result<(), E::Err> {
        self.inner.write_family(name, help, kind)
    }

    fn write_counter(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: u64,
        extras: SampleExtras<'_>,
    ) -> Result<(), E::Err> {
        let extras = extras.with_created(extras.created.or(Some(self.created)));
        self.inner.write_counter(name, labels, value, extras)
    }

    fn write_float_counter(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: f64,
        extras: SampleExtras<'_>,
    ) -> Result<(), E::Err> {
        let extras = extras.with_created(extras.created.or(Some(self.created)));
        self.inner.write_float_counter(name, labels, value, extras)
    }

    fn write_gauge(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: i64,
    ) -> Result<(), E::Err> {
        self.inner.write_gauge(name, labels, value)
    }

    fn write_float_gauge(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        value: f64,
    ) -> Result<(), E::Err> {
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
    ) -> Result<(), E::Err> {
        let extras = extras.with_created(extras.created.or(Some(self.created)));
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
    ) -> Result<(), E::Err> {
        let extras = extras.with_created(extras.created.or(Some(self.created)));
        self.inner
            .write_summary(name, labels, quantiles, count, sum, extras)
    }

    fn write_gauge_histogram(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        le: &[f64],
        counts: &[i64],
        inf: i64,
        sum: f64,
    ) -> Result<(), E::Err> {
        self.inner
            .write_gauge_histogram(name, labels, le, counts, inf, sum)
    }

    fn write_state_set<S: LabelValue>(
        &mut self,
        name: impl MetricNameEncoder,
        labels: impl LabelGroup,
        states: impl IntoIterator<Item = (S, bool)>,
    ) -> Result<(), E::Err> {
        self.inner.write_state_set(name, labels, states)
    }
}

macro_rules! created_counter {
    ($metric:ident, $vec:ident, $value:ty, $zero:expr) => {
        impl $metric {
            /// Increment the counter value by 1
            pub fn inc(&self) {
                self.get_metric().inc();
            }

            /// Increment the counter value by `x`
            pub fn inc_by(&self, x: $value) {
                self.get_metric().inc_by(x);
            }

            /// Get the current counter value
            pub fn get(&self) -> $value {
                self.get_metric().get()
            }

            /// When this counter was created
            pub fn created(&self) -> SystemTime {
                self.get_metric().created()
            }
        }

        impl<L: LabelGroupSet> $vec<L> {
            /// Increment the counter value by 1, keyed by the label group
            pub fn inc(&self, label: L::Group<'_>) {
                self.get_metric(self.with_labels(label)).inc();
            }

            /// Increment the counter value by `x`, keyed by the label group
            pub fn inc_by(&self, label: L::Group<'_>, x: $value) {
                self.get_metric(self.with_labels(label)).inc_by(x);
            }

            /// Get the counter value for the label group. Returns 0 if the counter has not been used yet.
            ///
            /// This does not create a new counter for the label group, or add its values to dynamic label sets.
            pub fn get(&self, label: L::Group<'_>) -> $value {
                self.find_metric(self.with_labels(label))
                    .map_or($zero, |m| m.get())
            }
        }
    };
}

created_counter!(CreatedCounter, CreatedCounterVec, u64, 0);
created_counter!(CreatedFloatCounter, CreatedFloatCounterVec, f64, 0.0);

/// A shared ref to an individual histogram that records when it was created
pub type CreatedHistogramLockGuard<'a, const N: usize> =
    MetricLockGuard<'a, Created<HistogramState<N>>>;

impl<const N: usize> CreatedHistogramLockGuard<'_, N> {
    /// Add a single observation to the histogram.
    pub fn observe(self, x: f64) {
        self.metric.observe_with(self.metadata(), x);
    }
}

impl<const N: usize> CreatedHistogram<N> {
    /// Add a single observation to the histogram.
    pub fn observe(&self, x: f64) {
        self.get_metric().observe(x);
    }

    /// Take a consistent snapshot of the histogram.
    pub fn sample(&self) -> HistogramSample<N> {
        self.get_metric().sample()
    }

    /// When this histogram was created
    pub fn created(&self) -> SystemTime {
        self.get_metric().created()
    }
}

impl<L: LabelGroupSet, const N: usize> CreatedHistogramVec<L, N> {
    /// Add a single observation to the histogram, keyed by the label group.
    pub fn observe(&self, label: L::Group<'_>, x: f64) {
        self.get_metric(self.with_labels(label)).observe(x);
    }

    /// Take a consistent snapshot of the histogram for the label group.
    /// Returns an empty sample if the histogram has not been used yet.
    ///
    /// This does not create a new histogram for the label group, or add its values to dynamic label sets.
    pub fn sample(&self, label: L::Group<'_>) -> HistogramSample<N> {
        self.find_metric(self.with_labels(label))
            .map_or(HistogramSample::default(), |m| m.sample())
    }
}
