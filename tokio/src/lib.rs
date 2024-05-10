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
//!     #[metric(namespace = "tokio")]
//!     #[metric(init = measured_tokio::RuntimeCollector::current())]
//!     tokio: measured_tokio::RuntimeCollector,
//!
//!     // other metrics
//! }}
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

use std::{borrow::Cow, sync::RwLock, time::Duration};

use measured::{
    label::{ComposedGroup, LabelGroupVisitor, LabelName, LabelValue, LabelVisitor, NoLabels},
    metric::{
        group::{Encoding, MetricValue},
        name::{Bucket, Count, MetricName},
    },
    LabelGroup, MetricGroup,
};
use tokio::runtime::RuntimeMetrics;

/// A collector which exports the current state of tokio metrics including, with the given name as a label
pub struct NamedRuntimesCollector {
    runtimes: RwLock<Vec<RuntimeCollector>>,
}

impl NamedRuntimesCollector {
    /// Create a new empty `NamedRuntimesCollector`
    pub fn new() -> Self {
        Self {
            runtimes: RwLock::new(vec![]),
        }
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
        self.runtimes
            .write()
            .unwrap()
            .push(RuntimeCollector::current().with_name(name))
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

/// A collector which exports the current state of tokio metrics including
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
            name: RuntimeName {
                name: Some(name.into()),
            },
        }
    }
}

fn histogram_le(rt: &RuntimeMetrics, bucket: usize) -> HistogramLabelLe {
    let le = rt.poll_count_histogram_bucket_range(bucket).end;
    let le = if le == Duration::from_nanos(u64::MAX) {
        f64::INFINITY
    } else {
        le.as_secs_f64()
    };
    HistogramLabelLe { le }
}

fn collect<Enc: Encoding>(runtimes: &[RuntimeCollector], enc: &mut Enc) -> Result<(), Enc::Err> {
    macro_rules! metric {
        ($name:literal, $help:literal, |$rt:ident| $expr:expr) => {{
            #![allow(unused_macros)]
            const NAME: &MetricName = MetricName::from_str($name);
            enc.write_help(NAME, $help)?;
            for rt in runtimes {
                let rt_name = &rt.name;
                macro_rules! write_int {
                    ($labels:expr, $val:expr) => {
                        enc.write_metric_value(
                            NAME,
                            ComposedGroup(rt_name, $labels),
                            MetricValue::Int($val),
                        )?
                    };
                    ($suffix:expr, $labels:expr, $val:expr) => {
                        enc.write_metric_value(
                            NAME.with_suffix($suffix),
                            ComposedGroup(rt_name, $labels),
                            MetricValue::Int($val),
                        )?
                    };
                }
                macro_rules! write_float {
                    ($labels:expr, $val:expr) => {
                        enc.write_metric_value(
                            NAME,
                            ComposedGroup(rt_name, $labels),
                            MetricValue::Float($val),
                        )?
                    };
                    ($suffix:expr, $labels:expr, $val:expr) => {
                        enc.write_metric_value(
                            NAME.with_suffix($suffix),
                            ComposedGroup(rt_name, $labels),
                            MetricValue::Float($val),
                        )?
                    };
                }
                let $rt = &rt.runtime;
                ($expr)
            }
        }};
    }

    metric!(
        "worker_threads",
        "number of worker threads used by the runtime",
        |rt| write_int!(NoLabels, rt.num_workers() as i64)
    );

    metric!(
        "blocking_threads",
        "number of blocking threads used by the runtime",
        |rt| write_int!(NoLabels, rt.num_blocking_threads() as i64)
    );

    metric!(
        "active_tasks",
        "number of active tasks spawned in the runtime",
        |rt| write_int!(NoLabels, rt.active_tasks_count() as i64)
    );

    metric!(
        "worker_queue_depth",
        "number of tasks currently scheduled in the given worker's local queue",
        |rt| for worker in 0..rt.num_workers() {
            let queue_depth = rt.worker_local_queue_depth(worker);
            write_int!(WorkerLabels { worker }, queue_depth as i64);
        }
    );

    metric!(
        "worker_mean_poll_time_seconds",
        "estimated weighted moving average of the poll time for this worker",
        |rt| for worker in 0..rt.num_workers() {
            let poll_time = rt.worker_mean_poll_time(worker);
            write_float!(WorkerLabels { worker }, poll_time.as_secs_f64());
        }
    );

    metric!(
        "worker_busy_time_seconds_total",
        "amount of time the given worker thread has been busy",
        |rt| for worker in 0..rt.num_workers() {
            let busy_time = rt.worker_total_busy_duration(worker);
            write_float!(WorkerLabels { worker }, busy_time.as_secs_f64());
        }
    );

    metric!(
        "worker_poll_time_seconds",
        "time this runtime thread has spent polling tasks",
        |rt| if rt.poll_count_histogram_enabled() {
            let buckets = rt.poll_count_histogram_num_buckets();
            for worker in 0..rt.num_workers() {
                let worker_label = WorkerLabels { worker };
                let mut total = 0;
                for bucket in 0..buckets {
                    let le = histogram_le(rt, bucket);
                    total += rt.poll_count_histogram_bucket_count(worker, bucket);
                    write_int!(Bucket, ComposedGroup(worker_label, le), total as i64);
                }
                write_int!(Count, worker_label, total as i64);
            }
        }
    );

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

struct F64(f64);
impl LabelValue for F64 {
    fn visit<V: LabelVisitor>(&self, v: V) -> V::Output {
        v.write_float(self.0)
    }
}

#[derive(Copy, Clone)]
struct WorkerLabels {
    worker: usize,
}

impl LabelGroup for WorkerLabels {
    fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
        const LE: &LabelName = LabelName::from_str("worker");
        v.write_value(LE, &I64(self.worker as i64));
    }
}

struct HistogramLabelLe {
    le: f64,
}

impl LabelGroup for HistogramLabelLe {
    fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
        const LE: &LabelName = LabelName::from_str("le");
        v.write_value(LE, &F64(self.le));
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

// #[cfg(test)]
// mod tests {
//     use std::io::Write;

//     use measured::{text::BufferedTextEncoder, MetricGroup};
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
