use std::sync::atomic::AtomicU64;

use dashmap::{mapref::one::Ref, DashMap};

pub mod label;

#[derive(Default)]
pub struct Counter {
    count: AtomicU64,
}

impl Counter {
    pub fn inc(&self) {
        self.count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn inc_by(&self, x: u64) {
        self.count
            .fetch_add(x, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Metric for Counter {
    type Metadata = ();
}

pub struct Histogram<const N: usize> {
    buckets: [AtomicU64; N],
    count: AtomicU64,
}

impl<const N: usize> Default for Histogram<N> {
    fn default() -> Self {
        #[allow(clippy::declare_interior_mutable_const)]
        const ZERO: AtomicU64 = AtomicU64::new(0);
        Self {
            buckets: [ZERO; N],
            count: ZERO,
        }
    }
}

impl<const N: usize> Metric for Histogram<N> {
    type Metadata = Thresholds<N>;
}

pub struct Thresholds<const N: usize> {
    le: [f64; N],
}

impl<const N: usize> Histogram<N> {
    pub fn observe(&self, x: f64, metadata: Thresholds<N>) {
        for i in 0..N {
            if x < metadata.le[i] {
                self.buckets[i].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
        self.count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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

    pub fn with_labels<'a>(&'a self, label: L::Group<'a>) -> Ref<'a, L::Unique, M> {
        let index = self.label_set.encode(label);
        if let Some(m) = self.metrics.get(&index) {
            return m;
        }
        self.metrics.entry(index).or_default().downgrade()
    }
}

pub type CounterVec<L> = MetricVec<Counter, L>;
impl<L: label::LabelGroupSet> MetricVec<Counter, L> {
    pub fn new_counter_vec(label_set: L) -> Self {
        Self::new_metric_vec(label_set, ())
    }
}
