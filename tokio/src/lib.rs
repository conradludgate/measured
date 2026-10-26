//! Monitor Tokio runtimes with [`measured`].
//!
//! `RuntimeCollector` samples the current Tokio runtime when a metric group is collected.
//! `NamedRuntimesCollector` can sample several runtimes and add a runtime name label.
//!
//! Add `measured-tokio` alongside `measured`. The following example uses stable runtime metrics
//! and exports them as Prometheus text:
//!
//! ```
//! use measured::MetricGroup;
//! use measured_prometheus_text::BufferedTextEncoder;
//!
//! #[derive(MetricGroup)]
//! #[metric(new())]
//! struct AppMetrics {
//!     #[metric(namespace = "tokio")]
//!     #[metric(init = measured_tokio::RuntimeCollector::current())]
//!     runtime: measured_tokio::RuntimeCollector,
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let metrics = AppMetrics::new();
//!     let mut encoder = BufferedTextEncoder::new();
//!     metrics.collect_group_into(&mut encoder).unwrap();
//!     let body = encoder.finish();
//!     assert!(
//!         std::str::from_utf8(&body)
//!             .unwrap()
//!             .contains("tokio_alive_tasks")
//!     );
//! }
//! ```
//!
//! By default the collector uses stable Tokio APIs. Some additional metrics require passing
//! `--cfg tokio_unstable`. I/O driver metrics require both that cfg and this crate's `net` feature.
//! Schedule-latency metrics additionally require `__tokio-unstable-schedule-latency`; they rely on
//! unstable Tokio APIs and are outside this crate's semver guarantees. They require a 64-bit target.

use std::borrow::Cow;
use std::sync::RwLock;

use measured::label::{ComposedGroup, LabelGroupVisitor, LabelName, LabelValue, LabelVisitor, NoLabels};
use measured::metric::group::{Encoding, FamilyMeta, MetricKind, SampleExtras};
use measured::metric::name::MetricName;
use measured::{FixedCardinalityLabel, LabelGroup, MetricGroup};
use tokio::runtime::RuntimeMetrics;

/// A collector which contains multiple named tokio runtimes
pub struct NamedRuntimesCollector {
    runtimes: RwLock<Vec<RuntimeCollector>>,
}

impl NamedRuntimesCollector {
    /// Create a new empty `NamedRuntimesCollector`
    pub fn new() -> Self {
        Self { runtimes: RwLock::new(vec![]) }
    }

    /// Add a collector for a runtime and attach `name` as a label to its metrics.
    pub fn add(&self, rt: RuntimeMetrics, name: impl Into<Cow<'static, str>>) {
        self.runtimes
            .write()
            .unwrap()
            .push(RuntimeCollector::new(rt).with_name(name))
    }

    /// Add a collector for the current runtime with the given name.
    ///
    /// # Panics
    ///
    /// This will panic if called outside the context of a Tokio runtime. That means that you must
    /// call this on one of the threads **being run by the runtime**, or from a thread with an active
    /// `EnterGuard`. Calling this from within a thread created by `std::thread::spawn` (for example)
    /// will cause a panic unless that thread has an active `EnterGuard`.
    pub fn add_current(&self, name: impl Into<Cow<'static, str>>) {
        self.add(tokio::runtime::Handle::current().metrics(), name);
    }
}

impl Default for NamedRuntimesCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl<Enc: Encoding> MetricGroup<Enc> for NamedRuntimesCollector {
    fn collect_group_into(&self, enc: &mut Enc) -> Result<(), <Enc as Encoding>::Err> {
        collect(&self.runtimes.read().unwrap(), enc)
    }
}

/// A collector which exports the current state of tokio metrics
pub struct RuntimeCollector {
    runtime: RuntimeMetrics,
    name: RuntimeName,
}

impl RuntimeCollector {
    /// Create a collector for the given runtime's metrics.
    pub fn new(runtime: RuntimeMetrics) -> Self {
        RuntimeCollector {
            runtime,
            name: RuntimeName { name: None },
        }
    }

    /// Return a collector for the current runtime.
    ///
    /// # Panics
    ///
    /// This will panic if called outside the context of a Tokio runtime. That means that you must
    /// call this on one of the threads **being run by the runtime**, or from a thread with an active
    /// `EnterGuard`. Calling this from within a thread created by `std::thread::spawn` (for example)
    /// will cause a panic unless that thread has an active `EnterGuard`.
    pub fn current() -> Self {
        RuntimeCollector::new(tokio::runtime::Handle::current().metrics())
    }

    /// Attach a runtime name label to each metric from this collector.
    pub fn with_name(self, name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            runtime: self.runtime,
            name: RuntimeName { name: Some(name.into()) },
        }
    }
}

#[cfg(tokio_unstable)]
#[cfg(feature = "__tokio-unstable-schedule-latency")]
fn histogram_le(le: std::time::Duration) -> HistogramLabelLe {
    let le = if le == std::time::Duration::from_nanos(u64::MAX) {
        f64::INFINITY
    } else {
        le.as_secs_f64()
    };
    HistogramLabelLe { le }
}

fn collect<Enc: Encoding>(runtimes: &[RuntimeCollector], enc: &mut Enc) -> Result<(), Enc::Err> {
    macro_rules! metric {
        ($name:literal, $help:literal, kind = $kind:expr, |$rt:ident| $expr:expr) => {{
            #![allow(unused_macros)]
            const NAME: &MetricName = MetricName::from_str($name);
            enc.write_family(NAME, $kind, FamilyMeta::new().with_help($help))?;
            for rt in runtimes {
                let rt_name = &rt.name;
                macro_rules! write_counter {
                    ($labels:expr, $val:expr) => {
                        enc.write_counter(
                            NAME,
                            ComposedGroup(rt_name, $labels),
                            $val,
                            SampleExtras::NONE,
                        )?
                    };
                    ($suffix:expr, $labels:expr, $val:expr) => {
                        enc.write_counter(
                            NAME.with_suffix($suffix),
                            ComposedGroup(rt_name, $labels),
                            $val,
                            SampleExtras::NONE,
                        )?
                    };
                }
                macro_rules! write_gauge {
                    ($labels:expr, $val:expr) => {
                        enc.write_gauge(
                            NAME,
                            ComposedGroup(rt_name, $labels),
                            $val,
                        )?
                    };
                    ($suffix:expr, $labels:expr, $val:expr) => {
                        enc.write_gauge(
                            NAME.with_suffix($suffix),
                            ComposedGroup(rt_name, $labels),
                            $val,
                        )?
                    };
                }
                macro_rules! write_float_gauge {
                    ($labels:expr, $val:expr) => {
                        enc.write_float_gauge(
                            NAME,
                            ComposedGroup(rt_name, $labels),
                            $val,
                        )?
                    };
                    ($suffix:expr, $labels:expr, $val:expr) => {
                        enc.write_float_gauge(
                            NAME.with_suffix($suffix),
                            ComposedGroup(rt_name, $labels),
                            $val,
                        )?
                    };
                }
                macro_rules! write_histogram {
                    ($labels:expr, $le:expr, $counts:expr, $inf:expr, $sum:expr) => {
                        enc.write_histogram(
                            NAME,
                            ComposedGroup(rt_name, $labels),
                            $le,
                            $counts,
                            $inf,
                            $sum,
                            measured::metric::group::HistogramExtras::NONE,
                        )?
                    };
                }
                let $rt = &rt.runtime;
                ($expr)
            }
        }};
    }

    metric!(
        "threads",
        "number of threads used by the runtime",
        kind = MetricKind::Gauge,
        |rt| {
            write_gauge!(ThreadKind::Worker, rt.num_workers() as i64);

            #[cfg(tokio_unstable)]
            let idle = rt.num_idle_blocking_threads();

            // we subtract here so that `sum(threads)` actually gives the total number of threads.
            #[cfg(tokio_unstable)]
            write_gauge!(
                ThreadKind::Blocking,
                rt.num_blocking_threads().saturating_sub(idle) as i64
            );

            #[cfg(tokio_unstable)]
            write_gauge!(ThreadKind::BlockingIdle, idle as i64);
        }
    );

    metric!(
        "alive_tasks",
        "number of live tasks spawned in the runtime",
        kind = MetricKind::Gauge,
        |rt| write_gauge!(NoLabels, rt.num_alive_tasks() as i64)
    );

    #[cfg(tokio_unstable)]
    metric!(
        "tasks_total",
        "number of tasks",
        kind = MetricKind::Counter,
        |rt| {
            write_counter!(NoLabels, rt.spawned_tasks_count());
        }
    );

    metric!(
        "queued_tasks",
        "number of tasks currently in a queue",
        kind = MetricKind::Gauge,
        |rt| {
            #[cfg(tokio_unstable)]
            write_gauge!(QueueKind::Blocking, rt.blocking_queue_depth() as i64);

            write_gauge!(QueueKind::Global, rt.global_queue_depth() as i64);

            #[cfg(tokio_unstable)]
            for worker in 0..rt.num_workers() {
                let queue_depth = rt.worker_local_queue_depth(worker);
                write_gauge!(
                    QueueKind::Worker.compose_with(Worker(worker)),
                    queue_depth as i64
                );
            }
        }
    );

    #[cfg(tokio_unstable)]
    metric!(
        "scheduled_tasks_total",
        "total number of tasks scheduled into the runtime",
        kind = MetricKind::Counter,
        |rt| {
            struct Overflow(bool);

            impl LabelGroup for Overflow {
                fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
                    const OVERFLOW: &LabelName = LabelName::from_str("overflow");
                    v.write_value(OVERFLOW, if self.0 { &Str("true") } else { &Str("false") });
                }
            }

            struct Remote;

            impl LabelGroup for Remote {
                fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
                    const LE: &LabelName = LabelName::from_str("worker");
                    v.write_value(LE, &Str("remote"));
                }
            }

            for worker in 0..rt.num_workers() {
                write_counter!(
                    Worker(worker).compose_with(Overflow(false)),
                    rt.worker_local_schedule_count(worker)
                );
                write_counter!(
                    Worker(worker).compose_with(Overflow(true)),
                    rt.worker_overflow_count(worker)
                );
            }
            write_counter!(
                Remote.compose_with(Overflow(false)),
                rt.remote_schedule_count()
            );
        }
    );

    #[cfg(tokio_unstable)]
    metric!(
        "budget_forced_yield_total",
        "number of tasks forced to yield after exhausting their budget",
        kind = MetricKind::Counter,
        |rt| write_counter!(NoLabels, rt.budget_forced_yield_count())
    );

    #[cfg(tokio_unstable)]
    metric!(
        "worker_mean_poll_time_seconds",
        "estimated weighted moving average of the poll time for this worker",
        kind = MetricKind::Gauge,
        |rt| for worker in 0..rt.num_workers() {
            let poll_time = rt.worker_mean_poll_time(worker);
            write_float_gauge!(Worker(worker), poll_time.as_secs_f64());
        }
    );

    #[cfg(tokio_unstable)]
    metric!(
        "worker_noop_total",
        "number of times the given worker thread woke up with no work",
        kind = MetricKind::Counter,
        |rt| for worker in 0..rt.num_workers() {
            let noops = rt.worker_noop_count(worker);
            write_counter!(Worker(worker), noops);
        }
    );

    metric!(
        "worker_park_total",
        "number of times the given worker thread has parked",
        kind = MetricKind::Counter,
        |rt| for worker in 0..rt.num_workers() {
            let count = rt.worker_park_count(worker);
            write_counter!(Worker(worker), count);
        }
    );

    metric!(
        "worker_park_unpark_total",
        "number of times the given worker thread has parked and unparked",
        kind = MetricKind::Counter,
        |rt| for worker in 0..rt.num_workers() {
            let count = rt.worker_park_unpark_count(worker);
            write_counter!(Worker(worker), count);
        }
    );

    #[cfg(tokio_unstable)]
    metric!(
        "worker_steal_total",
        "number of tasks the given worker thread has stolen",
        kind = MetricKind::Counter,
        |rt| for worker in 0..rt.num_workers() {
            let count = rt.worker_steal_count(worker);
            write_counter!(Worker(worker), count);
        }
    );

    #[cfg(tokio_unstable)]
    metric!(
        "worker_steal_operations_total",
        "number of times the given worker thread has attempted to steal tasks",
        kind = MetricKind::Counter,
        |rt| for worker in 0..rt.num_workers() {
            let count = rt.worker_steal_operations(worker);
            write_counter!(Worker(worker), count);
        }
    );

    #[cfg(tokio_unstable)]
    metric!(
        "worker_poll_time_seconds",
        "time this runtime thread has spent polling tasks",
        kind = MetricKind::Histogram,
        |rt| {
            // the final bucket is unbounded, so it is the +Inf bucket
            let le: Vec<f64> = if rt.poll_time_histogram_enabled() {
                (0..rt.poll_time_histogram_num_buckets() - 1)
                    .map(|bucket| {
                        rt.poll_time_histogram_bucket_range(bucket)
                            .end
                            .as_secs_f64()
                    })
                    .collect()
            } else {
                vec![]
            };

            for worker in 0..rt.num_workers() {
                let busy = rt.worker_total_busy_duration(worker).as_secs_f64();
                let counts: Vec<u64> = (0..le.len())
                    .map(|bucket| rt.poll_time_histogram_bucket_count(worker, bucket))
                    .collect();
                let inf = if rt.poll_time_histogram_enabled() {
                    rt.poll_time_histogram_bucket_count(worker, le.len())
                } else {
                    rt.worker_poll_count(worker)
                };
                write_histogram!(Worker(worker), &le, &counts, inf, busy);
            }
        }
    );

    // Without tokio_unstable, only the sum is available. It is a separate family so encoders
    // with native family representations preserve the full sample name.
    #[cfg(not(tokio_unstable))]
    metric!(
        "worker_poll_time_seconds_sum",
        "time this runtime thread has spent polling tasks",
        kind = MetricKind::Untyped,
        |rt| for worker in 0..rt.num_workers() {
            write_float_gauge!(
                Worker(worker),
                rt.worker_total_busy_duration(worker).as_secs_f64()
            );
        }
    );

    #[cfg(tokio_unstable)]
    #[cfg(feature = "__tokio-unstable-schedule-latency")]
    {
        // Read each bucket once so the +Inf bucket and count agree even on a busy runtime.
        let mut snapshots = Vec::new();
        for collector in runtimes {
            let rt = &collector.runtime;
            if !rt.schedule_latency_histogram_enabled() {
                continue;
            }
            for worker in 0..rt.num_workers() {
                let mut total = 0;
                let buckets: Vec<_> = (0..rt.schedule_latency_histogram_num_buckets())
                    .map(|bucket| {
                        total += rt.schedule_latency_histogram_bucket_count(worker, bucket);
                        (
                            histogram_le(rt.schedule_latency_histogram_bucket_range(bucket).end),
                            total,
                        )
                    })
                    .collect();
                snapshots.push((&collector.name, worker, buckets, total));
            }
        }
        if !snapshots.is_empty() {
            let name = MetricName::from_str("worker_schedule_latency_seconds_bucket");
            enc.write_family(
                name,
                MetricKind::Untyped,
                FamilyMeta::new().with_help("cumulative counts of task schedule latencies for this worker"),
            )?;
            for (rt_name, worker, buckets, _) in &snapshots {
                for (le, count) in buckets {
                    enc.write_counter(
                        name,
                        ComposedGroup(*rt_name, Worker(*worker).compose_with(le)),
                        *count,
                        SampleExtras::NONE,
                    )?;
                }
            }
            let name = MetricName::from_str("worker_schedule_latency_seconds_count");
            enc.write_family(
                name,
                MetricKind::Untyped,
                FamilyMeta::new().with_help("number of task schedule latencies recorded for this worker"),
            )?;
            for (rt_name, worker, _, total) in &snapshots {
                enc.write_counter(
                    name,
                    ComposedGroup(*rt_name, Worker(*worker)),
                    *total,
                    SampleExtras::NONE,
                )?;
            }
        }
    }

    #[cfg(tokio_unstable)]
    #[cfg(feature = "net")]
    {
        metric!(
            "registered_fds_total",
            "total number of file descriptors that have been registered in the runtime",
            kind = MetricKind::Counter,
            |rt| write_counter!(NoLabels, rt.io_driver_fd_registered_count())
        );
        metric!(
            "deregistered_fds_total",
            "total number of file descriptors that have been deregistered from the runtime",
            kind = MetricKind::Counter,
            |rt| write_counter!(NoLabels, rt.io_driver_fd_deregistered_count())
        );
        metric!(
            "io_ready_events_total",
            "total number of ready events the runtime's IO driver has processed",
            kind = MetricKind::Counter,
            |rt| write_counter!(NoLabels, rt.io_driver_ready_count())
        );
    }

    Ok(())
}

impl<Enc: Encoding> MetricGroup<Enc> for RuntimeCollector {
    fn collect_group_into(&self, enc: &mut Enc) -> Result<(), Enc::Err> {
        collect(std::slice::from_ref(self), enc)
    }
}

struct I64(i64);

impl LabelValue for I64 {
    fn visit<V: LabelVisitor>(&self, v: V) -> V::Output {
        v.write_int(self.0)
    }
}

#[derive(Copy, Clone)]
struct Worker(usize);

impl LabelGroup for Worker {
    fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
        const LE: &LabelName = LabelName::from_str("worker");
        v.write_value(LE, &I64(self.0 as i64));
    }
}

#[cfg(tokio_unstable)]
#[cfg(feature = "__tokio-unstable-schedule-latency")]
struct HistogramLabelLe {
    le: f64,
}

#[cfg(tokio_unstable)]
#[cfg(feature = "__tokio-unstable-schedule-latency")]
impl LabelGroup for HistogramLabelLe {
    fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
        const LE: &LabelName = LabelName::from_str("le");
        v.write_value(LE, &self.le);
    }
}

struct Str<'a>(&'a str);
impl LabelValue for Str<'_> {
    fn visit<V: LabelVisitor>(&self, v: V) -> V::Output {
        v.write_str(self.0)
    }
}

struct RuntimeName {
    name: Option<Cow<'static, str>>,
}

impl LabelGroup for RuntimeName {
    fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
        const LE: &LabelName = LabelName::from_str("runtime");
        if let Some(name) = self.name.as_deref() {
            v.write_value(LE, &Str(name));
        }
    }
}

#[derive(FixedCardinalityLabel, Clone, Copy)]
#[label(singleton = "kind")]
enum ThreadKind {
    Worker,
    BlockingIdle,
    Blocking,
}

#[derive(FixedCardinalityLabel, Clone, Copy)]
#[label(singleton = "kind")]
#[allow(unused)]
enum QueueKind {
    Worker,
    Blocking,
    Global,
}

#[cfg(test)]
mod tests {
    use measured::MetricGroup;
    use measured_prometheus_protobuf::ProtoEncoder;
    use measured_prometheus_text::{BufferedOpenMetricsEncoder, BufferedTextEncoder};
    use prost::Message;

    use crate::RuntimeCollector;

    // Decode wire fields independently of the production encoder. Unknown fields are skipped.
    #[derive(Message)]
    struct Family {
        #[prost(string, tag = "1")]
        name: String,
        #[prost(int32, tag = "3")]
        kind: i32,
        #[prost(message, repeated, tag = "4")]
        metrics: Vec<Sample>,
    }

    #[derive(Message)]
    struct Sample {
        #[prost(message, repeated, tag = "1")]
        labels: Vec<Label>,
        #[prost(message, optional, tag = "2")]
        gauge: Option<Value>,
        #[prost(message, optional, tag = "3")]
        counter: Option<Value>,
        #[prost(message, optional, tag = "5")]
        untyped: Option<Value>,
        #[prost(message, optional, tag = "7")]
        histogram: Option<Histogram>,
    }

    #[derive(Message)]
    struct Label {
        #[prost(string, tag = "1")]
        name: String,
        #[prost(string, tag = "2")]
        value: String,
    }

    #[derive(Message)]
    struct Value {
        #[prost(double, tag = "1")]
        value: f64,
    }

    #[derive(Message)]
    struct Histogram {
        #[prost(uint64, tag = "1")]
        count: u64,
        #[prost(double, tag = "2")]
        sum: f64,
        #[prost(message, repeated, tag = "3")]
        buckets: Vec<Bucket>,
    }

    #[derive(Message)]
    struct Bucket {
        #[prost(uint64, tag = "1")]
        count: u64,
        #[prost(double, tag = "2")]
        upper_bound: f64,
    }

    fn protobuf(rt: &tokio::runtime::Runtime) -> Vec<Family> {
        let mut enc = ProtoEncoder::new(Vec::new());
        RuntimeCollector::new(rt.metrics())
            .collect_group_into(&mut enc)
            .unwrap();
        enc.flush().unwrap();
        let bytes = enc.into_inner();
        let mut input = bytes.as_slice();
        let mut families = Vec::new();
        while !input.is_empty() {
            families.push(Family::decode_length_delimited(&mut input).unwrap());
        }
        families
    }

    fn family<'a>(families: &'a [Family], name: &str) -> &'a Family {
        families
            .iter()
            .find(|family| family.name == name)
            .unwrap_or_else(|| panic!("missing family {name}"))
    }

    fn label<'a>(sample: &'a Sample, name: &str) -> &'a str {
        &sample
            .labels
            .iter()
            .find(|label| label.name == name)
            .unwrap_or_else(|| panic!("missing label {name}"))
            .value
    }

    fn text(rt: &tokio::runtime::Runtime) -> String {
        let mut enc = BufferedTextEncoder::new();
        RuntimeCollector::new(rt.metrics())
            .collect_group_into(&mut enc)
            .unwrap();
        String::from_utf8(enc.finish().to_vec()).unwrap()
    }

    fn openmetrics(rt: &tokio::runtime::Runtime) -> String {
        let mut enc = BufferedOpenMetricsEncoder::new();
        RuntimeCollector::new(rt.metrics())
            .collect_group_into(&mut enc)
            .unwrap();
        String::from_utf8(enc.finish().to_vec()).unwrap()
    }

    #[cfg(tokio_unstable)]
    fn run_tasks(rt: &tokio::runtime::Runtime) {
        rt.block_on(async {
            for _ in 0..10 {
                tokio::spawn(async { tokio::task::yield_now().await })
                    .await
                    .unwrap();
            }
        });
    }

    #[test]
    fn ordinary_metrics_preserve_types_and_worker_labels() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .build()
            .unwrap();
        let output = text(&rt);
        assert!(output.contains("# TYPE alive_tasks gauge\n"));
        assert!(output.contains("# TYPE worker_park_total counter\n"));
        let families = protobuf(&rt);
        let alive = family(&families, "alive_tasks");
        assert_eq!(alive.kind, 1); // Gauge
        assert_eq!(alive.metrics[0].gauge.as_ref().unwrap().value, 0.0);
        let park = family(&families, "worker_park_total");
        assert_eq!(park.kind, 0); // Counter
        let workers: Vec<_> = park
            .metrics
            .iter()
            .map(|sample| {
                assert!(sample.counter.as_ref().unwrap().value >= 0.0);
                label(sample, "worker")
            })
            .collect();
        assert_eq!(workers, ["0", "1"]);
        #[cfg(tokio_unstable)]
        assert!(output.contains("scheduled_tasks_total{worker=\"remote\",overflow=\"false\"}"));
    }

    #[cfg(not(tokio_unstable))]
    #[test]
    fn stable_poll_sum_keeps_its_full_name_in_every_format() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        for (output, kind) in [(text(&rt), "untyped"), (openmetrics(&rt), "unknown")] {
            assert!(output.contains(&format!("# TYPE worker_poll_time_seconds_sum {kind}\n")));
            assert!(output.contains("worker_poll_time_seconds_sum{worker=\"0\"} "));
        }
        let families = protobuf(&rt);
        assert!(
            !families
                .iter()
                .any(|family| family.name == "worker_poll_time_seconds")
        );
        let sum = family(&families, "worker_poll_time_seconds_sum");
        assert_eq!(sum.kind, 3); // Untyped
        assert_eq!(sum.metrics.len(), 1);
        assert_eq!(label(&sum.metrics[0], "worker"), "0");
        assert!(sum.metrics[0].untyped.as_ref().unwrap().value >= 0.0);
    }

    #[cfg(tokio_unstable)]
    #[test]
    fn poll_histogram_exports_observations_as_a_native_histogram() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_metrics_poll_time_histogram()
            .build()
            .unwrap();
        run_tasks(&rt);
        assert!(text(&rt).contains("# TYPE worker_poll_time_seconds histogram\n"));
        assert!(openmetrics(&rt).contains("# TYPE worker_poll_time_seconds histogram\n"));
        let families = protobuf(&rt);
        let polls = family(&families, "worker_poll_time_seconds");
        assert_eq!(polls.kind, 4); // Histogram
        assert_eq!(polls.metrics.len(), 1);
        assert_eq!(label(&polls.metrics[0], "worker"), "0");
        let histogram = polls.metrics[0].histogram.as_ref().unwrap();
        assert!(histogram.count > 0);
        assert!(histogram.sum >= 0.0);
        assert!(!histogram.buckets.is_empty());
        assert!(
            histogram
                .buckets
                .windows(2)
                .all(|pair| pair[0].count <= pair[1].count && pair[0].upper_bound < pair[1].upper_bound)
        );
        assert!(histogram.buckets.last().unwrap().count <= histogram.count);
    }

    #[cfg(all(tokio_unstable, feature = "__tokio-unstable-schedule-latency"))]
    mod schedule_latency {
        use super::*;

        #[test]
        fn disabled_histogram_is_omitted_in_every_format() {
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();
            assert!(!text(&rt).contains("schedule_latency"));
            assert!(!openmetrics(&rt).contains("schedule_latency"));
            assert!(
                !protobuf(&rt)
                    .iter()
                    .any(|family| family.name.contains("schedule_latency"))
            );
        }

        #[test]
        fn enabled_histogram_keeps_names_labels_and_cumulative_counts() {
            // A current-thread runtime stops polling tasks when block_on returns, so separate
            // family collections see the same completed workload without timing assumptions.
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_metrics_schedule_latency_histogram()
                .build()
                .unwrap();
            run_tasks(&rt);
            for (output, kind) in [(text(&rt), "untyped"), (openmetrics(&rt), "unknown")] {
                assert!(output.contains(&format!(
                    "# TYPE worker_schedule_latency_seconds_bucket {kind}\n"
                )));
                assert!(output.contains(&format!(
                    "# TYPE worker_schedule_latency_seconds_count {kind}\n"
                )));
                assert!(output.contains("worker_schedule_latency_seconds_bucket{worker=\"0\",le=\"+Inf\"}"));
            }
            let families = protobuf(&rt);
            assert!(
                !families
                    .iter()
                    .any(|family| family.name == "worker_schedule_latency_seconds")
            );
            let buckets = family(&families, "worker_schedule_latency_seconds_bucket");
            let counts = family(&families, "worker_schedule_latency_seconds_count");
            assert_eq!(buckets.kind, 3);
            assert_eq!(counts.kind, 3);
            assert_eq!(counts.metrics.len(), 1);
            assert_eq!(label(&counts.metrics[0], "worker"), "0");
            let total = counts.metrics[0].untyped.as_ref().unwrap().value;
            assert!(
                total > 0.0,
                "the completed task workload must produce observations"
            );
            assert_eq!(
                buckets.metrics.len(),
                rt.metrics().schedule_latency_histogram_num_buckets()
            );
            let mut previous = 0.0;
            let mut previous_bound = f64::NEG_INFINITY;
            for sample in &buckets.metrics {
                assert_eq!(label(sample, "worker"), "0");
                let bound: f64 = label(sample, "le").parse().unwrap();
                let value = sample.untyped.as_ref().unwrap().value;
                assert!(bound > previous_bound);
                assert!(value >= previous && value <= total);
                previous = value;
                previous_bound = bound;
            }
            assert_eq!(previous_bound, f64::INFINITY);
            assert_eq!(previous, total);
        }
        #[test]
        fn counts_match_buckets_when_observations_arrive_during_encoding() {
            struct Writer<'a> {
                runtime: &'a tokio::runtime::Runtime,
                output: Vec<u8>,
                advanced: bool,
            }
            impl std::io::Write for Writer<'_> {
                fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                    self.output.extend_from_slice(bytes);
                    if !self.advanced
                        && self
                            .output
                            .windows(b"worker_schedule_latency_seconds_count".len())
                            .any(|w| w == b"worker_schedule_latency_seconds_count")
                    {
                        self.advanced = true;
                        run_tasks(self.runtime);
                    }
                    Ok(bytes.len())
                }
                fn flush(&mut self) -> std::io::Result<()> {
                    Ok(())
                }
            }
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_metrics_schedule_latency_histogram()
                .build()
                .unwrap();
            run_tasks(&runtime);
            let writer = Writer {
                runtime: &runtime,
                output: Vec::new(),
                advanced: false,
            };
            let mut encoder = measured_prometheus_text::TextEncoder::from_writer(writer);
            RuntimeCollector::new(runtime.metrics())
                .collect_group_into(&mut encoder)
                .unwrap();
            let writer = encoder.into_inner();
            assert!(writer.advanced);
            let output = String::from_utf8(writer.output).unwrap();
            let value = |prefix: &str| {
                output
                    .lines()
                    .find(|line| line.starts_with(prefix))
                    .unwrap()
                    .rsplit(' ')
                    .next()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
            };
            let count = value("worker_schedule_latency_seconds_count{");
            assert_eq!(
                value("worker_schedule_latency_seconds_bucket{worker=\"0\",le=\"+Inf\"}"),
                count
            );
            let current: u64 = (0..runtime.metrics().schedule_latency_histogram_num_buckets())
                .map(|bucket| {
                    runtime
                        .metrics()
                        .schedule_latency_histogram_bucket_count(0, bucket)
                })
                .sum();
            assert!(
                current > count,
                "the writer must cause new observations after the bucket snapshot"
            );
        }
    }
}
