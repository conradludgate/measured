//! All about metrics

use core::hash::Hash;
use std::{
    hash::BuildHasher,
    ops::{Deref, DerefMut},
    sync::OnceLock,
};

use crate::label::{LabelGroup, LabelGroupSet, NoLabels};
use crossbeam_utils::CachePadded;

use self::{group::Encoding, name::MetricNameEncoder};

pub mod counter;
pub mod gauge;
pub mod group;
pub mod histogram;
pub mod name;
mod sparse;

/// Defines a metric
pub trait MetricType: Default {
    /// Some metrics require additional metadata
    type Metadata: Sized;
}

/// A shared ref to an individual metric value.
///
/// As the name implies, it might hold a lock (only applicable to sparse metric vecs, but this is not guaranteed behaviour)
pub struct MetricLockGuard<'a, M: MetricType>(MetricLockGuardRepr<'a, M>, &'a M::Metadata);

enum MetricLockGuardRepr<'a, M> {
    Dense(&'a M),
    Sparse(sparse::SparseLockGuard<'a, M>),
}

impl<M: MetricType> Deref for MetricLockGuard<'_, M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        match self.0 {
            MetricLockGuardRepr::Dense(d) => d,
            MetricLockGuardRepr::Sparse(ref s) => s,
        }
    }
}

impl<M: MetricType> MetricLockGuard<'_, M> {
    pub fn metadata(&self) -> &M::Metadata {
        self.1
    }
}

/// A unique ref to an individual metric value
pub struct MetricMut<'a, M: MetricType>(&'a mut M, &'a M::Metadata);

impl<M: MetricType> Deref for MetricMut<'_, M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<M: MetricType> DerefMut for MetricMut<'_, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<M: MetricType> MetricMut<'_, M> {
    pub fn metadata(&self) -> &M::Metadata {
        self.1
    }
}

/// A single metric value.
pub struct Metric<M: MetricType> {
    metric: M,
    metadata: M::Metadata,
}

/// Multiple metric values, keyed by [`LabelGroup`]
///
/// # Note
///
/// The internal representation of the metric vec can either be 'dense' or 'sparse'.
/// We will try to pick the most sensible representation unless you
/// specify with one of the `dense_*` or `sparse_*` methods.
///
/// The default representation is not considered stable, and will change in future if
/// it is measured beneficial for performance or memory usage.
///
/// ## Dense
///
/// Dense metric vecs are represented as a single `Vec<_>` with lazily initiated cells.
/// The vec is pre-allocated to match the [`LabelGroupSet::cardinality`].
/// Each cell is [`CachePadded`] to improve performance, but induces higher memory overhead.
///
/// The [`MetricVec::remove_metric`] API does not work with dense representations as it introduced too much performance overhead.
/// Since removal would not reduce memory usage of a dense metric anyway, I do not consider if a priority to support. I suggest switching to
/// a sparse representation if this is necessary for your use case.
/// Please open an issue if you absolutely need removal paired with the dense repr metric vec.
///
/// This is currently the default if the label set cardinality is <= 1024
///
/// ## Sparse
///
/// Sparse metric vecs are represented as a sharded hashmap. Because the implementation makes use of [`RwLock`]s, it does have
/// slower performance compared to the dense representation.
///
/// Currently the number of shards used is taken as the number of CPU cores, multiplied by 4 and rounded up to the next power of 2.
/// This was chosen based on [`dashmap`](https://docs.rs/dashmap/latest/src/dashmap/lib.rs.html#66-71), which is used to reduce lock contention
/// on each shard. This is not considered stable.
///
/// This is currently the default if the label set cardinality is > 1024, or unbounded.
pub struct MetricVec<M: MetricType, L: LabelGroupSet> {
    metrics: VecInner<L::Unique, M>,
    metadata: M::Metadata,
    label_set: L,
}

enum VecInner<U: Hash + Eq, M: MetricType> {
    Dense(Box<[CachePadded<OnceLock<M>>]>),
    Sparse(sparse::ShardedMap<U, M>),
}

impl<M: MetricType> Metric<M>
where
    M::Metadata: Default,
{
    /// Create a new metric
    pub fn new() -> Self {
        Self::with_metadata(<M::Metadata>::default())
    }
}

impl<M: MetricType> Metric<M> {
    /// Create a new metric with the given metadata
    pub fn with_metadata(metadata: M::Metadata) -> Self {
        Self {
            metric: M::default(),
            metadata,
        }
    }

    /// Get a ref to the metric
    pub fn get_metric(&self) -> MetricLockGuard<'_, M> {
        MetricLockGuard(MetricLockGuardRepr::Dense(&self.metric), &self.metadata)
    }

    /// Get a mut ref to the metric
    pub fn get_metric_mut(&mut self) -> MetricMut<'_, M> {
        MetricMut(&mut self.metric, &self.metadata)
    }
}

fn new_dense<M: MetricType>(c: usize) -> Box<[CachePadded<OnceLock<M>>]> {
    let mut vec = Vec::with_capacity(c);
    vec.resize_with(c, CachePadded::<OnceLock<M>>::default);
    vec.into_boxed_slice()
}

// if cardinality is greater than this, then metric vecs are allocated sparsely by default.
const DEFAULT_MAX_DENSE: usize = 1024;

impl<M: MetricType, L: LabelGroupSet + Default> MetricVec<M, L> {
    /// Create a new metric vec with the given label set and metric metadata
    pub fn with_metadata(metadata: M::Metadata) -> Self {
        Self::with_label_set_and_metadata(L::default(), metadata)
    }

    /// Create a new dense metric vec. Useful if you need to force a dense allocation for high performance and you are ok with the memory usage.
    pub fn dense_with_metadata(metadata: M::Metadata) -> Self {
        Self::dense_with_label_set_and_metadata(L::default(), metadata)
    }

    /// Create a new sparse metric vec. Useful if you have a fixed cardinality vec but the cardinality is quite high
    pub fn sparse_with_metadata(metadata: M::Metadata) -> Self {
        Self::sparse_with_label_set_and_metadata(L::default(), metadata)
    }
}

impl<M: MetricType, L: LabelGroupSet> MetricVec<M, L>
where
    M::Metadata: Default,
{
    /// Create a new metric vec with the given label set and metric metadata
    pub fn with_label_set(label_set: L) -> Self {
        Self::with_label_set_and_metadata(label_set, <M::Metadata>::default())
    }

    /// Create a new dense metric vec. Useful if you need to force a dense allocation for high performance and you are ok with the memory usage.
    pub fn dense_with_label_set(label_set: L) -> Self {
        Self::dense_with_label_set_and_metadata(label_set, <M::Metadata>::default())
    }

    /// Create a new sparse metric vec. Useful if you have a fixed cardinality vec but the cardinality is quite high
    pub fn sparse_with_label_set(label_set: L) -> Self {
        Self::sparse_with_label_set_and_metadata(label_set, <M::Metadata>::default())
    }
}

impl<M: MetricType, L: LabelGroupSet + Default> MetricVec<M, L>
where
    M::Metadata: Default,
{
    /// Create a new metric vec with the given label set and metric metadata
    pub fn new() -> Self {
        Self::with_label_set_and_metadata(L::default(), <M::Metadata>::default())
    }

    /// Create a new dense metric vec. Useful if you need to force a dense allocation for high performance and you are ok with the memory usage.
    pub fn dense() -> Self {
        Self::dense_with_label_set_and_metadata(L::default(), <M::Metadata>::default())
    }

    /// Create a new sparse metric vec. Useful if you have a fixed cardinality vec but the cardinality is quite high
    pub fn sparse() -> Self {
        Self::sparse_with_label_set_and_metadata(L::default(), <M::Metadata>::default())
    }
}

impl<M: MetricType, L: LabelGroupSet + Default> Default for MetricVec<M, L>
where
    M::Metadata: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<M: MetricType> Default for Metric<M>
where
    M::Metadata: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<M: MetricType, U: Hash + Eq + Copy> VecInner<U, M> {
    fn get_metric(&self, id: LabelIdInner<U>) -> MetricLockGuardRepr<'_, M> {
        match self {
            VecInner::Dense(metrics) => {
                let m = metrics[id.hash as usize].get_or_init(M::default);
                MetricLockGuardRepr::Dense(m)
            }
            VecInner::Sparse(metrics) => MetricLockGuardRepr::Sparse(metrics.get_metric(id)),
        }
    }

    fn get_metric_mut(&mut self, id: LabelIdInner<U>) -> &mut M {
        match self {
            VecInner::Dense(metrics) => {
                let m = &mut metrics[id.hash as usize];
                if m.get_mut().is_none() {
                    *m = CachePadded::new(OnceLock::from(M::default()));
                }

                m.get_mut().unwrap()
            }
            VecInner::Sparse(metrics) => metrics.get_metric_mut(id),
        }
    }
}

impl<M: MetricType, L: LabelGroupSet> MetricVec<M, L> {
    /// Create a new metric vec with the given label set and metric metadata
    pub fn with_label_set_and_metadata(label_set: L, metadata: M::Metadata) -> Self {
        let metrics = match label_set.cardinality() {
            Some(c) if c <= DEFAULT_MAX_DENSE => VecInner::Dense(new_dense(c)),
            _ => VecInner::Sparse(sparse::ShardedMap::new()),
        };

        Self {
            metrics,
            metadata,
            label_set,
        }
    }

    /// Create a new dense metric vec. Useful if you need to force a dense allocation for high performance and you are ok with the memory usage.
    pub fn dense_with_label_set_and_metadata(label_set: L, metadata: M::Metadata) -> Self {
        let c = label_set
            .cardinality()
            .expect("Label group does not have a fixed cardinality.");

        Self {
            metrics: VecInner::Dense(new_dense(c)),
            metadata,
            label_set,
        }
    }

    /// Create a new sparse metric vec. Useful if you have a fixed cardinality vec but the cardinality is quite high
    pub fn sparse_with_label_set_and_metadata(label_set: L, metadata: M::Metadata) -> Self {
        Self {
            metrics: VecInner::Sparse(sparse::ShardedMap::new()),
            metadata,
            label_set,
        }
    }

    /// For dense metric-vecs, sometimes you might want to initialise all metric values to their initial state.
    /// This is intended to run once at startup.
    ///
    /// # Note
    /// This does nothing if the metric vec is not 'dense'.
    ///
    /// You can initialise specific metric labels with the [`get_metric`](MetricVec::get_metric) method.
    pub fn init_all_dense(&mut self) {
        if let VecInner::Dense(metrics) = &mut self.metrics {
            for m in metrics.iter_mut() {
                if m.get_mut().is_none() {
                    *m = CachePadded::new(OnceLock::from(M::default()));
                }
            }
        }
    }

    /// View the metric metadata
    pub fn metadata(&self) -> &M::Metadata {
        &self.metadata
    }

    /// Get an identifier for the specific metric identified by this label group
    ///
    /// # Panics
    /// Panics if the label group is not contained within the label set.
    pub fn with_labels(&self, label: L::Group<'_>) -> LabelId<L> {
        self.try_with_labels(label)
            .expect("label group was not contained within this label set")
    }

    /// Get an identifier for the specific metric identified by this label group
    ///
    /// # Errors
    /// Returns None if the label group is not contained within the label set.
    pub fn try_with_labels(&self, label: L::Group<'_>) -> Option<LabelId<L>> {
        let id = self.label_set.encode(label)?;

        let hash = match &self.metrics {
            VecInner::Dense(metrics) => {
                let index = self.label_set.encode_dense(id).expect("If the label set is fixed in cardinality, it must return a value here in the range of `0..cardinality`");
                debug_assert!(index < metrics.len());
                index as u64
            }
            VecInner::Sparse(metrics) => metrics.hasher.hash_one(id),
        };

        Some(LabelId(LabelIdInner { id, hash }))
    }

    /// Get the individual metric at the given identifier.
    ///
    /// # Panics
    /// Can panic or cause strange behaviour if the label ID comes from a different metric family.
    pub fn get_metric(&self, id: LabelId<L>) -> MetricLockGuard<'_, M> {
        MetricLockGuard(self.metrics.get_metric(id.0), &self.metadata)
    }

    /// Remove the metric with the given label, returning it's inner state.
    ///
    /// # Note
    /// 'dense' metrics cannot be removed, and will always return None.
    /// If you need to support removal with a fixed-cardinality label set, use
    /// one of the sparse constructors.
    ///
    /// # Panics
    /// Can panic or cause strange behaviour if the label ID comes from a different metric family.
    pub fn remove_metric(&self, id: LabelId<L>) -> Option<M> {
        match &self.metrics {
            VecInner::Dense(_) => None,
            VecInner::Sparse(metrics) => metrics.remove_metric(id.0),
        }
    }

    /// Get the individual metric at the given identifier.
    ///
    /// # Panics
    /// Can panic or cause strange behaviour if the label ID comes from a different metric family.
    pub fn get_metric_mut(&mut self, id: LabelId<L>) -> MetricMut<M> {
        MetricMut(self.metrics.get_metric_mut(id.0), &self.metadata)
    }

    /// Inspect the current cardinality of this metric-vec, returning the lower bound and the upper bound if known
    pub fn get_cardinality(&self) -> (usize, Option<usize>) {
        match &self.metrics {
            VecInner::Dense(x) => (
                x.iter().filter(|x| x.get().is_some()).count(),
                Some(x.len()),
            ),
            VecInner::Sparse(x) => (x.get_cardinality(), self.label_set.cardinality()),
        }
    }

    /// Borrow the label set values
    pub fn get_label_set(&self) -> &L {
        &self.label_set
    }
}

/// Defines the encoding of a metric
pub trait MetricEncoding<T: Encoding>: MetricType {
    /// Write the type information for this metric into the encoder
    fn write_type(name: impl MetricNameEncoder, enc: &mut T) -> Result<(), T::Err>;
    /// Sample this metric into the encoder
    fn collect_into(
        &self,
        metadata: &Self::Metadata,
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut T,
    ) -> Result<(), T::Err>;
}

/// The encoding visitor for a single [`Metric`] or [`MetricVec`]
pub trait MetricFamilyEncoding<T: Encoding> {
    /// Collect these metric values into the given encoder with the given metric name
    fn collect_family_into(&self, name: impl MetricNameEncoder, enc: &mut T) -> Result<(), T::Err>;
}

impl<M: MetricFamilyEncoding<T>, T: Encoding> MetricFamilyEncoding<T> for Option<M> {
    fn collect_family_into(&self, name: impl MetricNameEncoder, enc: &mut T) -> Result<(), T::Err> {
        if let Some(this) = self {
            this.collect_family_into(name, enc)?;
        }
        Ok(())
    }
}

impl<M: MetricEncoding<T>, T: Encoding> MetricFamilyEncoding<T> for Metric<M> {
    /// Collect this metric value into the given encoder with the given metric name
    fn collect_family_into(&self, name: impl MetricNameEncoder, enc: &mut T) -> Result<(), T::Err> {
        M::write_type(&name, enc)?;
        self.metric
            .collect_into(&self.metadata, NoLabels, name, enc)
    }
}

impl<M: MetricEncoding<T>, L: LabelGroupSet, T: Encoding> MetricFamilyEncoding<T>
    for MetricVec<M, L>
{
    fn collect_family_into(&self, name: impl MetricNameEncoder, enc: &mut T) -> Result<(), T::Err> {
        M::write_type(&name, enc)?;
        match &self.metrics {
            VecInner::Dense(m) => {
                for (index, value) in m.iter().enumerate() {
                    if let Some(value) = value.get() {
                        value.collect_into(
                            &self.metadata,
                            self.label_set.decode_dense(index),
                            &name,
                            enc,
                        )?;
                    }
                }
            }
            VecInner::Sparse(m) => {
                for shard in m.shards.iter() {
                    for (k, v) in shard.read().iter() {
                        v.collect_into(&self.metadata, self.label_set.decode(k), &name, enc)?;
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct LabelId<L: LabelGroupSet>(LabelIdInner<L::Unique>);

#[derive(Clone, Copy)]
pub struct LabelIdInner<U> {
    id: U,
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
        self.0 == other.0
    }
}
impl<L: LabelGroupSet> Eq for LabelId<L> {}
impl<L: Hash + Eq> PartialEq for LabelIdInner<L> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<L: Hash + Eq> Eq for LabelIdInner<L> {}

#[cfg(test)]
mod tests {
    use crate::{CounterVec, FixedCardinalityLabel, LabelGroup};

    #[derive(Clone, Copy, PartialEq, Debug, LabelGroup)]
    #[label(crate = crate, set = ErrorsSet)]
    struct Error {
        kind: ErrorKind,
    }

    #[derive(Clone, Copy, PartialEq, Debug, Hash, Eq, FixedCardinalityLabel)]
    #[label(crate = crate)]
    enum ErrorKind {
        User,
        Internal,
        Network,
    }

    #[test]
    fn dense_cardinality() {
        let errors = CounterVec::<ErrorsSet>::dense();
        assert_eq!(errors.get_cardinality(), (0, Some(3)));

        errors.inc(Error {
            kind: ErrorKind::Internal,
        });
        errors.inc(Error {
            kind: ErrorKind::User,
        });
        assert_eq!(errors.get_cardinality(), (2, Some(3)));
    }

    #[test]
    fn sparse_cardinality() {
        let errors = CounterVec::<ErrorsSet>::sparse();
        assert_eq!(errors.get_cardinality(), (0, Some(3)));

        errors.inc(Error {
            kind: ErrorKind::Internal,
        });
        errors.inc(Error {
            kind: ErrorKind::User,
        });
        assert_eq!(errors.get_cardinality(), (2, Some(3)));
    }

    #[test]
    fn remove() {
        let errors = CounterVec::<ErrorsSet>::sparse();

        errors.inc(Error {
            kind: ErrorKind::Internal,
        });
        errors.inc(Error {
            kind: ErrorKind::User,
        });
        assert_eq!(errors.get_cardinality(), (2, Some(3)));
        let user_errors = errors
            .remove_metric(errors.with_labels(Error {
                kind: ErrorKind::User,
            }))
            .unwrap();

        assert_eq!(errors.get_cardinality(), (1, Some(3)));
        assert_eq!(user_errors.count.into_inner(), 1)
    }

    #[cfg(feature = "lasso")]
    #[derive(Clone, Copy, PartialEq, Debug, measured_derive::LabelGroup)]
    #[label(crate = crate, set = ErrorsSet2)]
    struct Error2<'a> {
        kind: ErrorKind,
        #[label(dynamic_with = lasso::ThreadedRodeo, default)]
        user: &'a str,
    }

    #[cfg(feature = "lasso")]
    #[test]
    fn dynamic_labels() {
        use fake::{faker::name::raw::Name, locales::EN, Fake};

        use crate::GaugeVec;

        let set = GaugeVec::with_label_set(ErrorsSet2::default());

        let names = (0..64).map(|_| Name(EN).fake()).collect::<Vec<String>>();

        let error_kinds = [ErrorKind::User, ErrorKind::Internal, ErrorKind::Network];

        for kind in error_kinds {
            for name in &names {
                let error = Error2 { kind, user: name };
                set.inc(error);
            }
        }
        for kind in error_kinds {
            for name in &names {
                let error = Error2 { kind, user: name };
                set.inc_by(error, 2);
            }
        }

        for kind in error_kinds {
            for name in &names {
                let error = Error2 { kind, user: name };
                let label = set.with_labels(error);
                let _ = set.remove_metric(label);
            }
        }
    }
}
