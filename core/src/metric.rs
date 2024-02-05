//! All about metrics

use core::hash::Hash;

use crate::label::{LabelGroup, LabelGroupSet, NoLabels};
use rustc_hash::FxHasher;

use self::name::MetricName;

pub mod counter;
pub mod histogram;
pub mod name;

type BuildFxHasher = core::hash::BuildHasherDefault<FxHasher>;
type FxDashMap<K, V> = dashmap::DashMap<K, V, BuildFxHasher>;

/// Defines a metric
pub trait MetricType: Default {
    /// Some metrics require additional metadata
    type Metadata: Sized;
}

/// A shared ref to an individual metric value
pub struct MetricRef<'a, M: MetricType>(&'a M, &'a M::Metadata);

/// A single metric value.
pub struct Metric<M: MetricType> {
    metric: M,
    metadata: M::Metadata,
}

/// Multiple metric values, keyed by [`LabelGroup`]
pub struct MetricVec<M: MetricType, L: LabelGroupSet> {
    metrics: VecInner<L::Unique, M>,
    metadata: M::Metadata,
    label_set: L,
}

enum VecInner<U: Hash + Eq, M: MetricType> {
    Dense(Box<[M]>),
    Sparse(FxDashMap<U, M>),
}

impl<M: MetricType> Metric<M> {
    /// Create a new metric with the given metadata
    pub fn new_metric(metadata: M::Metadata) -> Self {
        Self {
            metric: M::default(),
            metadata,
        }
    }

    /// Get a ref to the metric
    pub fn get_metric(&self) -> MetricRef<'_, M> {
        MetricRef(&self.metric, &self.metadata)
    }
}

impl<M: MetricType, L: LabelGroupSet> MetricVec<M, L> {
    /// Create a new metric vec with the given label set and metric metadata
    pub fn new_metric_vec(label_set: L, metadata: M::Metadata) -> Self {
        let metrics = match label_set.cardinality() {
            Some(c) => {
                let mut vec = Vec::with_capacity(c);
                vec.resize_with(c, M::default);
                VecInner::Dense(vec.into_boxed_slice())
            }
            None => VecInner::Sparse(FxDashMap::with_hasher(BuildFxHasher::default())),
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
            metrics: VecInner::Sparse(FxDashMap::with_hasher(BuildFxHasher::default())),
            metadata,
            label_set,
        }
    }

    /// View the metric metadata
    pub fn metadata(&self) -> &M::Metadata {
        &self.metadata
    }

    /// Get an identifier for the specific metric identified by this label group
    pub fn with_labels(&self, label: L::Group<'_>) -> Option<LabelId<L>> {
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
                if let Some(m) = metrics.get(&index) {
                    return f(MetricRef(m.value(), &self.metadata));
                }

                let m = metrics.entry(index).or_default().downgrade();
                f(MetricRef(m.value(), &self.metadata))
            }
        }
    }

    /// Inspect the current cardinality of this metric-vec, returning the lower bound and the upper bound if known
    pub fn get_cardinality(&self) -> (usize, Option<usize>) {
        match &self.metrics {
            VecInner::Dense(x) => (x.len(), Some(x.len())),
            VecInner::Sparse(x) => (x.len(), self.label_set.cardinality()),
        }
    }

    /// Borrow the label set values
    pub fn get_label_set(&self) -> &L {
        &self.label_set
    }
}

/// Defines the encoding of a metric
pub trait MetricEncoding<T>: MetricType {
    /// Write the type information for this metric into the encoder
    fn write_type(name: impl MetricName, enc: &mut T);
    /// Sample this metric into the encoder
    fn collect_into(
        &self,
        metadata: &Self::Metadata,
        labels: impl LabelGroup,
        name: impl MetricName,
        enc: &mut T,
    );
}

impl<M: MetricType> Metric<M> {
    /// Collect this metric value into the given encoder with the given metric name
    pub fn collect_into<T>(&self, name: impl MetricName, enc: &mut T)
    where
        M: MetricEncoding<T>,
    {
        M::write_type(&name, enc);
        self.metric
            .collect_into(&self.metadata, NoLabels, name, enc);
    }
}

impl<M: MetricType, L: LabelGroupSet> MetricVec<M, L> {
    /// Collect these metric values into the given encoder with the given metric name
    pub fn collect_into<T>(&self, name: impl MetricName, enc: &mut T)
    where
        M: MetricEncoding<T>,
    {
        M::write_type(&name, enc);
        match &self.metrics {
            VecInner::Dense(m) => {
                for (index, value) in m.iter().enumerate() {
                    value.collect_into(
                        &self.metadata,
                        self.label_set.decode_dense(index),
                        &name,
                        enc,
                    )
                }
            }
            VecInner::Sparse(m) => {
                for values in m {
                    values.value().collect_into(
                        &self.metadata,
                        self.label_set.decode(values.key()),
                        &name,
                        enc,
                    )
                }
            }
        }
    }
}

pub struct LabelId<L: LabelGroupSet>(L::Unique);

impl<L: LabelGroupSet> Clone for LabelId<L> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<L: LabelGroupSet> Copy for LabelId<L> {}
impl<L: LabelGroupSet> Hash for LabelId<L> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl<L: LabelGroupSet> PartialEq for LabelId<L> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<L: LabelGroupSet> Eq for LabelId<L> {}
