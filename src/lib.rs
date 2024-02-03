use std::sync::atomic::AtomicU64;

use dashmap::DashMap;
use label::LabelGroupSet;

pub mod label;

#[derive(Default)]
pub struct Counter {
    count: AtomicU64,
}

pub type CounterRef<'a> = MetricRef<'a, Counter>;

impl CounterRef<'_> {
    pub fn inc(self) {
        self.0
            .count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn inc_by(self, x: u64) {
        self.0
            .count
            .fetch_add(x, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Metric for Counter {
    type Metadata = ();
}

pub struct Histogram<const N: usize> {
    buckets: [AtomicU64; N],
    count: AtomicU64,
    sum: AtomicU64,
}

pub type HistogramRef<'a, const N: usize> = MetricRef<'a, Histogram<N>>;

impl<const N: usize> Default for Histogram<N> {
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

impl<const N: usize> Metric for Histogram<N> {
    type Metadata = Thresholds<N>;
}

pub struct Thresholds<const N: usize> {
    le: [f64; N],
}

impl<const N: usize> HistogramRef<'_, N> {
    pub fn observe(self, x: f64) {
        for i in 0..N {
            if x < self.1.le[i] {
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

pub trait Metric: Default {
    type Metadata: Sized;
}

pub struct MetricVec<M: Metric, L: label::LabelGroupSet> {
    metrics: DashMap<L::Unique, M>,
    metadata: M::Metadata,
    label_set: L,
}

impl<M: Metric, L: label::LabelGroupSet> MetricVec<M, L> {
    pub fn new_metric_vec(label_set: L, metadata: M::Metadata) -> Self {
        Self {
            metrics: DashMap::new(),
            metadata,
            label_set,
        }
    }

    pub fn metadata(&self) -> &M::Metadata {
        &self.metadata
    }

    pub fn with_labels<'a>(&'a self, label: L::Group<'a>) -> Option<LabelId<L>> {
        Some(LabelId(self.label_set.encode(label)?))
    }

    pub fn get_metric<R>(
        &self,
        id: LabelId<L>,
        f: impl for<'a> FnOnce(MetricRef<'a, M>) -> R,
    ) -> R {
        let index = id.0;
        let m = self
            .metrics
            .get(&index)
            .unwrap_or_else(|| self.metrics.entry(index).or_default().downgrade());

        f(MetricRef(&m, &self.metadata))
    }
}

pub type CounterVec<L> = MetricVec<Counter, L>;
impl<L: label::LabelGroupSet> MetricVec<Counter, L> {
    pub fn new_counter_vec(label_set: L) -> Self {
        Self::new_metric_vec(label_set, ())
    }
}

pub struct MetricRef<'a, M: Metric>(&'a M, &'a M::Metadata);

pub struct LabelId<L: LabelGroupSet>(L::Unique);
