//! Metrics that are computed when they are collected. See [`CallbackMetric`]

use core::marker::PhantomData;

use super::counter::{CounterState, FloatCounterState};
use super::gauge::{FloatGaugeState, GaugeState};
use super::group::Encoding;
use super::name::MetricNameEncoder;
use super::{MetricEncoding, MetricFamilyEncoding};
use crate::label::NoLabels;

/// A metric whose value is computed by calling a function every time it is collected.
///
/// This is useful for values that are already tracked elsewhere, such as the size of a pool or cache,
/// and saves implementing [`MetricGroup`](crate::MetricGroup) by hand.
///
/// ```
/// use std::sync::Arc;
/// use std::sync::atomic::{AtomicUsize, Ordering};
///
/// use measured::{CallbackGauge, MetricGroup};
///
/// #[derive(MetricGroup)]
/// #[metric(new(pool: Arc<AtomicUsize>))]
/// struct Metrics {
///     /// number of idle connections in the pool
///     #[metric(init = CallbackGauge::gauge(move || pool.load(Ordering::Relaxed) as i64))]
///     idle_connections: CallbackGauge,
/// }
///
/// let pool = Arc::new(AtomicUsize::new(3));
/// let metrics = Metrics::new(pool.clone());
/// ```
pub struct CallbackMetric<M, F = Box<dyn Fn() -> M + Send + Sync>> {
    f: F,
    _metric: PhantomData<fn() -> M>,
}

/// A gauge whose value is computed when it is collected. See [`CallbackMetric`]
pub type CallbackGauge = CallbackMetric<GaugeState>;
/// A float gauge whose value is computed when it is collected. See [`CallbackMetric`]
pub type CallbackFloatGauge = CallbackMetric<FloatGaugeState>;
/// A counter whose value is computed when it is collected. See [`CallbackMetric`]
pub type CallbackCounter = CallbackMetric<CounterState>;
/// A float counter whose value is computed when it is collected. See [`CallbackMetric`]
pub type CallbackFloatCounter = CallbackMetric<FloatCounterState>;

impl<M, F: Fn() -> M> CallbackMetric<M, F> {
    /// Create a metric that calls `f` to get the current metric state every time it is collected.
    pub fn new(f: F) -> Self {
        Self {
            f,
            _metric: PhantomData,
        }
    }
}

impl CallbackGauge {
    /// Create a gauge that calls `f` to get its value every time it is collected.
    pub fn gauge(f: impl Fn() -> i64 + Send + Sync + 'static) -> Self {
        Self::new(Box::new(move || GaugeState::new(f())))
    }
}

impl CallbackFloatGauge {
    /// Create a float gauge that calls `f` to get its value every time it is collected.
    pub fn float_gauge(f: impl Fn() -> f64 + Send + Sync + 'static) -> Self {
        Self::new(Box::new(move || FloatGaugeState::new(f())))
    }
}

impl CallbackCounter {
    /// Create a counter that calls `f` to get its value every time it is collected.
    ///
    /// The value returned should never decrease.
    pub fn counter(f: impl Fn() -> u64 + Send + Sync + 'static) -> Self {
        Self::new(Box::new(move || CounterState::new(f())))
    }
}

impl CallbackFloatCounter {
    /// Create a float counter that calls `f` to get its value every time it is collected.
    ///
    /// The value returned should never decrease.
    pub fn float_counter(f: impl Fn() -> f64 + Send + Sync + 'static) -> Self {
        Self::new(Box::new(move || FloatCounterState::new(f())))
    }
}

impl<M, F> MetricFamilyEncoding for CallbackMetric<M, F>
where
    M: MetricEncoding<Metadata = ()>,
    F: Fn() -> M,
{
    fn collect_family_into<E: Encoding>(
        &self,
        name: impl MetricNameEncoder,
        help: Option<&str>,
        enc: &mut E,
    ) -> Result<(), E::Err> {
        enc.write_family(&name, help, M::KIND)?;
        (self.f)().collect_into(&(), NoLabels, name, enc)
    }
}
