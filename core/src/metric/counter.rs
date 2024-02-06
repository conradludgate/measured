use core::sync::atomic::AtomicU64;

use crate::{label::LabelGroupSet, Counter, CounterVec};

use super::{MetricRef, MetricType};

#[derive(Default)]
/// The internal state that is used by [`Counter`] and [`CounterVec`]
pub struct CounterState {
    pub count: AtomicU64,
}

/// A reference to a specific counter.
pub type CounterRef<'a> = MetricRef<'a, CounterState>;

impl CounterRef<'_> {
    /// Increment the counter value by 1
    pub fn inc(self) {
        self.0
            .count
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    }

    /// Increment the counter value by `x`
    pub fn inc_by(self, x: u64) {
        self.0
            .count
            .fetch_add(x, core::sync::atomic::Ordering::Relaxed);
    }
}

impl<L: LabelGroupSet> CounterVec<L> {
    /// Create a new `CounterVec`, with label keys identified by the label_set argument.
    pub fn new(label_set: L) -> Self {
        Self::new_metric_vec(label_set, ())
    }

    /// Create a new sparse `CounterVec`, with label keys identified by the label_set argument.
    ///
    /// Sparse vecs are recommended if your max cardinality is quite high but the expected cardinality is low.
    /// The trade-off is that sparse vecs are not lock-free, although effort has been made to keep lock contention to a minimum.
    pub fn new_sparse(label_set: L) -> Self {
        Self::new_sparse_metric_vec(label_set, ())
    }

    /// Increment the counter value by 1, keyed by the label group
    pub fn inc(&self, label: L::Group<'_>) {
        self.get_metric(
            self.with_labels(label)
                .expect("label group should be in the set"),
            |x| x.inc(),
        );
    }

    /// Increment the counter value by `y`, keyed by the label group
    pub fn inc_by(&self, label: L::Group<'_>, y: u64) {
        self.get_metric(
            self.with_labels(label)
                .expect("label group should be in the set"),
            |x| x.inc_by(y),
        );
    }
}

impl Counter {
    /// Create a new `Counter` metric.
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            metadata: (),
            metric: CounterState {
                count: AtomicU64::new(0),
            },
        }
    }
}

impl MetricType for CounterState {
    /// [`Counter`]s require no additional metadata
    type Metadata = ();
}
