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
//! let metrics = MyAppMetrics::new();
//!
//! // when you run metrics.collect_group_into(...), you will sample tokio to get runtime state.
//!
//! # drop(metrics);
//! ```

use std::{borrow::Cow, time::Duration};

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
pub struct NamedRuntimeCollector {
    runtime: RuntimeCollector,
    name: Cow<'static, str>,
}

impl NamedRuntimeCollector {
    /// Create a `RuntimeCollector` with the given process id and namespace.
    pub fn new(
        runtime: RuntimeMetrics,
        name: impl Into<Cow<'static, str>>,
    ) -> NamedRuntimeCollector {
        RuntimeCollector::new(runtime).with_name(name)
    }

    /// Return a `RuntimeCollector` of the calling process.
    ///
    /// # Panics
    ///
    /// This will panic if called outside the context of a Tokio runtime. That means that you must
    /// call this on one of the threads **being run by the runtime**, or from a thread with an active
    /// `EnterGuard`. Calling this from within a thread created by `std::thread::spawn` (for example)
    /// will cause a panic unless that thread has an active `EnterGuard`.
    pub fn current(name: impl Into<Cow<'static, str>>) -> NamedRuntimeCollector {
        RuntimeCollector::current().with_name(name)
    }
}

impl<Enc: Encoding> MetricGroup<Enc> for NamedRuntimeCollector
where
    RuntimeCollector: for<'a, 'b> MetricGroup<WithRuntimeLabel<'a, 'b, Enc>>,
{
    fn collect_group_into(&self, encoder: &mut Enc) -> Result<(), <Enc as Encoding>::Err> {
        let name: &str = &self.name;
        self.runtime
            .collect_group_into(&mut WithRuntimeLabel { name, encoder })
    }
}

struct WithRuntimeLabel<'a, 'b, Enc> {
    name: &'a str,
    encoder: &'b mut Enc,
}

impl<'a, 'b, Enc: Encoding> Encoding for WithRuntimeLabel<'a, 'b, Enc> {
    type Err = Enc::Err;

    fn write_help(
        &mut self,
        name: impl measured::metric::name::MetricNameEncoder,
        help: &str,
    ) -> Result<(), Self::Err> {
        self.encoder.write_help(name, help)
    }

    fn write_metric_value(
        &mut self,
        name: impl measured::metric::name::MetricNameEncoder,
        labels: impl LabelGroup,
        value: MetricValue,
    ) -> Result<(), Self::Err> {
        self.encoder.write_metric_value(
            name,
            ComposedGroup(RuntimeName { name: self.name }, labels),
            value,
        )
    }
}

/// A collector which exports the current state of tokio metrics including
pub struct RuntimeCollector {
    runtime: RuntimeMetrics,
}

impl RuntimeCollector {
    /// Create a `RuntimeCollector` with the given process id and namespace.
    pub fn new(runtime: RuntimeMetrics) -> RuntimeCollector {
        RuntimeCollector { runtime }
    }

    /// Return a `RuntimeCollector` of the calling process.
    ///
    /// # Panics
    ///
    /// This will panic if called outside the context of a Tokio runtime. That means that you must
    /// call this on one of the threads **being run by the runtime**, or from a thread with an active
    /// `EnterGuard`. Calling this from within a thread created by `std::thread::spawn` (for example)
    /// will cause a panic unless that thread has an active `EnterGuard`.
    pub fn current() -> RuntimeCollector {
        RuntimeCollector::new(tokio::runtime::Handle::current().metrics())
    }

    pub fn with_name(self, name: impl Into<Cow<'static, str>>) -> NamedRuntimeCollector {
        NamedRuntimeCollector {
            runtime: self,
            name: name.into(),
        }
    }
}

impl<Enc: Encoding> MetricGroup<Enc> for RuntimeCollector {
    fn collect_group_into(&self, enc: &mut Enc) -> Result<(), Enc::Err> {
        // const IDLE_BLOCKING_THREADS: &MetricName = MetricName::from_str("idle_blocking_threads");
        // const REMOTE_SCHEDULE: &MetricName = MetricName::from_str("remote_scheduled_tasks_total");
        // const BUDGET: &MetricName = MetricName::from_str("budget_forced_yield_total");
        // const WORKER_PARK: &MetricName = MetricName::from_str("workers_parked_total");

        const WORKERS: &MetricName = MetricName::from_str("worker_threads");
        let workers = self.runtime.num_workers();
        enc.write_help(WORKERS, "number of worker threads used by the runtime")?;
        enc.write_metric_value(WORKERS, NoLabels, MetricValue::Int(workers as i64))?;

        const QUEUE_DEPTH: &MetricName = MetricName::from_str("worker_queue_depth");
        enc.write_help(
            QUEUE_DEPTH,
            "number of tasks currently scheduled in the given worker's local queue",
        )?;
        for worker in 0..workers {
            let queue_depth = self.runtime.worker_local_queue_depth(worker);
            enc.write_metric_value(
                QUEUE_DEPTH,
                WorkerLabels {
                    worker: worker as i64,
                },
                MetricValue::Int(queue_depth as i64),
            )?;
        }

        const POLL_TIME: &MetricName = MetricName::from_str("worker_mean_poll_time_seconds");
        enc.write_help(
            POLL_TIME,
            "estimated weighted moving average of the poll time for this worker",
        )?;
        for worker in 0..workers {
            let poll_time = self.runtime.worker_mean_poll_time(worker);
            enc.write_metric_value(
                POLL_TIME,
                WorkerLabels {
                    worker: worker as i64,
                },
                MetricValue::Float(poll_time.as_secs_f64()),
            )?;
        }

        const BUSY_TIME: &MetricName = MetricName::from_str("worker_busy_time_seconds_total");
        enc.write_help(
            BUSY_TIME,
            "amount of time the given worker thread has been busy",
        )?;
        for worker in 0..workers {
            let busy_time = self.runtime.worker_total_busy_duration(worker);
            enc.write_metric_value(
                BUSY_TIME,
                WorkerLabels {
                    worker: worker as i64,
                },
                MetricValue::Float(busy_time.as_secs_f64()),
            )?;
        }

        const BLOCKING_THREADS: &MetricName = MetricName::from_str("blocking_threads");
        enc.write_help(
            BLOCKING_THREADS,
            "number of blocking threads used by the runtime",
        )?;
        enc.write_metric_value(
            BLOCKING_THREADS,
            NoLabels,
            MetricValue::Int(self.runtime.num_blocking_threads() as i64),
        )?;

        const ACTIVE_TASKS: &MetricName = MetricName::from_str("active_tasks");
        enc.write_help(
            ACTIVE_TASKS,
            "number of active tasks spawned in the runtime",
        )?;
        enc.write_metric_value(
            ACTIVE_TASKS,
            NoLabels,
            MetricValue::Int(self.runtime.active_tasks_count() as i64),
        )?;

        if self.runtime.poll_count_histogram_enabled() {
            const POLL_TIME: &MetricName = MetricName::from_str("worker_poll_time_seconds");
            enc.write_help(
                POLL_TIME,
                "time this runtime thread has spent polling tasks",
            )?;
            let buckets = self.runtime.poll_count_histogram_num_buckets();
            for worker in 0..workers {
                let worker_label = WorkerLabels {
                    worker: worker as i64,
                };
                let mut total = 0;
                for bucket in 0..buckets {
                    let le = self.runtime.poll_count_histogram_bucket_range(bucket).end;
                    let le = if le == Duration::from_nanos(u64::MAX) {
                        f64::INFINITY
                    } else {
                        le.as_secs_f64()
                    };
                    total += self
                        .runtime
                        .poll_count_histogram_bucket_count(worker, bucket);
                    enc.write_metric_value(
                        POLL_TIME.with_suffix(Bucket),
                        ComposedGroup(worker_label, HistogramLabelLe { le }),
                        MetricValue::Int(total as i64),
                    )?
                }
                enc.write_metric_value(
                    POLL_TIME.with_suffix(Count),
                    worker_label,
                    MetricValue::Int(total as i64),
                )?
            }
        }

        Ok(())
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
    worker: i64,
}

impl LabelGroup for WorkerLabels {
    fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
        const LE: &LabelName = LabelName::from_str("worker");
        v.write_value(LE, &I64(self.worker));
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

#[derive(Copy, Clone)]
struct RuntimeName<'a> {
    name: &'a str,
}

impl LabelGroup for RuntimeName<'_> {
    fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
        const LE: &LabelName = LabelName::from_str("runtime");
        v.write_value(LE, &Str(self.name));
    }
}

// #[cfg(test)]
// mod tests {
//     use std::io::Write;

//     use measured::{text::BufferedTextEncoder, MetricGroup};
//     use tokio::task::JoinSet;

//     use crate::RuntimeCollector;

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

//             let rt = RuntimeCollector::current().with_name("blah");
//             let mut enc = BufferedTextEncoder::new();
//             rt.collect_group_into(&mut enc).unwrap();
//             std::io::stdout().write_all(&enc.finish()).unwrap();
//         })
//     }
// }
