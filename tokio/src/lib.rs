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

// /// A collector which exports the current state of tokio metrics including, with the given name as a label
// pub struct NamedRuntimesCollector {
//     runtime: RuntimeCollector,
//     name: Option<Cow<'static, str>>,
// }

// impl NamedRuntimesCollector {
//     /// Create a `RuntimeCollector` with the given process id and namespace.
//     pub fn new(
//         runtime: RuntimeMetrics,
//         name: impl Into<Cow<'static, str>>,
//     ) -> NamedRuntimesCollector {
//         RuntimeCollector::new(runtime).with_name(name)
//     }

//     /// Return a `RuntimeCollector` of the calling process.
//     ///
//     /// # Panics
//     ///
//     /// This will panic if called outside the context of a Tokio runtime. That means that you must
//     /// call this on one of the threads **being run by the runtime**, or from a thread with an active
//     /// `EnterGuard`. Calling this from within a thread created by `std::thread::spawn` (for example)
//     /// will cause a panic unless that thread has an active `EnterGuard`.
//     pub fn current(name: impl Into<Cow<'static, str>>) -> NamedRuntimesCollector {
//         RuntimeCollector::current().with_name(name)
//     }
// }

// impl<Enc: Encoding> MetricGroup<Enc> for NamedRuntimesCollector {
//     fn collect_group_into(&self, encoder: &mut Enc) -> Result<(), <Enc as Encoding>::Err> {
//         let name = self.name.as_deref();
//         self.runtime
//             .collect_group_into(&mut WithRuntimeLabel { name, encoder })
//     }
// }

// struct WithRuntimeLabel<'a, 'b, Enc> {
//     name: Option<&'a str>,
//     encoder: &'b mut Enc,
// }

// impl<'a, 'b, Enc: Encoding> Encoding for WithRuntimeLabel<'a, 'b, Enc> {
//     type Err = Enc::Err;

//     fn write_help(
//         &mut self,
//         name: impl measured::metric::name::MetricNameEncoder,
//         help: &str,
//     ) -> Result<(), Self::Err> {
//         self.encoder.write_help(name, help)
//     }

//     fn write_metric_value(
//         &mut self,
//         name: impl measured::metric::name::MetricNameEncoder,
//         labels: impl LabelGroup,
//         value: MetricValue,
//     ) -> Result<(), Self::Err> {
//         self.encoder.write_metric_value(
//             name,
//             ComposedGroup(
//                 RuntimeName {
//                     name: Some(self.name),
//                 },
//                 labels,
//             ),
//             value,
//         )
//     }
// }

/// A collector which exports the current state of tokio metrics including
pub struct RuntimeCollector {
    runtime: RuntimeMetrics,
    name: RuntimeName,
}

impl RuntimeCollector {
    /// Create a `RuntimeCollector` with the given process id and namespace.
    pub fn new(runtime: RuntimeMetrics) -> Self {
        RuntimeCollector {
            runtime,
            name: RuntimeName { name: None },
        }
    }

    /// Return a `RuntimeCollector` of the calling process.
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

    fn histogram_le(&self, bucket: usize) -> HistogramLabelLe {
        let le = self.runtime.poll_count_histogram_bucket_range(bucket).end;
        let le = if le == Duration::from_nanos(u64::MAX) {
            f64::INFINITY
        } else {
            le.as_secs_f64()
        };
        HistogramLabelLe { le }
    }
}

macro_rules! metric {
    ($enc:expr, $runtime:expr, $name:literal, $help:literal, $expr:expr) => {{
        #![allow(unused_macros)]
        const NAME: &MetricName = MetricName::from_str($name);
        $enc.write_help(NAME, $help)?;
        macro_rules! write_int {
            ($labels:expr, $val:expr) => {
                $enc.write_metric_value(
                    NAME,
                    ComposedGroup($runtime, $labels),
                    MetricValue::Int($val),
                )?
            };
            ($suffix:expr, $labels:expr, $val:expr) => {
                $enc.write_metric_value(
                    NAME.with_suffix($suffix),
                    ComposedGroup($runtime, $labels),
                    MetricValue::Int($val),
                )?
            };
        }
        macro_rules! write_float {
            ($labels:expr, $val:expr) => {
                $enc.write_metric_value(
                    NAME,
                    ComposedGroup($runtime, $labels),
                    MetricValue::Float($val),
                )?
            };
            ($suffix:expr, $labels:expr, $val:expr) => {
                $enc.write_metric_value(
                    NAME.with_suffix($suffix),
                    ComposedGroup($runtime, $labels),
                    MetricValue::Float($val),
                )?
            };
        }
        $expr
    }};
}

impl<Enc: Encoding> MetricGroup<Enc> for RuntimeCollector {
    fn collect_group_into(&self, enc: &mut Enc) -> Result<(), Enc::Err> {
        // const IDLE_BLOCKING_THREADS: &MetricName = MetricName::from_str("idle_blocking_threads");
        // const REMOTE_SCHEDULE: &MetricName = MetricName::from_str("remote_scheduled_tasks_total");
        // const BUDGET: &MetricName = MetricName::from_str("budget_forced_yield_total");
        // const WORKER_PARK: &MetricName = MetricName::from_str("workers_parked_total");

        let workers = self.runtime.num_workers();

        metric!(
            enc,
            &self.name,
            "worker_threads",
            "number of worker threads used by the runtime",
            write_int!(NoLabels, workers as i64)
        );

        metric!(
            enc,
            &self.name,
            "blocking_threads",
            "number of blocking threads used by the runtime",
            write_int!(NoLabels, self.runtime.num_blocking_threads() as i64)
        );

        metric!(
            enc,
            &self.name,
            "active_tasks",
            "number of active tasks spawned in the runtime",
            write_int!(NoLabels, self.runtime.active_tasks_count() as i64)
        );

        metric!(
            enc,
            &self.name,
            "worker_queue_depth",
            "number of tasks currently scheduled in the given worker's local queue",
            for worker in 0..workers {
                let queue_depth = self.runtime.worker_local_queue_depth(worker);
                write_int!(WorkerLabels { worker }, queue_depth as i64);
            }
        );

        metric!(
            enc,
            &self.name,
            "worker_mean_poll_time_seconds",
            "estimated weighted moving average of the poll time for this worker",
            for worker in 0..workers {
                let poll_time = self.runtime.worker_mean_poll_time(worker);
                write_float!(WorkerLabels { worker }, poll_time.as_secs_f64());
            }
        );

        metric!(
            enc,
            &self.name,
            "worker_busy_time_seconds_total",
            "amount of time the given worker thread has been busy",
            for worker in 0..workers {
                let busy_time = self.runtime.worker_total_busy_duration(worker);
                write_float!(WorkerLabels { worker }, busy_time.as_secs_f64());
            }
        );

        if self.runtime.poll_count_histogram_enabled() {
            let buckets = self.runtime.poll_count_histogram_num_buckets();
            metric!(
                enc,
                &self.name,
                "worker_poll_time_seconds",
                "time this runtime thread has spent polling tasks",
                for worker in 0..workers {
                    let worker_label = WorkerLabels { worker };
                    let mut total = 0;
                    for bucket in 0..buckets {
                        total += self
                            .runtime
                            .poll_count_histogram_bucket_count(worker, bucket);

                        let le = self.histogram_le(bucket);
                        write_int!(Bucket, ComposedGroup(worker_label, le), total as i64);
                    }
                    write_int!(Count, worker_label, total as i64);
                }
            );
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
