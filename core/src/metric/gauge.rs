use core::sync::atomic::AtomicU64;

use crate::{label::LabelGroupSet, Gauge, GaugeVec};

use super::{MetricRef, MetricType};

#[derive(Default)]
/// The internal state that is used by [`Gauge`] and [`GaugeVec`]
pub struct GaugeState {
    pub count: AtomicU64,
}

/// A reference to a specific gauge.
pub type GaugeRef<'a> = MetricRef<'a, GaugeState>;

impl GaugeRef<'_> {
    /// Increment the gauge value by 1
    pub fn inc(self) {
        self.0
            .count
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    }

    /// Increment the gauge value by `x`
    pub fn inc_by(self, x: u64) {
        self.0
            .count
            .fetch_add(x, core::sync::atomic::Ordering::Relaxed);
    }

    /// Decrement the gauge value by 1
    pub fn dec(self) {
        self.0
            .count
            .fetch_sub(1, core::sync::atomic::Ordering::Relaxed);
    }

    /// Decrement the gauge value by `x`
    pub fn dec_by(self, x: u64) {
        self.0
            .count
            .fetch_sub(x, core::sync::atomic::Ordering::Relaxed);
    }

    /// Set the gauge value to `x`
    pub fn set(self, x: u64) {
        self.0.count.store(x, core::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for Gauge {
    fn default() -> Self {
        Self::new()
    }
}

impl<L: LabelGroupSet + Default> Default for GaugeVec<L> {
    fn default() -> Self {
        Self::new(L::default())
    }
}

impl<L: LabelGroupSet> GaugeVec<L> {
    /// Create a new `GaugeVec`, with label keys identified by the label_set argument.
    pub fn new(label_set: L) -> Self {
        Self::new_metric_vec(label_set, ())
    }

    /// Create a new sparse `GaugeVec`, with label keys identified by the label_set argument.
    ///
    /// Sparse vecs are recommended if your max cardinality is quite high but the expected cardinality is low.
    /// The trade-off is that sparse vecs are not lock-free, although effort has been made to keep lock contention to a minimum.
    pub fn with_capacity(label_set: L, capacity: usize) -> Self {
        Self::with_capacity_and_metadata(label_set, capacity, ())
    }

    /// Increment the gauge value by 1, keyed by the label group
    pub fn inc(&self, label: L::Group<'_>) {
        self.get_metric(
            self.with_labels(label)
                .expect("label group should be in the set"),
            |x| x.inc(),
        );
    }

    /// Increment the gauge value by `y`, keyed by the label group
    pub fn inc_by(&self, label: L::Group<'_>, y: u64) {
        self.get_metric(
            self.with_labels(label)
                .expect("label group should be in the set"),
            |x| x.inc_by(y),
        );
    }

    /// Decrement the gauge value by 1, keyed by the label group
    pub fn dec(&self, label: L::Group<'_>) {
        self.get_metric(
            self.with_labels(label)
                .expect("label group should be in the set"),
            |x| x.dec(),
        );
    }

    /// Decrement the gauge value by `y`, keyed by the label group
    pub fn dec_by(&self, label: L::Group<'_>, y: u64) {
        self.get_metric(
            self.with_labels(label)
                .expect("label group should be in the set"),
            |x| x.dec_by(y),
        );
    }

    /// Set the gauge value to `y`, keyed by the label group
    pub fn set(&self, label: L::Group<'_>, y: u64) {
        self.get_metric(
            self.with_labels(label)
                .expect("label group should be in the set"),
            |x| x.set(y),
        );
    }
}

impl Gauge {
    /// Create a new `Gauge` metric.
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            metadata: (),
            metric: GaugeState {
                count: AtomicU64::new(0),
            },
        }
    }
}

impl MetricType for GaugeState {
    /// [`Gauge`]s require no additional metadata
    type Metadata = ();
}
