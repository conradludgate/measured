//! Monitor a tokio runtime.
//!
//! # Usage
//!
//! ```
//! use measured::MetricGroup;
//!
//! #[derive(MetricGroup)]
//! #[metric(new())]
//! struct MyAppMetrics {
//!     #[cfg(tokio_unstable)]
//!     #[metric(namespace = "tokio")]
//!     #[metric(init = measured_tokio::RuntimeCollector::current())]
//!     tokio: measured_tokio::RuntimeCollector,
//!     // other metrics
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let metrics = MyAppMetrics::new();
//!
//!     // when you run metrics.collect_group_into(...), you will sample tokio to get runtime state.
//!
//!     # drop(metrics);
//! }
//! ```

use std::borrow::Cow;
use std::sync::RwLock;

use measured::label::{ComposedGroup, LabelGroupVisitor, LabelName, LabelValue, LabelVisitor, NoLabels};
use measured::metric::group::{Encoding, MetricKind, SampleExtras};
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

    /// Inserts a `RuntimeCollector` with the given runtime.
    pub fn add(&self, rt: RuntimeMetrics, name: impl Into<Cow<'static, str>>) {
        self.runtimes
            .write()
            .unwrap()
            .push(RuntimeCollector::new(rt).with_name(name))
    }

    /// Inserts a `RuntimeCollector` for the current runtime.
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
    /// Create a `RuntimeCollector` with the given runtime.
    pub fn new(runtime: RuntimeMetrics) -> Self {
        RuntimeCollector {
            runtime,
            name: RuntimeName { name: None },
        }
    }

    /// Return a `RuntimeCollector` for the current runtime.
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
            enc.write_family(NAME, Some($help), $kind)?;
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
                Some("cumulative counts of task schedule latencies for this worker"),
                MetricKind::Untyped,
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
                Some("number of task schedule latencies recorded for this worker"),
                MetricKind::Untyped,
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
    use measured_prometheus_text::BufferedTextEncoder;

    use crate::RuntimeCollector;

    #[test]
    fn types() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .build()
            .unwrap();

        let mut enc = BufferedTextEncoder::new();
        RuntimeCollector::new(rt.metrics())
            .collect_group_into(&mut enc)
            .unwrap();
        let output = String::from_utf8(enc.finish().to_vec()).unwrap();

        assert!(output.contains("# TYPE alive_tasks gauge\n"));
        assert!(output.contains("# TYPE worker_park_total counter\n"));

        #[cfg(tokio_unstable)]
        assert!(output.contains("scheduled_tasks_total{worker=\"remote\",overflow=\"false\"}"));

        #[cfg(tokio_unstable)]
        {
            assert!(output.contains("# TYPE worker_poll_time_seconds histogram\n"));
            assert!(output.contains("worker_poll_time_seconds_bucket{worker=\"0\",le=\"+Inf\"} "));
            assert!(output.contains("worker_poll_time_seconds_count{worker=\"0\"} "));
        }
    }
}

#[cfg(test)]
#[cfg(tokio_unstable)]
#[cfg(feature = "__tokio-unstable-schedule-latency")]
mod schedule_latency_tests {
    use measured::MetricGroup;
    use measured_prometheus_text::BufferedTextEncoder;

    use crate::RuntimeCollector;

    fn encode(rt: &tokio::runtime::Runtime) -> String {
        rt.block_on(async {
            for _ in 0..10 {
                tokio::spawn(async {}).await.unwrap();
            }
        });

        let mut enc = BufferedTextEncoder::new();
        RuntimeCollector::new(rt.metrics())
            .collect_group_into(&mut enc)
            .unwrap();
        String::from_utf8(enc.finish().to_vec()).unwrap()
    }

    #[test]
    fn disabled() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .build()
            .unwrap();
        assert!(!encode(&rt).contains("schedule_latency"));
    }

    #[test]
    fn enabled() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_metrics_schedule_latency_histogram()
            .build()
            .unwrap();
        let output = encode(&rt);
        assert!(output.contains("worker_schedule_latency_seconds_bucket{worker=\"0\",le=\"+Inf\"}"));
        assert!(output.contains("worker_schedule_latency_seconds_count{worker=\"1\"}"));
    }
}

// #[cfg(test)]
// mod tests {
//     use std::io::Write;

//     use measured::MetricGroup;
//     use measured_prometheus_text::BufferedTextEncoder;
//     use tokio::task::JoinSet;

//     use crate::{NamedRuntimesCollector, RuntimeCollector};

//     #[test]
//     fn demo() {
//         let rt = tokio::runtime::Builder::new_multi_thread()
//             .worker_threads(4)
//             .metrics_poll_count_histogram_scale(tokio::runtime::HistogramScale::Log)
//             .enable_metrics_poll_count_histogram()
//             .enable_all()
//             .build()
//             .unwrap();
//         rt.block_on(async {
//             let mut js = JoinSet::new();
//             for _ in 0..100 {
//                 js.spawn(async {
//                     for _ in 0..100 {
//                         tokio::task::yield_now().await;
//                     }
//                 });
//             }
//             while js.join_next().await.is_some() {}
//         });

//         let rt2 = tokio::runtime::Builder::new_multi_thread()
//             .worker_threads(8)
//             .metrics_poll_count_histogram_scale(tokio::runtime::HistogramScale::Linear)
//             .enable_metrics_poll_count_histogram()
//             .enable_all()
//             .build()
//             .unwrap();
//         rt2.block_on(async {
//             let mut js = JoinSet::new();
//             for _ in 0..100 {
//                 js.spawn(async {
//                     for _ in 0..100 {
//                         tokio::task::yield_now().await;
//                     }
//                 });
//             }
//             while js.join_next().await.is_some() {}
//         });

//         let collector = NamedRuntimesCollector::new();
//         collector.add(rt.metrics(), "foo");
//         collector.add(rt2.metrics(), "bar");

//         let mut enc = BufferedTextEncoder::new();
//         collector.collect_group_into(&mut enc).unwrap();
//         std::io::stdout().write_all(&enc.finish()).unwrap();
//     }
// }
