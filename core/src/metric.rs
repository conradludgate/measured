use std::{hash::Hash, sync::RwLock};

use crate::label::{LabelGroup, LabelGroupSet, NoLabels};
use rustc_hash::FxHasher;

use self::name::MetricName;

pub mod counter;
pub mod histogram;
pub mod name;

type FxHashMap<K, V> = hashbrown::HashMap<K, V, BuildFxHasher>;

pub trait MetricType: Default {
    type Metadata: Sized;
}

pub struct MetricRef<'a, M: MetricType>(&'a M, &'a M::Metadata);

pub struct Metric<M: MetricType> {
    metric: M,
    metadata: M::Metadata,
}

pub struct MetricVec<M: MetricType, L: LabelGroupSet> {
    metrics: VecInner<L::Unique, M>,
    metadata: M::Metadata,
    label_set: L,
}

enum VecInner<U: Hash + Eq, M: MetricType> {
    Dense(Box<[M]>),
    Sparse(RwLock<FxHashMap<U, M>>),
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

impl<M: MetricType, L: LabelGroupSet> MetricVec<M, L> {
    pub fn new_metric_vec(label_set: L, metadata: M::Metadata) -> Self {
        let metrics = match label_set.cardinality() {
            Some(c) => {
                let mut vec = Vec::with_capacity(c);
                vec.resize_with(c, M::default);
                VecInner::Dense(vec.into_boxed_slice())
            }
            None => VecInner::Sparse(RwLock::new(FxHashMap::with_hasher(BuildFxHasher))),
        };

        Self {
            metrics,
            metadata,
            label_set,
        }
    }

    /// Create a new sparse metric vec. Useful if you have a fixed cardinality vec but the cardinality is quite high
    pub const fn new_sparse_metric_vec(label_set: L, metadata: M::Metadata) -> Self {
        Self {
            metrics: VecInner::Sparse(RwLock::new(FxHashMap::with_hasher(BuildFxHasher))),
            metadata,
            label_set,
        }
    }

    pub fn metadata(&self) -> &M::Metadata {
        &self.metadata
    }

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
                if let Some(m) = metrics.read().unwrap().get(&index) {
                    return f(MetricRef(m, &self.metadata));
                }

                let _ = metrics.write().unwrap().entry(index).or_default();

                let read = metrics.read().unwrap();
                let m = read.get(&index).unwrap();
                f(MetricRef(m, &self.metadata))
            }
        }
    }
}

pub trait MetricEncoder<T>: MetricType {
    fn write_type(name: impl MetricName, enc: &mut T);
    fn collect_into(
        &self,
        metadata: &Self::Metadata,
        labels: impl LabelGroup,
        name: impl MetricName,
        enc: &mut T,
    );
}

impl<M: MetricType> Metric<M> {
    pub fn collect_into<T>(&self, name: impl MetricName, enc: &mut T)
    where
        M: MetricEncoder<T>,
    {
        M::write_type(&name, enc);
        self.metric
            .collect_into(&self.metadata, NoLabels, name, enc);
    }
}

impl<M: MetricType, L: LabelGroupSet> MetricVec<M, L> {
    pub fn collect_into<T>(&self, name: impl MetricName, enc: &mut T)
    where
        M: MetricEncoder<T>,
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
                for (key, value) in &*m.read().unwrap() {
                    value.collect_into(&self.metadata, self.label_set.decode(key), &name, enc)
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
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl<L: LabelGroupSet> PartialEq for LabelId<L> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<L: LabelGroupSet> Eq for LabelId<L> {}

#[derive(Debug, Clone, Copy)]
struct BuildFxHasher;

impl std::hash::BuildHasher for BuildFxHasher {
    type Hasher = FxHasher;

    fn build_hasher(&self) -> FxHasher {
        FxHasher::default()
    }
}
