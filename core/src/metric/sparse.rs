use core::hash::Hash;
use hashbrown::{hash_map::RawEntryMut, HashMap};
use parking_lot::{MappedRwLockReadGuard, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{collections::BTreeMap, hash::BuildHasherDefault};
use thread_local::ThreadLocal;

use super::{LabelIdInner, MetricType};

pub struct Sparse<U: Send, M: MetricType + Send> {
    // pub(super) hasher: BuildHasherDefault<rustc_hash::FxHasher>,
    pub(super) sample: Mutex<BTreeMap<U, M::Internal>>,
    pub(super) locals: ThreadLocal<RwLock<BTreeMap<U, M>>>,
}

pub(super) type SparseLockGuard<'a, M> = MappedRwLockReadGuard<'a, M>;

impl<M: MetricType + Send, U: Send> Sparse<U, M> {
    pub(super) fn new() -> Self {
        Self {
            sample: Mutex::new(BTreeMap::new()),
            locals: ThreadLocal::with_capacity(
                std::thread::available_parallelism().map_or(0, |x| x.get()),
            ),
        }
    }
}

impl<M: MetricType + Send + Sync, U: Ord + Copy + Send + Sync> Sparse<U, M> {
    pub(super) fn try_for_each<E>(
        &self,
        mut f: impl FnMut(LabelIdInner<U>, &M) -> Result<(), E>,
    ) -> Result<(), E> {
        for thread in self.locals.iter() {
            for (key, value) in thread.read().iter() {
                f(LabelIdInner { id: *key, hash: 0 }, value)?;
            }
        }

        Ok(())
    }
}

impl<M: MetricType + Send, U: Ord + Copy + Send> Sparse<U, M> {
    pub(super) fn get_metric(&self, id: LabelIdInner<U>) -> SparseLockGuard<'_, M> {
        let shard = self.locals.get_or_default();

        {
            let mapped = RwLockReadGuard::try_map(shard.read(), |shard| shard.get(&id.id));
            if let Ok(mapped) = mapped {
                return mapped;
            }
        }

        let shard = {
            let mut shard = shard.write();
            shard.entry(id.id).or_default();
            RwLockWriteGuard::downgrade(shard)
        };

        RwLockReadGuard::map(shard, |shard| shard.get(&id.id).unwrap())
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
        self.locals.get_or_default();
        let shard = self.locals.iter_mut().next().unwrap().get_mut();
        shard.entry(id.id).or_default()
    }

    pub(super) fn get_cardinality(&self) -> usize {
        // self.shards
        //     .iter()
        //     .map(|shard| shard.read().len())
        //     .sum::<usize>()
        0
    }
}
