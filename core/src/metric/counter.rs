use std::sync::atomic::AtomicU64;

use crate::{label::LabelGroupSet, CounterVec};

use super::{MetricRef, MetricType};

#[derive(Default)]
pub struct CounterState {
    pub count: AtomicU64,
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
impl<L: LabelGroupSet> CounterVec<L> {
    pub fn new_counter_vec(label_set: L) -> Self {
        Self::new_metric_vec(label_set, ())
    }
    pub fn new_sparse_counter_vec(label_set: L) -> Self {
        Self::new_sparse_metric_vec(label_set, ())
    }

    pub fn inc(&self, label: L::Group<'_>) {
        self.get_metric(
            self.with_labels(label)
                .expect("label group should be in the set"),
            |x| x.inc(),
        )
    }

    pub fn inc_by(&self, label: L::Group<'_>, y: u64) {
        self.get_metric(
            self.with_labels(label)
                .expect("label group should be in the set"),
            |x| x.inc_by(y),
        )
    }
}

impl MetricType for CounterState {
    type Metadata = ();
}
