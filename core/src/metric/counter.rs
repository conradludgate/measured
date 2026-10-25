//! All things counters. See [`Counter`]

use core::sync::atomic::AtomicU64;

use super::gauge::AtomicF64;
use super::group::{Encoding, MetricKind, SampleExtras};
use super::name::MetricNameEncoder;
use super::{MetricEncoding, MetricLockGuard, MetricMut, MetricType};
use crate::label::LabelGroupSet;
use crate::{Counter, CounterVec, FloatCounter, FloatCounterVec, LabelGroup};

#[derive(Default)]
/// The internal state that is used by [`Counter`] and [`CounterVec`]
pub struct CounterState {
    count: AtomicU64,
}

/// A reference to a specific counter.
pub type CounterLockGuard<'a> = MetricLockGuard<'a, CounterState>;
/// A mut reference to a specific counter.
pub type CounterMut<'a> = MetricMut<'a, CounterState>;

impl CounterState {
    /// Create a new counter state with the given initial value
    pub fn new(value: u64) -> Self {
        Self {
            count: AtomicU64::new(value),
        }
    }

    /// Get the current counter value
    pub fn get(&self) -> u64 {
        self.count.load(core::sync::atomic::Ordering::Relaxed)
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
    pub fn inc(self) {
        self.inc_by(1);
    }

    /// Increment the counter value by `x`. Wraps on overflow, like [`CounterState::inc_by`]
    pub fn inc_by(mut self, x: u64) {
        let count = self.count.get_mut();
        *count = count.wrapping_add(x);
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

    /// Get the counter value for the label group. Returns 0 if the counter has not been used yet.
    ///
    /// This does not create a new counter for the label group, or add its values to dynamic label sets.
    pub fn get(&self, label: L::Group<'_>) -> u64 {
        self.find_labels(label)
            .and_then(|id| self.find_metric(id))
            .map_or(0, |m| m.get())
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

    /// Get the current counter value
    pub fn get(&self) -> u64 {
        self.get_metric().get()
    }
}

impl MetricType for CounterState {
    /// [`Counter`]s require no additional metadata
    type Metadata = ();
}

impl MetricEncoding for CounterState {
    const KIND: MetricKind = MetricKind::Counter;

    fn collect_into<E: Encoding>(
        &self,
        _: &(),
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut E,
    ) -> Result<(), E::Err> {
        enc.write_counter(name, labels, self.get(), SampleExtras::NONE)
    }
}

#[derive(Default)]
/// The internal state that is used by [`FloatCounter`] and [`FloatCounterVec`]
pub struct FloatCounterState {
    count: AtomicF64,
}

/// A reference to a specific float counter.
pub type FloatCounterLockGuard<'a> = MetricLockGuard<'a, FloatCounterState>;
/// A mut reference to a specific float counter.
pub type FloatCounterMut<'a> = MetricMut<'a, FloatCounterState>;

impl FloatCounterState {
    /// Create a new float counter state with the given initial value
    pub fn new(value: f64) -> Self {
        Self {
            count: AtomicF64::new(value),
        }
    }

    /// Get the current counter value
    pub fn get(&self) -> f64 {
        self.count.get()
    }

    /// Increment the counter value by 1
    pub fn inc(&self) {
        self.inc_by(1.0);
    }

    /// Increment the counter value by `x`.
    ///
    /// Counters cannot decrease, so negative and NaN values are ignored. They panic in debug builds.
    pub fn inc_by(&self, x: f64) {
        debug_assert!(x >= 0.0, "counters should not decrease: {x}");
        if x >= 0.0 {
            self.count.inc_by(x);
        }
    }
}

impl FloatCounterMut<'_> {
    /// Increment the counter value by 1
    pub fn inc(self) {
        self.inc_by(1.0);
    }

    /// Increment the counter value by `x`.
    ///
    /// Counters cannot decrease, so negative and NaN values are ignored. They panic in debug builds.
    pub fn inc_by(mut self, x: f64) {
        debug_assert!(x >= 0.0, "counters should not decrease: {x}");
        if x >= 0.0 {
            let count = self.count.get_ex() + x;
            self.count.set_mut(count);
        }
    }
}

impl<L: LabelGroupSet> FloatCounterVec<L> {
    /// Increment the counter value by 1, keyed by the label group
    pub fn inc(&self, label: L::Group<'_>) {
        self.get_metric(self.with_labels(label)).inc();
    }

    /// Increment the counter value by `y`, keyed by the label group
    pub fn inc_by(&self, label: L::Group<'_>, y: f64) {
        self.get_metric(self.with_labels(label)).inc_by(y);
    }

    /// Increment the counter value by 1, keyed by the label group
    pub fn inc_mut(&mut self, label: L::Group<'_>) {
        self.get_metric_mut(self.with_labels(label)).inc()
    }

    /// Increment the counter value by `y`, keyed by the label group
    pub fn inc_by_mut(&mut self, label: L::Group<'_>, y: f64) {
        self.get_metric_mut(self.with_labels(label)).inc_by(y)
    }

    /// Get the counter value for the label group. Returns 0 if the counter has not been used yet.
    ///
    /// This does not create a new counter for the label group, or add its values to dynamic label sets.
    pub fn get(&self, label: L::Group<'_>) -> f64 {
        self.find_labels(label)
            .and_then(|id| self.find_metric(id))
            .map_or(0.0, |m| m.get())
    }
}

impl FloatCounter {
    /// Increment the counter value by 1
    pub fn inc(&self) {
        self.get_metric().inc()
    }

    /// Increment the counter value by `x`
    pub fn inc_by(&self, x: f64) {
        self.get_metric().inc_by(x)
    }

    /// Increment the counter value by 1
    pub fn inc_mut(&mut self) {
        self.get_metric_mut().inc()
    }

    /// Increment the counter value by `x`
    pub fn inc_by_mut(&mut self, x: f64) {
        self.get_metric_mut().inc_by(x)
    }

    /// Get the current counter value
    pub fn get(&self) -> f64 {
        self.get_metric().get()
    }
}

impl MetricType for FloatCounterState {
    /// [`FloatCounter`]s require no additional metadata
    type Metadata = ();
}

impl MetricEncoding for FloatCounterState {
    const KIND: MetricKind = MetricKind::Counter;

    fn collect_into<E: Encoding>(
        &self,
        _: &(),
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut E,
    ) -> Result<(), E::Err> {
        enc.write_float_counter(name, labels, self.get(), SampleExtras::NONE)
    }
}
