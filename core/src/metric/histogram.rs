//! All things histograms. See [`Histogram`]

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use parking_lot::RwLock;

use super::{gauge::AtomicF64, MetricLockGuard, MetricMut, MetricType};
use crate::{label::LabelGroupSet, Histogram, HistogramVec};

/// The inner state of a histogram.
///
/// A histogram is comprised of 'buckets' where each bucket tracks a range or possible observations.
/// For instance, an observation of 1.5 would increment a counter in the bucket `1.0..2.0`.
/// If there is no suitable bucket, 'inf' is incremented.
pub struct HistogramStateInner<const N: usize> {
    /// The buckets count the number of observed values in the ranges described by [`Thresholds`]
    pub buckets: [AtomicU64; N],
    /// The number of observed values that are greater than described by [`Thresholds`]
    pub inf: AtomicU64,
    /// The accumulated sum
    pub sum: AtomicF64,
}

impl<const N: usize> HistogramStateInner<N> {
    /// Add a single observation to the [`Histogram`].
    pub fn observe(&self, bucket: usize, x: f64) {
        assert!(bucket <= N);
        if bucket < N {
            self.buckets[bucket].fetch_add(1, Ordering::Relaxed);
        } else {
            self.inf.fetch_add(1, Ordering::Relaxed);
        }
        self.sum.inc_by(x);
    }

    /// Add a single observation to the [`Histogram`].
    pub fn observe_mut(&mut self, bucket: usize, x: f64) {
        assert!(bucket <= N);
        if bucket < N {
            *self.buckets[bucket].get_mut() += 1;
        } else {
            *self.inf.get_mut() += 1;
        }
        let v = self.sum.get_ex();
        self.sum.set_mut(v + x);
    }

    pub(crate) fn sample(&mut self) -> ([u64; N], u64, f64) {
        let mut output = [0; N];
        #[allow(clippy::needless_range_loop)]
        for i in 0..N {
            output[i] = *self.buckets[i].get_mut();
        }
        (output, *self.inf.get_mut(), self.sum.get_ex())
    }
}

/// The state of a histogram. See also [`HistogramStateInner`]
pub struct HistogramState<const N: usize> {
    /// A rwlock over the inner histogram state.
    /// The read lock is acquired for observations.
    /// The write lock is acquired for sampling.
    pub inner: RwLock<HistogramStateInner<N>>,
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
pub struct Thresholds<const N: usize> {
    le: [f64; N],
}

impl<const N: usize> Thresholds<N> {
    /// Create `N` buckets, where the lowest bucket has an upper bound of `start` and each following bucket’s upper bound is `factor` times the previous bucket’s upper bound.
    /// The final +Inf bucket is not counted and not included.
    ///
    /// # Panics
    /// The function panics if `start` is zero or negative, or if `factor` is less than or equal 1.
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

        Thresholds { le: buckets }
    }

    /// Create `N` buckets, each `width`  wide, where the lowest bucket has an upper bound of `start`.
    /// The final +Inf bucket is not counted and not included.
    ///
    /// # Panics
    /// The function panics `width` is zero or negative.
    pub fn linear_buckets(start: f64, width: f64) -> Self {
        assert!(
            width > 0.0,
            "linear_buckets needs a width greater than 0, width: {width}",
        );

        let buckets = core::array::from_fn(|i| start + width * i as f64);

        Thresholds { le: buckets }
    }

    /// Create the histogram thresholds with the given sizes
    ///
    /// # Panics
    /// Will panic if the buckets are not strictly monotonically increasing
    pub fn with_buckets(buckets: [f64; N]) -> Self {
        for i in 0..N - 1 {
            assert!(
                buckets[i] < buckets[i + 1],
                "consecutive histogram buckets must not decrease or be equal",
            );
        }
        Thresholds { le: buckets }
    }

    /// View the bucket upper bounds
    pub fn get(&self) -> &[f64; N] {
        &self.le
    }
}

impl<const N: usize> HistogramLockGuard<'_, N> {
    /// Add a single observation to the [`Histogram`].
    pub fn observe(self, x: f64) {
        let bucket = self.metadata().le.partition_point(|le| x > *le);
        self.inner.read().observe(bucket, x);
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
        let bucket = self.metadata().le.partition_point(|le| x > *le);
        self.inner.get_mut().observe(bucket, x);
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

    /// Create a [`HistogramVecTimer`] object that automatically observes a duration when the timer is dropped.
    pub fn start_timer(&self) -> HistogramTimer<'_, N> {
        HistogramTimer {
            vec: Some(self),
            start: std::time::Instant::now(),
        }
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
}

/// See [`HistogramVec::start_timer`]
pub struct HistogramVecTimer<'a, L: LabelGroupSet, const N: usize> {
    vec: Option<&'a HistogramVec<L, N>>,
    id: super::LabelId<L>,
    start: std::time::Instant,
}

impl<'a, L: LabelGroupSet, const N: usize> HistogramVecTimer<'a, L, N> {
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
