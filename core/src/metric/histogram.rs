use std::sync::atomic::AtomicU64;

use crate::{label::LabelGroupSet, HistogramVec};

use super::{MetricRef, MetricType};

pub struct HistogramState<const N: usize> {
    pub buckets: [AtomicU64; N],
    pub count: AtomicU64,
    pub sum: AtomicU64,
}

pub type HistogramRef<'a, const N: usize> = MetricRef<'a, HistogramState<N>>;

impl<const N: usize> Default for HistogramState<N> {
    fn default() -> Self {
        #[allow(clippy::declare_interior_mutable_const)]
        const ZERO: AtomicU64 = AtomicU64::new(0);
        Self {
            buckets: [ZERO; N],
            count: ZERO,
            sum: AtomicU64::new(f64::to_bits(0.0)),
        }
    }
}

impl<const N: usize> MetricType for HistogramState<N> {
    type Metadata = Thresholds<N>;
}

pub struct Thresholds<const N: usize> {
    le: [f64; N],
}
impl<const N: usize> Thresholds<N> {
    pub fn exponential_buckets(start: f64, factor: f64) -> Self {
        if start <= 0.0 {
            panic!(
                "exponential_buckets needs a positive start value, \
                 start: {start}",
            );
        }
        if factor <= 1.0 {
            panic!(
                "exponential_buckets needs a factor greater than 1, \
                 factor: {factor}",
            );
        }

        let mut next = start;
        let buckets = std::array::from_fn(|_| {
            let x = next;
            next *= factor;
            x
        });

        Thresholds { le: buckets }
    }

    pub fn get(&self) -> &[f64; N] {
        &self.le
    }
}

impl<const N: usize> HistogramRef<'_, N> {
    pub fn observe(self, x: f64) {
        for i in 0..N {
            if x <= self.1.le[i] {
                self.0.buckets[i].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
        self.0
            .count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.0
            .sum
            .fetch_update(
                std::sync::atomic::Ordering::Release,
                std::sync::atomic::Ordering::Acquire,
                |y| Some(f64::to_bits(f64::from_bits(y) + x)),
            )
            .expect("we always return Some in fetch_update");
    }
}

impl<L: LabelGroupSet, const N: usize> HistogramVec<L, N> {
    pub fn observe(&self, label: L::Group<'_>, y: f64) {
        self.get_metric(
            self.with_labels(label)
                .expect("label group should be in the set"),
            |x| x.observe(y),
        )
    }
}
