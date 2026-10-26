//! A metric type defined outside of measured, as a third party crate would.

use measured::metric::group::{Encoding, FamilyMeta, MetricKind, SampleExtras};
use measured::metric::name::MetricNameEncoder;
use measured::metric::{MetricEncoding, MetricType};
use measured::{LabelGroup, MetricGroup};
use measured_prometheus_text::BufferedTextEncoder;
use parking_lot::Mutex;

/// A summary that reports the min and max observations as the 0 and 1 quantiles.
#[derive(Default)]
struct MinMaxSummary {
    state: Mutex<Option<(f64, f64, f64, u64)>>,
}

impl MinMaxSummary {
    fn observe(&self, x: f64) {
        let mut state = self.state.lock();
        let (min, max, sum, count) = state.get_or_insert((x, x, 0.0, 0));
        *min = min.min(x);
        *max = max.max(x);
        *sum += x;
        *count += 1;
    }
}

impl MetricType for MinMaxSummary {
    type Metadata = ();
}

impl MetricEncoding for MinMaxSummary {
    const KIND: MetricKind = MetricKind::Summary;

    fn collect_into<E: Encoding>(
        &self,
        _: &(),
        labels: impl LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut E,
    ) -> Result<(), E::Err> {
        let (min, max, sum, count) = self.state.lock().unwrap_or_default();
        let quantiles = [(0.0, min), (1.0, max)];
        enc.write_summary(name, labels, &quantiles, count, sum, SampleExtras::NONE)
    }
}

/// A metric with a type that measured does not know about.
/// The prometheus text format does not support it, so it is written as untyped.
#[derive(Default)]
struct Flag(measured::Gauge);

impl measured::metric::MetricFamilyEncoding for Flag {
    fn collect_family_into<E: Encoding>(
        &self,
        name: impl MetricNameEncoder,
        meta: FamilyMeta<'_>,
        enc: &mut E,
    ) -> Result<(), E::Err> {
        enc.write_family(&name, MetricKind::Custom("stateset"), meta)?;
        enc.write_gauge(name, measured::label::NoLabels, self.0.get())
    }
}

#[derive(MetricGroup, Default)]
struct Metrics {
    /// request latency
    latency: measured::metric::Metric<MinMaxSummary>,
    /// a feature flag
    flag: Flag,
}

#[test]
fn custom_metric_types() {
    let metrics = Metrics::default();
    metrics.latency.get_metric().observe(0.5);
    metrics.latency.get_metric().observe(2.0);
    metrics.flag.0.set(1);

    let mut enc = BufferedTextEncoder::new();
    metrics.collect_group_into(&mut enc).unwrap();
    assert_eq!(
        String::from_utf8(enc.finish().to_vec()).unwrap(),
        r#"# HELP latency request latency
# TYPE latency summary
latency{quantile="0.0"} 0.5
latency{quantile="1.0"} 2.0
latency_sum 2.5
latency_count 2

# HELP flag a feature flag
# TYPE flag untyped
flag 1
"#
    );
}
