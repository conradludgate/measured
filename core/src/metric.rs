//! All about metrics

use core::hash::Hash;
use std::{
    hash::{BuildHasher, BuildHasherDefault},
    sync::OnceLock,
};

use crate::label::{LabelGroup, LabelGroupSet, NoLabels};
use hashbrown::HashMap;
use parking_lot::RwLockWriteGuard;
use rustc_hash::FxHasher;

use self::name::MetricNameEncoder;

pub mod counter;
pub mod gauge;
pub mod group;
pub mod histogram;
pub mod name;

/// Defines a metric
pub trait MetricType: Default {
    /// Some metrics require additional metadata
    type Metadata: Sized;
}

/// A shared ref to an individual metric value
pub struct MetricRef<'a, M: MetricType>(&'a M, &'a M::Metadata);

/// A unique ref to an individual metric value
pub struct MetricMut<'a, M: MetricType>(&'a mut M, &'a mut M::Metadata);

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
    Sparse(DashMap<U, M>),
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

    /// Get a mut ref to the metric
    pub fn get_metric_mut(&mut self) -> MetricMut<'_, M> {
        MetricMut(&mut self.metric, &mut self.metadata)
    }
}

// taken from dashmap
fn default_shard_amount() -> usize {
    static DEFAULT_SHARD_AMOUNT: OnceLock<usize> = OnceLock::new();
    *DEFAULT_SHARD_AMOUNT.get_or_init(|| {
        (std::thread::available_parallelism().map_or(1, usize::from) * 4).next_power_of_two()
    })
}

type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;
struct DashMap<K, V> {
    shards: Box<[parking_lot::RwLock<FxHashMap<K, V>>]>,
    shift: u32,
}

fn new_sparse<U: Hash + Eq, M: MetricType>() -> DashMap<U, M> {
    let shards = default_shard_amount();
    let mut vec = Vec::with_capacity(shards);
    vec.resize_with(shards, || {
        parking_lot::RwLock::new(HashMap::with_hasher(BuildHasherDefault::default()))
    });
    DashMap {
        shards: vec.into_boxed_slice(),
        shift: (std::mem::size_of::<usize>() * 8) as u32 - shards.trailing_zeros(),
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
            None => VecInner::Sparse(new_sparse()),
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
            metrics: VecInner::Sparse(new_sparse()),
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
        let id = self.label_set.encode(label)?;

        let hash = match &self.metrics {
            VecInner::Dense(metrics) => {
                let index = self.label_set.encode_dense(id).expect("If the label set is fixed in cardinality, it must return a value here in the range of `0..cardinality`");
                debug_assert!(index < metrics.len());
                index as u64
            }
            VecInner::Sparse(_) => BuildHasherDefault::<FxHasher>::default().hash_one(id),
        };

        Some(LabelId { id, hash })
    }

    /// Get the individual metric at the given identifier.
    ///
    /// # Panics
    /// Can panic of cause strange behaviour if the label ID comes from a different metric family.
    pub fn get_metric<R>(
        &self,
        id: LabelId<L>,
        f: impl for<'a> FnOnce(MetricRef<'a, M>) -> R,
    ) -> R {
        match &self.metrics {
            VecInner::Dense(metrics) => {
                let m = &metrics[id.hash as usize];
                f(MetricRef(m, &self.metadata))
            }
            VecInner::Sparse(metrics) => {
                let shard = &metrics.shards[((id.hash as usize) << 7) >> metrics.shift];

                if let Some((_, v)) = shard.read().raw_table().get(id.hash, |(k, _v)| *k == id.id) {
                    return f(MetricRef(v, &self.metadata));
                }

                let shard = {
                    let mut shard = shard.write();
                    let entry = shard.raw_entry_mut().from_hash(id.hash, |k| *k == id.id);
                    match entry {
                        hashbrown::hash_map::RawEntryMut::Occupied(_) => {}
                        hashbrown::hash_map::RawEntryMut::Vacant(v) => {
                            v.insert_hashed_nocheck(id.hash, id.id, M::default());
                        }
                    }
                    RwLockWriteGuard::downgrade(shard)
                };

                let (_, v) = shard
                    .raw_table()
                    .get(id.hash, |(k, _v)| *k == id.id)
                    .expect("it was just inserted");

                f(MetricRef(v, &self.metadata))
            }
        }
    }

    /// Get the individual metric at the given identifier.
    ///
    /// # Panics
    /// Can panic of cause strange behaviour if the label ID comes from a different metric family.
    pub fn get_metric_mut(&mut self, id: LabelId<L>) -> MetricMut<M> {
        match &mut self.metrics {
            VecInner::Dense(metrics) => {
                let m = &mut metrics[id.hash as usize];
                MetricMut(m, &mut self.metadata)
            }
            VecInner::Sparse(metrics) => {
                let shard = &mut metrics.shards[((id.hash as usize) << 7) >> metrics.shift];

                let (_, v) = shard
                    .get_mut()
                    .raw_entry_mut()
                    .from_hash(id.hash, |k| *k == id.id)
                    .or_insert_with(|| (id.id, M::default()));

                MetricMut(v, &mut self.metadata)
            }
        }
    }

    /// Inspect the current cardinality of this metric-vec, returning the lower bound and the upper bound if known
    pub fn get_cardinality(&self) -> (usize, Option<usize>) {
        match &self.metrics {
            VecInner::Dense(x) => (x.len(), Some(x.len())),
            VecInner::Sparse(x) => (
                x.shards
                    .iter()
                    .map(|shard| shard.read().len())
                    .sum::<usize>(),
                self.label_set.cardinality(),
            ),
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
    fn write_type(name: impl MetricNameEncoder, enc: &mut T);
    /// Sample this metric into the encoder
    fn collect_into(
        &self,
        metadata: &Self::Metadata,
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut T,
    );
}

pub trait MetricFamilyEncoding<T> {
    /// Collect these metric values into the given encoder with the given metric name
    fn collect_into(&self, name: impl MetricNameEncoder, enc: &mut T);
}

impl<M: MetricEncoding<T>, T> MetricFamilyEncoding<T> for Metric<M> {
    /// Collect this metric value into the given encoder with the given metric name
    fn collect_into(&self, name: impl MetricNameEncoder, enc: &mut T) {
        M::write_type(&name, enc);
        self.metric
            .collect_into(&self.metadata, NoLabels, name, enc);
    }
}

impl<M: MetricEncoding<T>, L: LabelGroupSet, T> MetricFamilyEncoding<T> for MetricVec<M, L> {
    fn collect_into(&self, name: impl MetricNameEncoder, enc: &mut T) {
        M::write_type(&name, enc);
        match &self.metrics {
            VecInner::Dense(m) => {
                for (index, value) in m.iter().enumerate() {
                    value.collect_into(
                        &self.metadata,
                        self.label_set.decode_dense(index),
                        &name,
                        enc,
                    );
                }
            }
            VecInner::Sparse(m) => {
                for shard in m.shards.iter() {
                    for (k, v) in shard.read().iter() {
                        v.collect_into(&self.metadata, self.label_set.decode(k), &name, enc);
                    }
                }
            }
        }
    }
}

pub struct LabelId<L: LabelGroupSet> {
    id: L::Unique,
    hash: u64,
}

impl<L: LabelGroupSet> Clone for LabelId<L> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<L: LabelGroupSet> Copy for LabelId<L> {}
impl<L: LabelGroupSet> PartialEq for LabelId<L> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<L: LabelGroupSet> Eq for LabelId<L> {}
