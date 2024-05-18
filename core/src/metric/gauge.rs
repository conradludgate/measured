//! All things gauges. See [`Gauge`]

use core::sync::atomic::{AtomicI64, AtomicU64, Ordering};

use crate::{label::LabelGroupSet, FloatGauge, FloatGaugeVec, Gauge, GaugeVec, LabelGroup};

use super::{
    group::Encoding, name::MetricNameEncoder, MetricEncoding, MetricLockGuard, MetricMut,
    MetricType,
};

#[derive(Default)]
/// The internal state that is used by [`Gauge`] and [`GaugeVec`]
pub struct GaugeState {
    pub count: AtomicI64,
}

impl GaugeState {
    pub fn new(value: i64) -> Self {
        Self {
            count: AtomicI64::new(value),
        }
    }
}

/// A reference to a specific gauge.
pub type GaugeLockGuard<'a> = MetricLockGuard<'a, GaugeState>;

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

impl GaugeLockGuard<'_> {
    /// Increment the gauge value by 1
    pub fn inc(self) {
        self.count
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    }

    /// Increment the gauge value by `x`
    pub fn inc_by(self, x: i64) {
        self.count
            .fetch_add(x, core::sync::atomic::Ordering::Relaxed);
    }

    /// Decrement the gauge value by 1
    pub fn dec(self) {
        self.count
            .fetch_sub(1, core::sync::atomic::Ordering::Relaxed);
    }

    /// Decrement the gauge value by `x`
    pub fn dec_by(self, x: i64) {
        self.count
            .fetch_sub(x, core::sync::atomic::Ordering::Relaxed);
    }

    /// Set the gauge value to `x`
    pub fn set(self, x: i64) {
        self.count.store(x, core::sync::atomic::Ordering::Relaxed);
    }
}

impl GaugeMut<'_> {
    /// Increment the gauge value by 1
    pub fn inc(mut self) {
        *self.count.get_mut() += 1;
    }

    /// Increment the gauge value by `x`
    pub fn inc_by(mut self, x: i64) {
        *self.count.get_mut() += x;
    }

    /// Decrement the gauge value by 1
    pub fn dec(mut self) {
        *self.count.get_mut() -= 1;
    }

    /// Decrement the gauge value by `x`
    pub fn dec_by(mut self, x: i64) {
        *self.count.get_mut() -= x;
    }

    /// Set the gauge value to `x`
    pub fn set(mut self, x: i64) {
        *self.count.get_mut() = x;
    }
}

impl<L: LabelGroupSet> GaugeVec<L> {
    /// Increment the gauge value by 1, keyed by the label group
    pub fn inc(&self, label: L::Group<'_>) {
        self.get_metric(self.with_labels(label)).inc();
    }

    /// Increment the gauge value by `y`, keyed by the label group
    pub fn inc_by(&self, label: L::Group<'_>, y: i64) {
        self.get_metric(self.with_labels(label)).inc_by(y);
    }

    /// Decrement the gauge value by 1, keyed by the label group
    pub fn dec(&self, label: L::Group<'_>) {
        self.get_metric(self.with_labels(label)).dec();
    }

    /// Decrement the gauge value by `y`, keyed by the label group
    pub fn dec_by(&self, label: L::Group<'_>, y: i64) {
        self.get_metric(self.with_labels(label)).dec_by(y);
    }

    /// Set the gauge value to `y`, keyed by the label group
    pub fn set(&self, label: L::Group<'_>, y: i64) {
        self.get_metric(self.with_labels(label)).set(y);
    }
}

impl MetricType for GaugeState {
    /// [`Gauge`]s require no additional metadata
    type Metadata = ();
}

pub fn write_gauge<Enc: Encoding>(
    enc: &mut Enc,
    name: impl MetricNameEncoder,
    labels: impl LabelGroup,
    value: i64,
) -> Result<(), Enc::Err>
where
    GaugeState: MetricEncoding<Enc>,
{
    GaugeState {
        count: AtomicI64::new(value),
    }
    .collect_into(&(), labels, name, enc)
}

#[derive(Default)]
/// The internal state that is used by [`FloatGauge`] and [`FloatGaugeVec`]
pub struct FloatGaugeState {
    pub count: AtomicF64,
}

impl FloatGaugeState {
    pub fn new(value: f64) -> Self {
        Self {
            count: AtomicF64::new(value),
        }
    }
}

/// A reference to a specific gauge.
pub type FloatGaugeLockGuard<'a> = MetricLockGuard<'a, FloatGaugeState>;

/// A mut reference to a specific gauge.
pub type FloatGaugeMut<'a> = MetricMut<'a, FloatGaugeState>;

impl FloatGauge {
    /// Increment the gauge value by 1
    pub fn inc(&self) {
        self.get_metric().inc()
    }

    /// Increment the gauge value by `x`
    pub fn inc_by(&self, x: f64) {
        self.get_metric().inc_by(x)
    }

    /// Decrement the gauge value by 1
    pub fn dec(&self) {
        self.get_metric().dec()
    }

    /// Decrement the gauge value by `x`
    pub fn dec_by(&self, x: f64) {
        self.get_metric().dec_by(x)
    }

    /// Set the gauge value to `x`
    pub fn set(&self, x: f64) {
        self.get_metric().set(x)
    }
}

impl FloatGaugeLockGuard<'_> {
    /// Increment the gauge value by 1
    pub fn inc(self) {
        self.count.inc_by(1.0);
    }

    /// Increment the gauge value by `x`
    pub fn inc_by(self, x: f64) {
        self.count.inc_by(x);
    }

    /// Decrement the gauge value by 1
    pub fn dec(self) {
        self.count.dec_by(1.0);
    }

    /// Decrement the gauge value by `x`
    pub fn dec_by(self, x: f64) {
        self.count.dec_by(x);
    }

    /// Set the gauge value to `x`
    pub fn set(self, x: f64) {
        self.count.set(x);
    }
}

impl FloatGaugeMut<'_> {
    /// Increment the gauge value by 1
    pub fn inc(mut self) {
        let x = self.count.get_ex() + 1.0;
        self.count.set_mut(x);
    }

    /// Increment the gauge value by `x`
    pub fn inc_by(mut self, x: f64) {
        let x = self.count.get_ex() + x;
        self.count.set_mut(x);
    }

    /// Decrement the gauge value by 1
    pub fn dec(mut self) {
        let x = self.count.get_ex() - 1.0;
        self.count.set_mut(x);
    }

    /// Decrement the gauge value by `x`
    pub fn dec_by(mut self, x: f64) {
        let x = self.count.get_ex() - x;
        self.count.set_mut(x);
    }

    /// Set the gauge value to `x`
    pub fn set(mut self, x: f64) {
        self.count.set_mut(x);
    }
}

impl<L: LabelGroupSet> FloatGaugeVec<L> {
    /// Increment the gauge value by 1, keyed by the label group
    pub fn inc(&self, label: L::Group<'_>) {
        self.get_metric(self.with_labels(label)).inc();
    }

    /// Increment the gauge value by `y`, keyed by the label group
    pub fn inc_by(&self, label: L::Group<'_>, y: f64) {
        self.get_metric(self.with_labels(label)).inc_by(y);
    }

    /// Decrement the gauge value by 1, keyed by the label group
    pub fn dec(&self, label: L::Group<'_>) {
        self.get_metric(self.with_labels(label)).dec();
    }

    /// Decrement the gauge value by `y`, keyed by the label group
    pub fn dec_by(&self, label: L::Group<'_>, y: f64) {
        self.get_metric(self.with_labels(label)).dec_by(y);
    }

    /// Set the gauge value to `y`, keyed by the label group
    pub fn set(&self, label: L::Group<'_>, y: f64) {
        self.get_metric(self.with_labels(label)).set(y);
    }
}

impl MetricType for FloatGaugeState {
    /// [`Gauge`]s require no additional metadata
    type Metadata = ();
}

/// A atomic float.
#[derive(Debug, Default)]
pub struct AtomicF64 {
    inner: AtomicU64,
}

impl AtomicF64 {
    #[allow(clippy::declare_interior_mutable_const)]
    pub const ZERO: Self = Self {
        inner: AtomicU64::new(0),
    };

    pub fn new(val: f64) -> AtomicF64 {
        AtomicF64 {
            inner: AtomicU64::new(val.to_bits()),
        }
    }

    #[inline]
    pub fn set(&self, val: f64) {
        self.inner.store(val.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn get(&self) -> f64 {
        f64::from_bits(self.inner.load(Ordering::Relaxed))
    }

    #[inline]
    pub fn get_ex(&mut self) -> f64 {
        f64::from_bits(*self.inner.get_mut())
    }

    #[inline]
    pub fn set_mut(&mut self, f: f64) {
        *self.inner.get_mut() = f.to_bits();
    }

    #[inline]
    pub fn inc_by(&self, delta: f64) {
        loop {
            let current = self.inner.load(Ordering::Acquire);
            let new = f64::from_bits(current) + delta;
            let result = self.inner.compare_exchange_weak(
                current,
                new.to_bits(),
                Ordering::Release,
                Ordering::Relaxed,
            );
            if result.is_ok() {
                return;
            }
        }
    }

    #[inline]
    pub fn dec_by(&self, delta: f64) {
        self.inc_by(-delta);
    }
}

pub fn write_float_gauge<Enc: Encoding>(
    enc: &mut Enc,
    name: impl MetricNameEncoder,
    labels: impl LabelGroup,
    value: f64,
) -> Result<(), Enc::Err>
where
    FloatGaugeState: MetricEncoding<Enc>,
{
    FloatGaugeState {
        count: AtomicF64::new(value),
    }
    .collect_into(&(), labels, name, enc)
}

impl<E: Encoding> MetricEncoding<E> for GaugeState {
    fn collect_into(
        &self,
        _m: &(),
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut E,
    ) -> Result<(), E::Err> {
        enc.write_gauge(
            name,
            labels,
            self.count.load(std::sync::atomic::Ordering::Relaxed) as f64,
        )
    }
}

impl<E: Encoding> MetricEncoding<E> for FloatGaugeState {
    fn collect_into(
        &self,
        _m: &(),
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut E,
    ) -> Result<(), E::Err> {
        enc.write_gauge(name, labels, self.count.get())
    }
}
