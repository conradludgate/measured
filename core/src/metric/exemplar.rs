//! Exemplars, examples of observations with extra labels such as a trace ID. See [`Exemplar`]

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use parking_lot::Mutex;

use super::group::{Encoding, HistogramExtras, MetricKind, SampleExtras};
use super::histogram::{HistogramSample, HistogramState, Thresholds};
use super::name::MetricNameEncoder;
use super::{MetricEncoding, MetricLockGuard, MetricType};
use crate::label::{LabelGroup, LabelGroupSet, LabelGroupVisitor, LabelName};
use crate::{ExemplarCounter, ExemplarCounterVec, ExemplarHistogram, ExemplarHistogramVec};

/// An example of an observation, with labels that identify it, such as a trace ID.
///
/// Exemplars are written by the OpenMetrics and protobuf encoders, and ignored by the prometheus text encoder.
/// OpenMetrics limits the labels of an exemplar to 128 characters in total. Exemplars with longer labels
/// are not written by the OpenMetrics encoder.
///
/// ```
/// use measured::ExemplarCounter;
/// use measured::label::LabelName;
/// use measured::metric::exemplar::Exemplar;
///
/// const TRACE_ID: &LabelName = LabelName::from_str("trace_id");
///
/// let requests = ExemplarCounter::new();
/// requests.inc_with_exemplar(1, Exemplar::new().with_label(TRACE_ID, "4bf92f3577b34da6"));
/// ```
#[derive(Clone)]
pub struct Exemplar {
    labels: Vec<(&'static LabelName, String)>,
    value: f64,
    timestamp: Option<SystemTime>,
}

impl core::fmt::Debug for Exemplar {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Exemplar")
            .field(
                "labels",
                &self
                    .labels
                    .iter()
                    .map(|(name, value)| (name.as_str(), value))
                    .collect::<Vec<_>>(),
            )
            .field("value", &self.value)
            .field("timestamp", &self.timestamp)
            .finish()
    }
}

impl PartialEq for Exemplar {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.timestamp == other.timestamp
            && self.labels.len() == other.labels.len()
            && self
                .labels
                .iter()
                .zip(&other.labels)
                .all(|((a, x), (b, y))| a.as_str() == b.as_str() && x == y)
    }
}

impl Default for Exemplar {
    fn default() -> Self {
        Self::new()
    }
}

impl Exemplar {
    /// An exemplar with no labels.
    ///
    /// When recorded with a metric, such as with [`ExemplarCounter::inc_with_exemplar`], the value is set
    /// to the observed value.
    pub fn new() -> Self {
        Self {
            labels: Vec::new(),
            value: 0.0,
            timestamp: None,
        }
    }

    /// Add a label to this exemplar
    #[must_use]
    pub fn with_label(mut self, name: &'static LabelName, value: impl Into<String>) -> Self {
        self.labels.push((name, value.into()));
        self
    }

    /// Set the value of this exemplar
    #[must_use]
    pub fn with_value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    /// Set when this exemplar was observed
    #[must_use]
    pub fn with_timestamp(mut self, timestamp: SystemTime) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// The labels of this exemplar
    pub fn labels(&self) -> impl LabelGroup + '_ {
        ExemplarLabels(&self.labels)
    }

    /// The number of characters in the label names and values, which OpenMetrics limits to 128.
    pub fn labels_len(&self) -> usize {
        self.labels
            .iter()
            .map(|(name, value)| name.as_str().chars().count() + value.chars().count())
            .sum()
    }

    /// The observed value
    pub fn value(&self) -> f64 {
        self.value
    }

    /// When this exemplar was observed, if set
    pub fn timestamp(&self) -> Option<SystemTime> {
        self.timestamp
    }
}

struct ExemplarLabels<'a>(&'a [(&'static LabelName, String)]);

impl LabelGroup for ExemplarLabels<'_> {
    fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
        for (name, value) in self.0 {
            v.write_value(name, value);
        }
    }
}

/// The state of a counter that records the most recent exemplar. See [`ExemplarCounter`]
#[derive(Default)]
pub struct ExemplarCounterState {
    count: AtomicU64,
    exemplar: Mutex<Option<Exemplar>>,
}

impl ExemplarCounterState {
    /// Get the current counter value
    pub fn get(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get the most recent exemplar
    pub fn exemplar(&self) -> Option<Exemplar> {
        self.exemplar.lock().clone()
    }

    /// Increment the counter value by 1
    pub fn inc(&self) {
        self.inc_by(1);
    }

    /// Increment the counter value by `x`
    pub fn inc_by(&self, x: u64) {
        self.count.fetch_add(x, Ordering::Relaxed);
    }

    /// Increment the counter value by `x`, and record the exemplar with a value of `x`
    pub fn inc_with_exemplar(&self, x: u64, exemplar: Exemplar) {
        let mut stored = self.exemplar.lock();
        self.inc_by(x);
        *stored = Some(exemplar.with_value(x as f64));
    }
}

impl MetricType for ExemplarCounterState {
    type Metadata = ();
}

impl MetricEncoding for ExemplarCounterState {
    const KIND: MetricKind = MetricKind::Counter;

    fn collect_into<E: Encoding>(
        &self,
        _: &(),
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut E,
    ) -> Result<(), E::Err> {
        let (count, exemplar) = {
            let exemplar = self.exemplar.lock();
            (self.get(), exemplar.clone())
        };
        let extras = SampleExtras::NONE.with_exemplar(exemplar.as_ref());
        enc.write_counter(name, labels, count, extras)
    }
}

impl ExemplarCounter {
    /// Increment the counter value by 1
    pub fn inc(&self) {
        self.get_metric().inc();
    }

    /// Increment the counter value by `x`
    pub fn inc_by(&self, x: u64) {
        self.get_metric().inc_by(x);
    }

    /// Increment the counter value by `x`, and record the exemplar with a value of `x`
    pub fn inc_with_exemplar(&self, x: u64, exemplar: Exemplar) {
        self.get_metric().inc_with_exemplar(x, exemplar);
    }

    /// Get the current counter value
    pub fn get(&self) -> u64 {
        self.get_metric().get()
    }
}

impl<L: LabelGroupSet> ExemplarCounterVec<L> {
    /// Increment the counter value by 1, keyed by the label group
    pub fn inc(&self, label: L::Group<'_>) {
        self.get_metric(self.with_labels(label)).inc();
    }

    /// Increment the counter value by `x`, keyed by the label group
    pub fn inc_by(&self, label: L::Group<'_>, x: u64) {
        self.get_metric(self.with_labels(label)).inc_by(x);
    }

    /// Increment the counter value by `x`, and record the exemplar with a value of `x`, keyed by the label group
    pub fn inc_with_exemplar(&self, label: L::Group<'_>, x: u64, exemplar: Exemplar) {
        self.get_metric(self.with_labels(label))
            .inc_with_exemplar(x, exemplar);
    }

    /// Get the counter value for the label group. Returns 0 if the counter has not been used yet.
    ///
    /// This does not create a new counter for the label group, or add its values to dynamic label sets.
    pub fn get(&self, label: L::Group<'_>) -> u64 {
        self.find_labels(label)
            .and_then(|id| self.find_metric(id))
            .map_or(0, |m| m.get())
    }
}

/// The state of a histogram that records the most recent exemplar in each bucket. See [`ExemplarHistogram`]
pub struct ExemplarHistogramState<const N: usize> {
    histogram: HistogramState<N>,
    /// one exemplar per bucket, with the last for `+Inf`
    exemplars: Mutex<Box<[Option<Exemplar>]>>,
}

impl<const N: usize> Default for ExemplarHistogramState<N> {
    fn default() -> Self {
        Self {
            histogram: HistogramState::default(),
            exemplars: Mutex::new(vec![None; N + 1].into_boxed_slice()),
        }
    }
}

impl<const N: usize> ExemplarHistogramState<N> {
    /// Take a consistent snapshot of the histogram.
    pub fn sample(&self) -> HistogramSample<N> {
        self.histogram.sample()
    }

    /// The most recent exemplar in each bucket. The last is for the `+Inf` bucket.
    pub fn exemplars(&self) -> Vec<Option<Exemplar>> {
        self.exemplars.lock().to_vec()
    }
}

impl<const N: usize> MetricType for ExemplarHistogramState<N> {
    type Metadata = Thresholds<N>;
}

/// A shared ref to an individual exemplar histogram
pub type ExemplarHistogramLockGuard<'a, const N: usize> = MetricLockGuard<'a, ExemplarHistogramState<N>>;

impl<const N: usize> ExemplarHistogramLockGuard<'_, N> {
    /// Add a single observation to the histogram.
    pub fn observe(self, x: f64) {
        self.histogram.observe_with(self.metadata(), x);
    }

    /// Add a single observation to the histogram, and record the exemplar in its bucket with a value of `x`.
    pub fn observe_with_exemplar(self, x: f64, exemplar: Exemplar) {
        let mut exemplars = self.exemplars.lock();
        let bucket = self.histogram.observe_with(self.metadata(), x);
        // NaN observations are counted, but cannot identify a value inside a bucket.
        if !x.is_nan() {
            exemplars[bucket] = Some(exemplar.with_value(x));
        }
    }
}

impl<const N: usize> ExemplarHistogram<N> {
    /// Add a single observation to the histogram.
    pub fn observe(&self, x: f64) {
        self.get_metric().observe(x);
    }

    /// Add a single observation to the histogram, and record the exemplar in its bucket with a value of `x`.
    pub fn observe_with_exemplar(&self, x: f64, exemplar: Exemplar) {
        self.get_metric().observe_with_exemplar(x, exemplar);
    }

    /// Take a consistent snapshot of the histogram.
    pub fn sample(&self) -> HistogramSample<N> {
        self.get_metric().sample()
    }
}

impl<L: LabelGroupSet, const N: usize> ExemplarHistogramVec<L, N> {
    /// Add a single observation to the histogram, keyed by the label group.
    pub fn observe(&self, label: L::Group<'_>, x: f64) {
        self.get_metric(self.with_labels(label)).observe(x);
    }

    /// Add a single observation to the histogram, and record the exemplar in its bucket with a value of `x`,
    /// keyed by the label group.
    pub fn observe_with_exemplar(&self, label: L::Group<'_>, x: f64, exemplar: Exemplar) {
        self.get_metric(self.with_labels(label))
            .observe_with_exemplar(x, exemplar);
    }

    /// Take a consistent snapshot of the histogram for the label group.
    /// Returns an empty sample if the histogram has not been used yet.
    ///
    /// This does not create a new histogram for the label group, or add its values to dynamic label sets.
    pub fn sample(&self, label: L::Group<'_>) -> HistogramSample<N> {
        self.find_labels(label)
            .and_then(|id| self.find_metric(id))
            .map_or(HistogramSample::default(), |m| m.sample())
    }
}

impl<const N: usize> MetricEncoding for ExemplarHistogramState<N> {
    const KIND: MetricKind = MetricKind::Histogram;

    fn collect_into<E: Encoding>(
        &self,
        metadata: &Thresholds<N>,
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut E,
    ) -> Result<(), E::Err> {
        let (sample, exemplars) = {
            let exemplars = self.exemplars.lock();
            (self.histogram.sample(), exemplars.clone())
        };
        enc.write_histogram(
            name,
            labels,
            metadata.get(),
            &sample.buckets,
            sample.inf,
            sample.sum,
            HistogramExtras::NONE.with_exemplars(&exemplars),
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::label::NoLabels;
    use crate::metric::name::MetricName;

    struct Probe<F>(F);
    impl<F: Fn()> Encoding for Probe<F> {
        type Err = std::convert::Infallible;
        const MIME_TYPE: &'static str = "test/probe";
        fn write_family(
            &mut self,
            _: impl MetricNameEncoder,
            _: MetricKind,
            _: super::super::group::FamilyMeta<'_>,
        ) -> Result<(), Self::Err> {
            (self.0)();
            Ok(())
        }
        fn write_counter(
            &mut self,
            _: impl MetricNameEncoder,
            _: impl LabelGroup,
            _: u64,
            _: SampleExtras<'_>,
        ) -> Result<(), Self::Err> {
            (self.0)();
            Ok(())
        }
        fn write_gauge(&mut self, _: impl MetricNameEncoder, _: impl LabelGroup, _: i64) -> Result<(), Self::Err> {
            (self.0)();
            Ok(())
        }
        fn write_float_gauge(
            &mut self,
            _: impl MetricNameEncoder,
            _: impl LabelGroup,
            _: f64,
        ) -> Result<(), Self::Err> {
            (self.0)();
            Ok(())
        }
        fn write_float_counter(
            &mut self,
            _: impl MetricNameEncoder,
            _: impl LabelGroup,
            _: f64,
            _: SampleExtras<'_>,
        ) -> Result<(), Self::Err> {
            (self.0)();
            Ok(())
        }
    }

    #[test]
    fn encoding_releases_exemplar_locks_before_writing() {
        let counter = ExemplarCounterState::default();
        counter.inc_with_exemplar(1, Exemplar::new());
        let mut enc = Probe(|| assert!(counter.exemplar.try_lock().is_some()));
        counter
            .collect_into(&(), NoLabels, MetricName::from_str("counter"), &mut enc)
            .unwrap();

        let histogram = ExemplarHistogram::<1>::with_metadata(Thresholds::with_buckets([1.0]));
        histogram.observe_with_exemplar(0.5, Exemplar::new());
        let state = histogram.get_metric();
        let mut enc = Probe(|| assert!(state.exemplars.try_lock().is_some()));
        state
            .collect_into(
                state.metadata(),
                NoLabels,
                MetricName::from_str("histogram"),
                &mut enc,
            )
            .unwrap();
    }

    #[test]
    fn nan_observations_do_not_replace_exemplars() {
        let histogram = ExemplarHistogram::<1>::with_metadata(Thresholds::with_buckets([1.0]));
        histogram.observe_with_exemplar(f64::INFINITY, Exemplar::new());
        histogram.observe_with_exemplar(f64::NAN, Exemplar::new());
        assert_eq!(histogram.sample().inf, 2);
        assert_eq!(
            histogram.get_metric().exemplars()[1]
                .as_ref()
                .unwrap()
                .value(),
            f64::INFINITY
        );
    }

    #[test]
    fn derive_resolves_the_current_crate() {
        #[derive(crate::MetricGroup, Default)]
        #[metric(crate = crate)]
        struct Metrics {
            requests_total: crate::Counter,
        }
        let metrics = Metrics::default();
        metrics.requests_total.inc();
        use crate::MetricGroup;
        let mut enc = Probe(|| {});
        metrics.collect_group_into(&mut enc).unwrap();
        assert_eq!(metrics.requests_total.get(), 1);
    }
}
