use core::hash::Hash;
use crossbeam_utils::CachePadded;
use hashbrown::HashTable;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{
    hash::{BuildHasher, BuildHasherDefault},
    sync::OnceLock,
};

use super::{LabelIdInner, MetricType};

pub(super) struct ShardedMap<K, V> {
    // FxHasher performed the fastest in all my benchmarks.
    pub(super) hasher: BuildHasherDefault<rustc_hash::FxHasher>,
    // hasher: BuildHasherDefault<fnv::FnvHasher>,
    // hasher: BuildHasherDefault<twox_hash::XxHash64>,
    // hasher: BuildHasherDefault<twox_hash::Xxh3Hash64>,
    // hasher: BuildHasherDefault<ahash::AHasher>,
    // hasher: std::hash::RandomState,
    #[allow(clippy::type_complexity)]
    pub(super) shards: Box<[CachePadded<RwLock<HashTable<(K, V)>>>]>,
    shift: u32,
}

// taken from dashmap
fn default_shard_amount() -> usize {
    static DEFAULT_SHARD_AMOUNT: OnceLock<usize> = OnceLock::new();
    *DEFAULT_SHARD_AMOUNT.get_or_init(|| {
        (std::thread::available_parallelism().map_or(1, usize::from) * 4).next_power_of_two()
    })
}

pub(super) type SparseLockGuard<'a, M> = MappedRwLockReadGuard<'a, M>;

impl<M: MetricType, U: Hash + Eq> ShardedMap<U, M> {
    pub(super) fn new() -> Self {
        let shards = default_shard_amount();
        let mut vec = Vec::with_capacity(shards);
        vec.resize_with(shards, || CachePadded::new(RwLock::new(HashTable::new())));
        ShardedMap {
            hasher: Default::default(),
            shards: vec.into_boxed_slice(),
            shift: (std::mem::size_of::<usize>() * 8) as u32 - shards.trailing_zeros(),
        }
    }
}

impl<M: MetricType, U: Hash + Eq + Copy> ShardedMap<U, M> {
    pub(super) fn get_metric(&self, id: LabelIdInner<U>) -> SparseLockGuard<'_, M> {
        let shard = &self.shards[((id.hash as usize) << 7) >> self.shift];

        {
            let mapped = RwLockReadGuard::try_map(shard.read(), |shard| {
                shard.find(id.hash, |(k, _v)| *k == id.id).map(|(_, v)| v)
            });
            if let Ok(mapped) = mapped {
                return mapped;
            }
        }

        let shard = {
            let mut shard = shard.write();
            let entry = shard.find_entry(id.hash, |(k, _)| *k == id.id);
            match entry {
                Ok(_) => {}
                Err(_) => {
                    shard.insert_unique(id.hash, (id.id, M::default()), |(k, _)| {
                        self.hasher.hash_one(k)
                    });
                }
            }
            RwLockWriteGuard::downgrade(shard)
        };

        RwLockReadGuard::map(shard, |shard| {
            let (_, v) = shard.find(id.hash, |(k, _v)| *k == id.id).expect(
                "the entry was just inserted into the map without allowing any writes inbetween",
            );
            v
        })
    }

    pub(super) fn remove_metric(&self, id: LabelIdInner<U>) -> Option<M> {
        let shard = &self.shards[((id.hash as usize) << 7) >> self.shift];

        let mut shard = shard.write();
        let entry = shard.find_entry(id.hash, |(k, _)| *k == id.id);
        match entry {
            Ok(x) => Some(x.remove().0.1),
            Err(_) => None,
        }
    }

    pub(super) fn get_metric_mut(&mut self, id: LabelIdInner<U>) -> &mut M {
        let shard = &mut self.shards[((id.hash as usize) << 7) >> self.shift];

        let entry = shard.get_mut().find_entry(id.hash, |(k, _)| *k == id.id);
        let (_, v) = match entry {
            Ok(o) => o.into_mut(),
            Err(v) => v
                .into_table()
                .insert_unique(id.hash, (id.id, M::default()), |(k, _)| {
                    self.hasher.hash_one(k)
                })
                .into_mut(),
        };

        v
    }

    pub(super) fn get_cardinality(&self) -> usize {
        self.shards
            .iter()
            .map(|shard| shard.read().len())
            .sum::<usize>()
    }
}
