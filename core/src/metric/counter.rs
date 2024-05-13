//! All things counters. See [`Counter`]

use core::sync::atomic::AtomicU64;

use crate::{label::LabelGroupSet, Counter, CounterVec, LabelGroup};

use super::{
    group::Encoding, name::MetricNameEncoder, MetricEncoding, MetricLockGuard, MetricMut,
    MetricType,
};

#[derive(Default)]
/// The internal state that is used by [`Counter`] and [`CounterVec`]
pub struct CounterState {
    pub count: AtomicU64,
}

/// A reference to a specific counter.
pub type CounterLockGuard<'a> = MetricLockGuard<'a, CounterState>;
/// A mut reference to a specific counter.
pub type CounterMut<'a> = MetricMut<'a, CounterState>;

impl CounterState {
    pub fn new(value: u64) -> Self {
        Self {
            count: AtomicU64::new(value),
        }
    }

    /// Increment the counter value by 1
    pub fn inc(&self) {
        self.count
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    }

    /// Increment the counter value by `x`
    pub fn inc_by(&self, x: u64) {
        self.count
            .fetch_add(x, core::sync::atomic::Ordering::Relaxed);
    }
}

impl CounterMut<'_> {
    /// Increment the counter value by 1
    pub fn inc(mut self) {
        *self.count.get_mut() += 1;
    }

    /// Increment the counter value by `x`
    pub fn inc_by(mut self, x: u64) {
        *self.count.get_mut() += x;
    }
}

impl<L: LabelGroupSet> CounterVec<L> {
    /// Increment the counter value by 1, keyed by the label group
    pub fn inc(&self, label: L::Group<'_>) {
        self.get_metric(self.with_labels(label)).inc();
    }

    /// Increment the counter value by `y`, keyed by the label group
    pub fn inc_by(&self, label: L::Group<'_>, y: u64) {
        self.get_metric(self.with_labels(label)).inc_by(y);
    }

    /// Increment the counter value by 1, keyed by the label group
    pub fn inc_mut(&mut self, label: L::Group<'_>) {
        self.get_metric_mut(self.with_labels(label)).inc()
    }

    /// Increment the counter value by `y`, keyed by the label group
    pub fn inc_by_mut(&mut self, label: L::Group<'_>, y: u64) {
        self.get_metric_mut(self.with_labels(label)).inc_by(y)
    }
}

impl Counter {
    /// Increment the counter value by 1
    pub fn inc(&self) {
        self.get_metric().inc()
    }

    /// Increment the counter value by `x`
    pub fn inc_by(&self, x: u64) {
        self.get_metric().inc_by(x)
    }

    /// Increment the counter value by 1
    pub fn inc_mut(&mut self) {
        self.get_metric_mut().inc()
    }

    /// Increment the counter value by `x`
    pub fn inc_by_mut(&mut self, x: u64) {
        self.get_metric_mut().inc_by(x)
    }
}

impl MetricType for CounterState {
    /// [`Counter`]s require no additional metadata
    type Metadata = ();
}

pub fn write_counter<Enc: Encoding>(
    enc: &mut Enc,
    name: impl MetricNameEncoder,
    labels: impl LabelGroup,
    value: u64,
) -> Result<(), Enc::Err>
where
    CounterState: MetricEncoding<Enc>,
{
    CounterState {
        count: AtomicU64::new(value),
    }
    .collect_into(&(), labels, name, enc)
}
