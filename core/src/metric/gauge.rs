use core::sync::atomic::AtomicI64;

use crate::{label::LabelGroupSet, Gauge, GaugeVec};

use super::{MetricMut, MetricRef, MetricType};

#[derive(Default)]
/// The internal state that is used by [`Gauge`] and [`GaugeVec`]
pub struct GaugeState {
    pub count: AtomicI64,
}

/// A reference to a specific gauge.
pub type GaugeRef<'a> = MetricRef<'a, GaugeState>;

/// A mut reference to a specific gauge.
pub type GaugeMut<'a> = MetricMut<'a, GaugeState>;

impl Gauge {
    /// Increment the gauge value by 1
    pub fn inc(&self) {
        self.get_metric().inc()
    }

    /// Increment the gauge value by `x`
    pub fn inc_by(&self, x: i64) {
        self.get_metric().inc_by(x)
    }

    /// Decrement the gauge value by 1
    pub fn dec(&self) {
        self.get_metric().dec()
    }

    /// Decrement the gauge value by `x`
    pub fn dec_by(&self, x: i64) {
        self.get_metric().dec_by(x)
    }

    /// Set the gauge value to `x`
    pub fn set(&self, x: i64) {
        self.get_metric().set(x)
    }
}

impl GaugeRef<'_> {
    /// Increment the gauge value by 1
    pub fn inc(self) {
        self.0
            .count
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    }

    /// Increment the gauge value by `x`
    pub fn inc_by(self, x: i64) {
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
    pub fn dec_by(self, x: i64) {
        self.0
            .count
            .fetch_sub(x, core::sync::atomic::Ordering::Relaxed);
    }

    /// Set the gauge value to `x`
    pub fn set(self, x: i64) {
        self.0.count.store(x, core::sync::atomic::Ordering::Relaxed);
    }
}

impl GaugeMut<'_> {
    /// Increment the gauge value by 1
    pub fn inc(self) {
        *self.0.count.get_mut() += 1;
    }

    /// Increment the gauge value by `x`
    pub fn inc_by(self, x: i64) {
        *self.0.count.get_mut() += x;
    }

    /// Decrement the gauge value by 1
    pub fn dec(self) {
        *self.0.count.get_mut() -= 1;
    }

    /// Decrement the gauge value by `x`
    pub fn dec_by(self, x: i64) {
        *self.0.count.get_mut() -= x;
    }

    /// Set the gauge value to `x`
    pub fn set(self, x: i64) {
        *self.0.count.get_mut() = x;
    }
}

impl<L: LabelGroupSet> GaugeVec<L> {
    /// Increment the gauge value by 1, keyed by the label group
    pub fn inc(&self, label: L::Group<'_>) {
        self.get_metric(
            self.with_labels(label)
                .expect("label group should be in the set"),
            |x| x.inc(),
        );
    }

    /// Increment the gauge value by `y`, keyed by the label group
    pub fn inc_by(&self, label: L::Group<'_>, y: i64) {
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
    pub fn dec_by(&self, label: L::Group<'_>, y: i64) {
        self.get_metric(
            self.with_labels(label)
                .expect("label group should be in the set"),
            |x| x.dec_by(y),
        );
    }

    /// Set the gauge value to `y`, keyed by the label group
    pub fn set(&self, label: L::Group<'_>, y: i64) {
        self.get_metric(
            self.with_labels(label)
                .expect("label group should be in the set"),
            |x| x.set(y),
        );
    }
}

impl MetricType for GaugeState {
    /// [`Gauge`]s require no additional metadata
    type Metadata = ();
}
