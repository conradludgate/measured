//! All things histograms. See [`Histogram`]

use std::{
    sync::atomic::{AtomicI64, AtomicU64, Ordering},
    time::Duration,
};

use parking_lot::RwLock;

use super::{
    MetricEncoding, MetricLockGuard, MetricMut, MetricType,
    gauge::AtomicF64,
    group::{Encoding, HistogramExtras, MetricKind},
    name::MetricNameEncoder,
};
use crate::{
    GaugeHistogram, GaugeHistogramVec, Histogram, HistogramVec,
    label::{LabelGroup, LabelGroupSet},
};

/// The inner state of a histogram.
///
/// A histogram is comprised of 'buckets' where each bucket tracks a range or possible observations.
/// For instance, an observation of 1.5 would increment a counter in the bucket `1.0..2.0`.
/// If there is no suitable bucket, 'inf' is incremented.
pub(crate) struct HistogramStateInner<const N: usize> {
    /// The buckets count the number of observed values in the ranges described by [`Thresholds`]
    buckets: [AtomicU64; N],
    /// The number of observed values that are greater than described by [`Thresholds`]
    inf: AtomicU64,
    /// The accumulated sum
    sum: AtomicF64,
}

impl<const N: usize> HistogramStateInner<N> {
    /// Add a single observation to the [`Histogram`].
    fn observe(&self, bucket: usize, x: f64) {
        assert!(bucket <= N);
        if bucket < N {
            self.buckets[bucket].fetch_add(1, Ordering::Relaxed);
        } else {
            self.inf.fetch_add(1, Ordering::Relaxed);
        }
        self.sum.inc_by(x);
    }

    /// Add a single observation to the [`Histogram`].
    fn observe_mut(&mut self, bucket: usize, x: f64) {
        assert!(bucket <= N);
        if bucket < N {
            *self.buckets[bucket].get_mut() += 1;
        } else {
            *self.inf.get_mut() += 1;
        }
        let v = self.sum.get_ex();
        self.sum.set_mut(v + x);
    }

    fn sample(&mut self) -> HistogramSample<N> {
        HistogramSample {
            buckets: core::array::from_fn(|i| *self.buckets[i].get_mut()),
            inf: *self.inf.get_mut(),
            sum: self.sum.get_ex(),
        }
    }
}

/// A consistent snapshot of the state of a histogram. See [`HistogramState::sample`].
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct HistogramSample<const N: usize> {
    /// The number of observations in each bucket described by [`Thresholds`].
    ///
    /// These are *not* cumulative: `buckets[i]` counts the observations `x` where `le[i-1] < x <= le[i]`.
    pub buckets: [u64; N],
    /// The number of observations greater than all of the [`Thresholds`], including NaN.
    pub inf: u64,
    /// The sum of all observations.
    pub sum: f64,
}

impl<const N: usize> Default for HistogramSample<N> {
    fn default() -> Self {
        Self {
            buckets: [0; N],
            inf: 0,
            sum: 0.0,
        }
    }
}

impl<const N: usize> HistogramSample<N> {
    /// The total number of observations.
    pub fn count(&self) -> u64 {
        self.buckets.iter().sum::<u64>() + self.inf
    }
}

/// The state of a histogram.
pub struct HistogramState<const N: usize> {
    /// A rwlock over the inner histogram state.
    /// The read lock is acquired for observations.
    /// The write lock is acquired for sampling.
    inner: RwLock<HistogramStateInner<N>>,
}

impl<const N: usize> HistogramState<N> {
    /// Take a consistent snapshot of the histogram.
    ///
    /// This briefly takes the histogram's write lock, so it blocks concurrent observations
    /// until the snapshot is taken.
    pub fn sample(&self) -> HistogramSample<N> {
        self.inner.write().sample()
    }
}

/// A shared ref to an individual histogram
pub type HistogramLockGuard<'a, const N: usize> = MetricLockGuard<'a, HistogramState<N>>;
/// A unique ref to an individual histogram
pub type HistogramMut<'a, const N: usize> = MetricMut<'a, HistogramState<N>>;

impl<const N: usize> Default for HistogramState<N> {
    fn default() -> Self {
        #[allow(clippy::declare_interior_mutable_const)]
        const ZERO: AtomicU64 = AtomicU64::new(0);
        Self {
            inner: RwLock::new(HistogramStateInner {
                buckets: [ZERO; N],
                inf: ZERO,
                sum: AtomicF64::ZERO,
            }),
        }
    }
}

impl<const N: usize> MetricType for HistogramState<N> {
    type Metadata = Thresholds<N>;
}

/// `Thresholds` defines the size of buckets used in a [`Histogram`]
#[derive(Clone, Debug, PartialEq)]
pub struct Thresholds<const N: usize> {
    le: [f64; N],
}

impl<const N: usize> Thresholds<N> {
    /// Create `N` buckets, where the lowest bucket has an upper bound of `start` and each following bucket’s upper bound is `factor` times the previous bucket’s upper bound.
    /// The final +Inf bucket is not counted and not included.
    ///
    /// # Panics
    /// The function panics if `start` is zero or negative, if `factor` is less than or equal 1,
    /// or if the buckets are not strictly increasing (for example, due to overflowing to infinity).
    pub fn exponential_buckets(start: f64, factor: f64) -> Self {
        assert!(
            start > 0.0,
            "exponential_buckets needs a positive start value, start: {start}",
        );

        assert!(
            factor > 1.0,
            "exponential_buckets needs a factor greater than 1, factor: {factor}",
        );

        let buckets = core::array::from_fn(|i| start * factor.powi(i as i32));

        Self::with_buckets(buckets)
    }

    /// Create `N` buckets, each `width`  wide, where the lowest bucket has an upper bound of `start`.
    /// The final +Inf bucket is not counted and not included.
    ///
    /// # Panics
    /// The function panics if `width` is zero or negative, or if the buckets are not strictly increasing
    /// (for example, if `width` is too small relative to `start`).
    pub fn linear_buckets(start: f64, width: f64) -> Self {
        assert!(
            width > 0.0,
            "linear_buckets needs a width greater than 0, width: {width}",
        );

        let buckets = core::array::from_fn(|i| start + width * i as f64);

        Self::with_buckets(buckets)
    }

    /// Create the histogram thresholds with the given sizes
    ///
    /// # Panics
    /// Will panic if the buckets are not strictly monotonically increasing, or contain NaN or +Inf.
    /// The +Inf bucket is always included implicitly.
    pub fn with_buckets(buckets: [f64; N]) -> Self {
        assert!(
            !buckets.iter().any(|b| b.is_nan()),
            "histogram buckets must not be NaN",
        );
        assert!(
            !buckets.contains(&f64::INFINITY),
            "histogram buckets must not contain +Inf, it is included implicitly",
        );
        for w in buckets.windows(2) {
            assert!(
                w[0] < w[1],
                "consecutive histogram buckets must not decrease or be equal",
            );
        }
        Thresholds { le: buckets }
    }

    /// The index of the bucket that `x` should be counted in. `N` represents the +Inf bucket.
    ///
    /// NaN is not less than or equal to any bound, so it is only counted in the +Inf bucket.
    fn bucket(&self, x: f64) -> usize {
        if x.is_nan() {
            N
        } else {
            self.le.partition_point(|le| x > *le)
        }
    }

    /// View the bucket upper bounds
    pub fn get(&self) -> &[f64; N] {
        &self.le
    }
}

impl<const N: usize> HistogramState<N> {
    /// Add a single observation, returning the index of the bucket it was counted in.
    /// `N` is the `+Inf` bucket.
    pub(crate) fn observe_with(&self, thresholds: &Thresholds<N>, x: f64) -> usize {
        let bucket = thresholds.bucket(x);
        self.inner.read().observe(bucket, x);
        bucket
    }
}

impl<const N: usize> HistogramLockGuard<'_, N> {
    /// Add a single observation to the [`Histogram`].
    pub fn observe(self, x: f64) {
        self.observe_with(self.metadata(), x);
    }

    /// Observe the duration in seconds
    pub fn observe_duration(self, duration: std::time::Duration) {
        self.observe(duration.as_secs_f64());
    }

    /// Observe the duration in seconds since the given instant
    pub fn observe_duration_since(self, since: std::time::Instant) -> std::time::Duration {
        let d = since.elapsed();
        self.observe_duration(d);
        d
    }
}

impl<const N: usize> HistogramMut<'_, N> {
    /// Add a single observation to the [`Histogram`].
    pub fn observe(mut self, x: f64) {
        let bucket = self.metadata().bucket(x);
        self.inner.get_mut().observe_mut(bucket, x);
    }

    /// Observe the duration in seconds
    pub fn observe_duration(self, duration: std::time::Duration) {
        self.observe(duration.as_secs_f64());
    }

    /// Observe the duration in seconds since the given instant
    pub fn observe_duration_since(self, since: std::time::Instant) -> std::time::Duration {
        let d = since.elapsed();
        self.observe_duration(d);
        d
    }
}

impl<const N: usize> Histogram<N> {
    /// Add a single observation to the [`Histogram`].
    pub fn observe(&self, x: f64) {
        self.get_metric().observe(x);
    }

    /// Create a [`HistogramTimer`] object that automatically observes a duration when the timer is dropped.
    pub fn start_timer(&self) -> HistogramTimer<'_, N> {
        HistogramTimer {
            vec: Some(self),
            start: std::time::Instant::now(),
        }
    }

    /// Observe the duration in seconds
    pub fn observe_duration(&self, duration: std::time::Duration) {
        self.observe(duration.as_secs_f64());
    }

    /// Observe the duration in seconds since the given instant
    pub fn observe_duration_since(&self, since: std::time::Instant) -> Duration {
        let d = since.elapsed();
        self.observe_duration(d);
        d
    }

    /// Take a consistent snapshot of the histogram.
    pub fn sample(&self) -> HistogramSample<N> {
        self.get_metric().sample()
    }
}

impl<L: LabelGroupSet, const N: usize> HistogramVec<L, N> {
    /// Add a single observation to the [`Histogram`], keyed by the label group.
    pub fn observe(&self, label: L::Group<'_>, y: f64) {
        self.get_metric(self.with_labels(label)).observe(y);
    }

    /// Create a [`HistogramVecTimer`] object that automatically observes a duration when the timer is dropped.
    ///
    /// # Panics
    /// Panics if the label group is not contained within the label set.
    pub fn start_timer(&self, label: L::Group<'_>) -> HistogramVecTimer<'_, L, N> {
        HistogramVecTimer {
            vec: Some(self),
            id: self.with_labels(label),
            start: std::time::Instant::now(),
        }
    }

    /// Observe the duration in seconds
    pub fn observe_duration(&self, label: L::Group<'_>, duration: std::time::Duration) {
        self.observe(label, duration.as_secs_f64());
    }

    /// Observe the duration in seconds since the given instant
    pub fn observe_duration_since(
        &self,
        label: L::Group<'_>,
        since: std::time::Instant,
    ) -> Duration {
        let d = since.elapsed();
        self.observe_duration(label, d);
        d
    }

    /// Take a consistent snapshot of the histogram for the label group.
    /// Returns an empty sample if the histogram has not been used yet.
    ///
    /// This does not create a new histogram for the label group.
    ///
    /// # Panics
    /// Panics if the label group is not contained within the label set.
    pub fn sample(&self, label: L::Group<'_>) -> HistogramSample<N> {
        self.find_metric(self.with_labels(label))
            .map_or(HistogramSample::default(), |m| m.sample())
    }
}

/// See [`HistogramVec::start_timer`]
pub struct HistogramVecTimer<'a, L: LabelGroupSet, const N: usize> {
    vec: Option<&'a HistogramVec<L, N>>,
    id: super::LabelId<L>,
    start: std::time::Instant,
}

impl<L: LabelGroupSet, const N: usize> HistogramVecTimer<'_, L, N> {
    /// Discard the timer, do not observe the duration.
    pub fn forget(mut self) {
        self.vec = None;
    }

    /// Stop the timer and record the duration since the timer was started in the histogram, in seconds.
    pub fn observe(mut self) -> Duration {
        let v = self.vec.take().unwrap();
        v.get_metric(self.id).observe_duration_since(self.start)
    }
}

impl<'a, L: LabelGroupSet, const N: usize> Drop for HistogramVecTimer<'a, L, N> {
    fn drop(&mut self) {
        if let Some(v) = self.vec {
            v.get_metric(self.id).observe_duration_since(self.start);
        }
    }
}

/// See [`Histogram::start_timer`]
pub struct HistogramTimer<'a, const N: usize> {
    vec: Option<&'a Histogram<N>>,
    start: std::time::Instant,
}

impl<'a, const N: usize> HistogramTimer<'a, N> {
    /// Discard the timer, do not observe the duration.
    pub fn forget(mut self) {
        self.vec = None;
    }

    /// Stop the timer and record the duration since the timer was started in the histogram, in seconds.
    pub fn observe(mut self) -> Duration {
        let v = self.vec.take().unwrap();
        v.get_metric().observe_duration_since(self.start)
    }
}

impl<'a, const N: usize> Drop for HistogramTimer<'a, N> {
    fn drop(&mut self) {
        if let Some(v) = self.vec {
            v.get_metric().observe_duration_since(self.start);
        }
    }
}

impl<const N: usize> MetricEncoding for HistogramState<N> {
    const KIND: MetricKind = MetricKind::Histogram;

    fn collect_into<E: Encoding>(
        &self,
        metadata: &Thresholds<N>,
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut E,
    ) -> Result<(), E::Err> {
        let sample = self.sample();
        enc.write_histogram(
            name,
            labels,
            metadata.get(),
            &sample.buckets,
            sample.inf,
            sample.sum,
            HistogramExtras::NONE,
        )
    }
}

/// The state of a gauge histogram, a distribution of values that can be added and removed.
///
/// For instance, the ages of items currently in a queue: an item is observed when it is added,
/// and removed when it leaves the queue.
pub struct GaugeHistogramState<const N: usize> {
    /// The read lock is acquired for observations. The write lock is acquired for sampling.
    inner: RwLock<GaugeHistogramStateInner<N>>,
}

struct GaugeHistogramStateInner<const N: usize> {
    buckets: [AtomicI64; N],
    inf: AtomicI64,
    sum: AtomicF64,
}

impl<const N: usize> Default for GaugeHistogramState<N> {
    fn default() -> Self {
        Self {
            inner: RwLock::new(GaugeHistogramStateInner {
                buckets: core::array::from_fn(|_| AtomicI64::new(0)),
                inf: AtomicI64::new(0),
                sum: AtomicF64::ZERO,
            }),
        }
    }
}

impl<const N: usize> MetricType for GaugeHistogramState<N> {
    type Metadata = Thresholds<N>;
}

/// A consistent snapshot of the state of a gauge histogram. See [`GaugeHistogramState::sample`].
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct GaugeHistogramSample<const N: usize> {
    /// The number of values in each bucket described by [`Thresholds`]. These are *not* cumulative.
    pub buckets: [i64; N],
    /// The number of values greater than all of the [`Thresholds`], including NaN.
    pub inf: i64,
    /// The sum of all values.
    pub sum: f64,
}

impl<const N: usize> Default for GaugeHistogramSample<N> {
    fn default() -> Self {
        Self {
            buckets: [0; N],
            inf: 0,
            sum: 0.0,
        }
    }
}

impl<const N: usize> GaugeHistogramSample<N> {
    /// The total number of values.
    pub fn count(&self) -> i64 {
        self.buckets.iter().sum::<i64>() + self.inf
    }
}

impl<const N: usize> GaugeHistogramState<N> {
    /// Take a consistent snapshot of the gauge histogram.
    ///
    /// This briefly takes the write lock, so it blocks concurrent observations
    /// until the snapshot is taken.
    pub fn sample(&self) -> GaugeHistogramSample<N> {
        let mut inner = self.inner.write();
        GaugeHistogramSample {
            buckets: core::array::from_fn(|i| *inner.buckets[i].get_mut()),
            inf: *inner.inf.get_mut(),
            sum: inner.sum.get_ex(),
        }
    }

    fn add(&self, thresholds: &Thresholds<N>, x: f64, n: i64) {
        let inner = self.inner.read();
        let bucket = thresholds.bucket(x);
        if bucket < N {
            inner.buckets[bucket].fetch_add(n, Ordering::Relaxed);
        } else {
            inner.inf.fetch_add(n, Ordering::Relaxed);
        }
        inner.sum.inc_by(x * n as f64);
    }
}

/// A shared ref to an individual gauge histogram
pub type GaugeHistogramLockGuard<'a, const N: usize> = MetricLockGuard<'a, GaugeHistogramState<N>>;

impl<const N: usize> GaugeHistogramLockGuard<'_, N> {
    /// Add a value to the gauge histogram
    pub fn observe(self, x: f64) {
        self.add(self.metadata(), x, 1);
    }

    /// Remove a value that was previously added with [`observe`](Self::observe)
    pub fn remove(self, x: f64) {
        self.add(self.metadata(), x, -1);
    }
}

impl<const N: usize> GaugeHistogram<N> {
    /// Add a value to the gauge histogram
    pub fn observe(&self, x: f64) {
        self.get_metric().observe(x);
    }

    /// Remove a value that was previously added with [`observe`](Self::observe)
    pub fn remove(&self, x: f64) {
        self.get_metric().remove(x);
    }

    /// Take a consistent snapshot of the gauge histogram.
    pub fn sample(&self) -> GaugeHistogramSample<N> {
        self.get_metric().sample()
    }
}

impl<L: LabelGroupSet, const N: usize> GaugeHistogramVec<L, N> {
    /// Add a value to the gauge histogram, keyed by the label group
    pub fn observe(&self, label: L::Group<'_>, x: f64) {
        self.get_metric(self.with_labels(label)).observe(x);
    }

    /// Remove a value that was previously added with [`observe`](Self::observe), keyed by the label group
    pub fn remove(&self, label: L::Group<'_>, x: f64) {
        self.get_metric(self.with_labels(label)).remove(x);
    }

    /// Take a consistent snapshot of the gauge histogram for the label group.
    /// Returns an empty sample if the gauge histogram has not been used yet.
    ///
    /// This does not create a new gauge histogram for the label group.
    ///
    /// # Panics
    /// Panics if the label group is not contained within the label set.
    pub fn sample(&self, label: L::Group<'_>) -> GaugeHistogramSample<N> {
        self.find_metric(self.with_labels(label))
            .map_or(GaugeHistogramSample::default(), |m| m.sample())
    }
}

impl<const N: usize> MetricEncoding for GaugeHistogramState<N> {
    const KIND: MetricKind = MetricKind::GaugeHistogram;

    fn collect_into<E: Encoding>(
        &self,
        metadata: &Thresholds<N>,
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut E,
    ) -> Result<(), E::Err> {
        let sample = self.sample();
        enc.write_gauge_histogram(
            name,
            labels,
            metadata.get(),
            &sample.buckets,
            sample.inf,
            sample.sum,
        )
    }
}
