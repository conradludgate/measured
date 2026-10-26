//! Info metrics. See [`InfoMetric`]

use parking_lot::RwLock;

use super::MetricFamilyEncoding;
use super::group::{Encoding, MetricKind};
use super::name::MetricNameEncoder;
use crate::label::LabelGroup;

/// A gauge that is always 1, used to expose information as labels, such as a build version.
///
/// The metric name should end in `_info`. The Supporting encoders encoder removes it from the metric family name, as required by Supporting encoders.
///
/// ```
/// use measured::{InfoMetric, LabelGroup, MetricGroup};
///
/// #[derive(LabelGroup)]
/// struct BuildInfo {
///     version: &'static str,
///     commit: &'static str,
/// }
///
/// #[derive(MetricGroup)]
/// #[metric(new(info: BuildInfo))]
/// struct Metrics {
///     /// information about this build
///     #[metric(init = InfoMetric::new(info))]
///     build_info: InfoMetric<BuildInfo>,
/// }
///
/// let metrics = Metrics::new(BuildInfo { version: "1.0", commit: "abc" });
/// metrics
///     .build_info
///     .set_labels(BuildInfo { version: "1.1", commit: "def" });
/// ```
pub struct InfoMetric<L> {
    labels: RwLock<L>,
}

impl<L> InfoMetric<L> {
    /// Create a new info metric with the given labels
    pub fn new(labels: L) -> Self {
        Self { labels: RwLock::new(labels) }
    }

    /// Replace the labels of this info metric
    pub fn set_labels(&self, labels: L) {
        *self.labels.write() = labels;
    }

    /// Get a copy of the current labels of this info metric
    pub fn get_labels(&self) -> L
    where
        L: Clone,
    {
        self.labels.read().clone()
    }
}

impl<L: Default> Default for InfoMetric<L> {
    fn default() -> Self {
        Self::new(L::default())
    }
}

impl<L: LabelGroup> MetricFamilyEncoding for InfoMetric<L> {
    fn collect_family_into<E: Encoding>(
        &self,
        name: impl MetricNameEncoder,
        help: Option<&str>,
        enc: &mut E,
    ) -> Result<(), E::Err> {
        enc.write_family(&name, help, MetricKind::Info)?;
        enc.write_gauge(name, &*self.labels.read(), 1)
    }
}
