use core::hash::Hash;
use hashbrown::{hash_map::RawEntryMut, HashMap};
use parking_lot::{MappedRwLockReadGuard, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{
    hash::{BuildHasher, BuildHasherDefault},
    sync::OnceLock,
};
use thread_local::ThreadLocal;

use super::{LabelIdInner, MetricType};

pub struct Sparse<U: Hash + Eq + Send, M: MetricType + Send> {
    pub(super) hasher: BuildHasherDefault<rustc_hash::FxHasher>,
    pub(super) sample: Mutex<HashMap<U, M::Internal, ()>>,
    pub(super) locals: ThreadLocal<RwLock<HashMap<LabelIdInner<U>, M, ()>>>,
}

pub(super) struct ShardedMap<K, V> {
    // FxHasher performed the fastest in all my benchmarks.
    pub(super) hasher: BuildHasherDefault<rustc_hash::FxHasher>,
    // hasher: BuildHasherDefault<fnv::FnvHasher>,
    // hasher: BuildHasherDefault<twox_hash::XxHash64>,
    // hasher: BuildHasherDefault<twox_hash::Xxh3Hash64>,
    // hasher: BuildHasherDefault<ahash::AHasher>,
    // hasher: std::hash::RandomState,
    pub(super) shards: Box<[RwLock<HashMap<K, V, ()>>]>,
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

impl<M, U: Hash + Eq> ShardedMap<U, M> {
    pub(super) fn new() -> Self {
        let shards = default_shard_amount();
        let mut vec = Vec::with_capacity(shards);
        vec.resize_with(shards, || RwLock::new(HashMap::with_hasher(())));
        ShardedMap {
            hasher: Default::default(),
            shards: vec.into_boxed_slice(),
            shift: (std::mem::size_of::<usize>() * 8) as u32 - shards.trailing_zeros(),
        }
    }
}

impl<M: Default, U: Hash + Eq + Copy> ShardedMap<U, M> {
    pub(super) fn get_metric(&self, id: LabelIdInner<U>) -> SparseLockGuard<'_, M> {
        let shard = &self.shards[((id.hash as usize) << 7) >> self.shift];

        {
            let mapped = RwLockReadGuard::try_map(shard.read(), |shard| {
                shard
                    .raw_table()
                    .get(id.hash, |(k, _v)| *k == id.id)
                    .map(|(_, v)| v)
            });
            if let Ok(mapped) = mapped {
                return mapped;
            }
        }

        let shard = {
            let mut shard = shard.write();
            let entry = shard.raw_entry_mut().from_hash(id.hash, |k| *k == id.id);
            match entry {
                RawEntryMut::Occupied(_) => {}
                RawEntryMut::Vacant(v) => {
                    v.insert_with_hasher(id.hash, id.id, M::default(), |k| self.hasher.hash_one(k));
                }
            }
            RwLockWriteGuard::downgrade(shard)
        };

        RwLockReadGuard::map(shard, |shard| {
            &shard
                        .raw_table()
                        .get(id.hash, |(k, _v)| *k == id.id)
                        .expect("the entry was just inserted into the map without allowing any writes inbetween")
                        .1
        })
    }

    pub(super) fn remove_metric(&self, id: LabelIdInner<U>) -> Option<M> {
        let shard = &self.shards[((id.hash as usize) << 7) >> self.shift];

        let mut shard = shard.write();
        let entry = shard.raw_entry_mut().from_hash(id.hash, |k| *k == id.id);
        match entry {
            RawEntryMut::Occupied(x) => Some(x.remove()),
            RawEntryMut::Vacant(_) => None,
        }
    }

    pub(super) fn get_metric_mut(&mut self, id: LabelIdInner<U>) -> &mut M {
        let shard = &mut self.shards[((id.hash as usize) << 7) >> self.shift];

        let entry = shard
            .get_mut()
            .raw_entry_mut()
            .from_hash(id.hash, |k| *k == id.id);
        let (_, v) = match entry {
            RawEntryMut::Occupied(o) => o.into_key_value(),
            RawEntryMut::Vacant(v) => {
                v.insert_with_hasher(id.hash, id.id, M::default(), |k| self.hasher.hash_one(k))
            }
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

impl<M: MetricType + Send, U: Hash + Eq + Send> Sparse<U, M> {
    pub(super) fn new() -> Self {
        Self {
            hasher: Default::default(),
            sample: Mutex::new(HashMap::with_hasher(())),
            locals: ThreadLocal::with_capacity(
                std::thread::available_parallelism().map_or(0, |x| x.get()),
            ),
        }
    }
}

impl<M: MetricType + Send, U: Hash + Eq + Copy + Send> Sparse<U, M> {
    pub(super) fn get_metric(&self, id: LabelIdInner<U>) -> SparseLockGuard<'_, M> {
        let shard = self.locals.get_or_default();

        {
            let mapped = RwLockReadGuard::try_map(shard.read(), |shard| {
                shard
                    .raw_table()
                    .get(id.hash, |(k, _v)| k.id == id.id)
                    .map(|(_, v)| v)
            });
            if let Ok(mapped) = mapped {
                return mapped;
            }
        }

        let (shard, index) = {
            let mut shard = shard.write();
            // let entry = shard.raw_entry_mut().from_hash(id.hash, |k| k.id == id.id);

            let raw = shard.raw_table_mut();
            let res = raw.find_or_find_insert_slot(id.hash, |k| k.0.id == id.id, |k| k.0.hash);
            let bucket = match res {
                Ok(bucket) => bucket,
                Err(slot) => unsafe { raw.insert_in_slot(id.hash, slot, (id, M::default())) },
            };
            let index = unsafe { raw.bucket_index(&bucket) };

            // match entry {
            //     RawEntryMut::Occupied(_) => {}
            //     RawEntryMut::Vacant(v) => {
            //         v.insert_with_hasher(id.hash, id, M::default(), |k| k.hash);
            //     }
            // }
            (RwLockWriteGuard::downgrade(shard), index)
        };

        RwLockReadGuard::map(shard, |shard| {
            unsafe { &shard.raw_table().bucket(index).as_ref().1 }
            // &shard
            //             .raw_table()
            //             .get(id.hash, |(k, _v)| k.id == id.id)
            //             .expect("the entry was just inserted into the map without allowing any writes inbetween")
            //             .1
        })
    }

    pub(super) fn remove_metric(&self, id: LabelIdInner<U>) -> Option<M> {
        // let shard = &self.shards[((id.hash as usize) << 7) >> self.shift];

        // let mut shard = shard.write();
        // let entry = shard.raw_entry_mut().from_hash(id.hash, |k| k.id == id.id);
        // match entry {
        //     RawEntryMut::Occupied(x) => Some(x.remove()),
        //     RawEntryMut::Vacant(_) => None,
        // }
        todo!()
    }

    pub(super) fn get_metric_mut(&mut self, id: LabelIdInner<U>) -> &mut M {
        // let shard = &mut self.shards[((id.hash as usize) << 7) >> self.shift];

        // let entry = shard
        //     .get_mut()
        //     .raw_entry_mut()
        //     .from_hash(id.hash, |k| k.id == id.id);
        // let (_, v) = match entry {
        //     RawEntryMut::Occupied(o) => o.into_key_value(),
        //     RawEntryMut::Vacant(v) => {
        //         v.insert_with_hasher(id.hash, id.id, M::default(), |k| self.hasher.hash_one(k))
        //     }
        // };

        // v

        todo!()
    }

    pub(super) fn get_cardinality(&self) -> usize {
        // self.shards
        //     .iter()
        //     .map(|shard| shard.read().len())
        //     .sum::<usize>()
        0
    }
}
