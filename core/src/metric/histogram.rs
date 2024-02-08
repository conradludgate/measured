use std::sync::Mutex;

use super::{MetricRef, MetricType};
use crate::{label::LabelGroupSet, Histogram, HistogramVec};

#[derive(Clone, Copy)]
pub struct HistogramStateMut<const N: usize> {
    pub buckets: [u64; N],
    pub count: u64,
    pub sum: f64,
}

pub struct HistogramState<const N: usize> {
    pub inner: Mutex<HistogramStateMut<N>>,
}

pub type HistogramRef<'a, const N: usize> = MetricRef<'a, HistogramState<N>>;

impl<const N: usize> Default for HistogramState<N> {
    fn default() -> Self {
        Self {
            inner: Mutex::new(HistogramStateMut {
                buckets: [0; N],
                count: 0,
                sum: 0.0,
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
            "linear_buckets needs a width greate than 0, width: {width}",
        );

        let buckets = core::array::from_fn(|i| start + width * i as f64);

        Thresholds { le: buckets }
    }

    /// View the bucket upper bounds
    pub fn get(&self) -> &[f64; N] {
        &self.le
    }
}

impl<const N: usize> MetricRef<'_, HistogramState<N>> {
    /// Add a single observation to the [`Histogram`].
    pub fn observe(self, x: f64) {
        // TODO: does this SIMD?
        let increase: [u64; N] = std::array::from_fn(|i| if x <= self.1.le[i] { 1 } else { 0 });
        let mut inner = self.0.inner.lock().unwrap();
        for (x, y) in std::iter::zip(&mut inner.buckets, increase) {
            *x += y;
        }
        inner.count += 1;
        inner.sum += x;
    }

    /// Observe the duration in seconds since the given instant
    pub fn observe_duration_since(self, since: std::time::Instant) {
        self.observe(since.elapsed().as_secs_f64());
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
        self.get_metric(
            self.with_labels(label)
                .expect("label group should be in the set"),
            |x| x.observe(y),
        );
    }

    /// Create a [`HistogramVecTimer`] object that automatically observes a duration when the timer is dropped.
    pub fn start_timer(&self, label: L::Group<'_>) -> Option<HistogramVecTimer<'_, L, N>> {
        Some(HistogramVecTimer {
            vec: Some(self),
            id: self.with_labels(label)?,
            start: std::time::Instant::now(),
        })
    }

    /// Observe the duration in seconds since the given instant
    pub fn observe_duration_since(self, label: L::Group<'_>, since: std::time::Instant) {
        self.observe(label, since.elapsed().as_secs_f64());
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
}

impl<'a, L: LabelGroupSet, const N: usize> Drop for HistogramVecTimer<'a, L, N> {
    fn drop(&mut self) {
        if let Some(v) = self.vec {
            v.get_metric(self.id, |m| m.observe_duration_since(self.start));
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
}

impl<'a, const N: usize> Drop for HistogramTimer<'a, N> {
    fn drop(&mut self) {
        if let Some(v) = self.vec {
            v.get_metric().observe_duration_since(self.start);
        }
    }
}
