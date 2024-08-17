use core::hash::Hash;
use seize::Guard;
use std::hash::BuildHasherDefault;

use super::{LabelIdInner, MetricType};

pub(super) struct ShardedMap<K, V> {
    pub(super) map: papaya::HashMap<K, V, BuildHasherDefault<rustc_hash::FxHasher>>,
}

pub(super) type SparseLockGuard<'a, M> = &'a M;

impl<M: MetricType, U: Hash + Eq> ShardedMap<U, M> {
    pub(super) fn new() -> Self {
        ShardedMap {
            map: papaya::HashMap::with_hasher(BuildHasherDefault::default()),
        }
    }
}

impl<M: MetricType, U: Hash + Eq + Copy> ShardedMap<U, M> {
    pub(super) fn guard(&self) -> impl Guard + '_ {
        self.map.guard()
    }

    pub(super) fn get_metric<'g>(
        &self,
        id: LabelIdInner<U>,
        guard: &'g impl Guard,
    ) -> SparseLockGuard<'g, M>
    where
        U: 'g,
    {
        self.map.get_or_insert_with(id.id, M::default, guard)
    }

    // pub(super) fn remove_metric(&self, id: LabelIdInner<U>,
    //     guard: &'g impl Guard,) -> Option<M> {
    //     self.map.remove(key, guard)
    // }

    // pub(super) fn get_metric_mut(&mut self, id: LabelIdInner<U>) -> &mut M {
    //     let shard = &mut self.shards[((id.hash as usize) << 7) >> self.shift];

    //     let entry = shard.get_mut().find_entry(id.hash, |(k, _)| *k == id.id);
    //     let (_, v) = match entry {
    //         Ok(o) => o.into_mut(),
    //         Err(v) => v
    //             .into_table()
    //             .insert_unique(id.hash, (id.id, M::default()), |(k, _)| {
    //                 self.hasher.hash_one(k)
    //             })
    //             .into_mut(),
    //     };

    //     v
    // }

    pub(super) fn get_cardinality(&self) -> usize {
        self.map.len()
    }
}
