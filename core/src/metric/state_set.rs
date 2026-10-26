//! State sets. See [`StateSet`]

use core::marker::PhantomData;
use core::sync::atomic::{AtomicUsize, Ordering};

use super::MetricFamilyEncoding;
use super::group::{Encoding, FamilyMeta, MetricKind};
use super::name::MetricNameEncoder;
use crate::label::{FixedCardinalityLabel, LabelValue, NoLabels};

/// Which state an enum is currently in, exposed as one series per state.
///
/// ```
/// use measured::{FixedCardinalityLabel, MetricGroup, StateSet};
///
/// #[derive(FixedCardinalityLabel, Clone, Copy, PartialEq, Debug)]
/// enum Status {
///     Starting,
///     Ready,
///     Stopping,
/// }
///
/// #[derive(MetricGroup)]
/// #[metric(new())]
/// struct Metrics {
///     /// the status of the server
///     #[metric(init = StateSet::new(Status::Starting))]
///     status: StateSet<Status>,
/// }
///
/// let metrics = Metrics::new();
/// metrics.status.set(Status::Ready);
/// assert_eq!(metrics.status.get(), Status::Ready);
/// ```
///
/// This is written as `status{status="ready"} 1`, and 0 for the other states.
pub struct StateSet<E> {
    state: AtomicUsize,
    _state: PhantomData<fn() -> E>,
}

impl<E: FixedCardinalityLabel> StateSet<E> {
    /// Create a state set in the given state
    pub fn new(state: E) -> Self {
        Self {
            state: AtomicUsize::new(state.encode()),
            _state: PhantomData,
        }
    }

    /// Change the current state
    pub fn set(&self, state: E) {
        self.state.store(state.encode(), Ordering::Relaxed);
    }

    /// Get the current state
    pub fn get(&self) -> E {
        E::decode(self.state.load(Ordering::Relaxed))
    }
}

impl<E: FixedCardinalityLabel + LabelValue> MetricFamilyEncoding for StateSet<E> {
    const KIND: Option<MetricKind> = Some(MetricKind::StateSet);

    fn collect_family_into<Enc: Encoding>(
        &self,
        name: impl MetricNameEncoder,
        meta: FamilyMeta<'_>,
        enc: &mut Enc,
    ) -> Result<(), Enc::Err> {
        enc.write_family(&name, MetricKind::StateSet, meta)?;
        let current = self.state.load(Ordering::Relaxed);
        let states = (0..E::cardinality()).map(|i| (E::decode(i), i == current));
        enc.write_state_set(name, NoLabels, states)
    }
}
