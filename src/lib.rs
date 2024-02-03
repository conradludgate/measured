use std::{
    hash::{BuildHasherDefault, Hash},
    sync::atomic::AtomicU64,
};

use dashmap::DashMap;
use label::{LabelGroup, LabelGroupSet, LabelVisitor, NoLabels};
use rustc_hash::FxHasher;
use text::{Bucket, Count, MetricName, Sum, TextEncoder};

pub mod label;
pub mod text;

#[derive(Default)]
pub struct CounterState {
    count: AtomicU64,
}

pub type CounterRef<'a> = MetricRef<'a, CounterState>;

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

impl MetricType for CounterState {
    type Metadata = ();
}

pub struct HistogramState<const N: usize> {
    buckets: [AtomicU64; N],
    count: AtomicU64,
    sum: AtomicU64,
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
        let mut buckets = std::array::from_fn(|_| {
            let x = next;
            next *= factor;
            x
        });
        buckets[N - 1] = f64::INFINITY;

        Thresholds { le: buckets }
    }
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

pub trait MetricType: Default {
    type Metadata: Sized;
}

pub struct Metric<M: MetricType> {
    metric: M,
    metadata: M::Metadata,
}

impl<M: MetricType> Metric<M> {
    pub fn new_metric(metadata: M::Metadata) -> Self {
        Self {
            metric: M::default(),
            metadata,
        }
    }

    pub fn get_metric(&self) -> MetricRef<'_, M> {
        MetricRef(&self.metric, &self.metadata)
    }
}

pub struct MetricVec<M: MetricType, L: label::LabelGroupSet> {
    metrics: VecInner<L::Unique, M>,
    metadata: M::Metadata,
    label_set: L,
}

enum VecInner<U: Hash + Eq, M: MetricType> {
    Dense(Box<[M]>),
    Sparse(DashMap<U, M, BuildHasherDefault<FxHasher>>),
}

impl<M: MetricType, L: label::LabelGroupSet> MetricVec<M, L> {
    pub fn new_metric_vec(label_set: L, metadata: M::Metadata) -> Self {
        let metrics = match label_set.cardinality() {
            Some(c) => {
                let mut vec = Vec::with_capacity(c);
                vec.resize_with(c, M::default);
                VecInner::Dense(vec.into_boxed_slice())
            }
            None => VecInner::Sparse(DashMap::with_hasher(BuildHasherDefault::default())),
        };

        Self {
            metrics,
            metadata,
            label_set,
        }
    }

    /// Create a new sparse metric vec. Useful if you have a fixed cardinality vec but the cardinality is quite high
    pub fn new_sparse_metric_vec(label_set: L, metadata: M::Metadata) -> Self {
        Self {
            metrics: VecInner::Sparse(DashMap::with_hasher(BuildHasherDefault::default())),
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
        match &self.metrics {
            VecInner::Dense(metrics) => {
                let m = &metrics[self.label_set.encode_dense(index).unwrap()];
                f(MetricRef(m, &self.metadata))
            }
            VecInner::Sparse(metrics) => {
                let m = metrics
                    .get(&index)
                    .unwrap_or_else(|| metrics.entry(index).or_default().downgrade());

                f(MetricRef(&m, &self.metadata))
            }
        }
    }
}

pub type Histogram<const N: usize> = Metric<HistogramState<N>>;
pub type HistogramVec<L, const N: usize> = MetricVec<HistogramState<N>, L>;
pub type Counter = Metric<CounterState>;
pub type CounterVec<L> = MetricVec<CounterState, L>;
impl<L: label::LabelGroupSet> MetricVec<CounterState, L> {
    pub fn new_counter_vec(label_set: L) -> Self {
        Self::new_metric_vec(label_set, ())
    }
    pub fn new_sparse_counter_vec(label_set: L) -> Self {
        Self::new_sparse_metric_vec(label_set, ())
    }
}

pub struct MetricRef<'a, M: MetricType>(&'a M, &'a M::Metadata);

pub struct LabelId<L: LabelGroupSet>(L::Unique);

// pub trait Collect<Encoder> {

// }

struct HistogramLabelLe {
    le: f64,
}

impl LabelGroup for HistogramLabelLe {
    fn label_names() -> impl IntoIterator<Item = &'static str> {
        std::iter::once("le")
    }

    fn label_values(&self, v: &mut impl LabelVisitor) {
        v.write_float(self.le)
    }
}

impl<const N: usize> Histogram<N> {
    pub fn collect_into(&self, name: impl MetricName, enc: &mut TextEncoder) {
        enc.write_type(&name, text::MetricType::Histogram);
        for i in 0..N {
            let le = self.metadata.le[i];
            let val = &self.metric.buckets[i];
            enc.write_metric(
                &(&name).with_suffix(Bucket),
                NoLabels.compose_with(HistogramLabelLe { le }),
                text::MetricValue::Int(val.load(std::sync::atomic::Ordering::Relaxed) as i64),
            );
        }
        enc.write_metric(
            &(&name).with_suffix(Sum),
            NoLabels,
            text::MetricValue::Float(f64::from_bits(
                self.metric.sum.load(std::sync::atomic::Ordering::Relaxed),
            )),
        );
        enc.write_metric(
            &(&name).with_suffix(Count),
            NoLabels,
            text::MetricValue::Int(
                self.metric.count.load(std::sync::atomic::Ordering::Relaxed) as i64
            ),
        );
    }
}

impl Counter {
    pub fn collect_into(&self, name: impl MetricName, enc: &mut TextEncoder) {
        enc.write_type(&name, text::MetricType::Counter);
        enc.write_metric(
            &name,
            NoLabels,
            text::MetricValue::Int(
                self.metric.count.load(std::sync::atomic::Ordering::Relaxed) as i64
            ),
        );
    }
}

impl<L: LabelGroupSet> CounterVec<L> {
    pub fn collect_into(&self, name: impl MetricName, enc: &mut TextEncoder) {
        enc.write_type(&name, text::MetricType::Counter);
        match &self.metrics {
            VecInner::Dense(m) => {
                for (index, value) in m.iter().enumerate() {
                    enc.write_metric(
                        &name,
                        self.label_set.decode_dense(index),
                        text::MetricValue::Int(
                            value.count.load(std::sync::atomic::Ordering::Relaxed) as i64,
                        ),
                    );
                }
            }
            VecInner::Sparse(m) => {
                for values in m {
                    enc.write_metric(
                        &name,
                        self.label_set.decode(values.key()),
                        text::MetricValue::Int(
                            values
                                .value()
                                .count
                                .load(std::sync::atomic::Ordering::Relaxed)
                                as i64,
                        ),
                    );
                }
            }
        }
    }
}
