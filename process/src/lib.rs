//! Monitor a process.
//!
//! This crate only supports **Linux** platform.
//!
//! # Usage
//!
//! ```
//! use measured::MetricGroup;
//!
//! #[derive(MetricGroup)]
//! #[metric(new())]
//! struct MyAppMetrics {
//!     #[cfg(target_os = "linux")]
//!     #[metric(namespace = "process")]
//!     #[metric(init = measured_process::ProcessCollector::for_self())]
//!     process: measured_process::ProcessCollector,
//!
//!     // other metrics
//! }}
//!
//! let metrics = MyAppMetrics::new();
//!
//! // when you run metrics.collect_group_into(...), you will sample procfs to get process stats.
//!
//! # drop(metrics);
//! ```

use std::sync::OnceLock;

use libc::pid_t;
use measured::{
    label::NoLabels,
    metric::{
        counter::CounterState,
        gauge::GaugeState,
        group::{Encoding, MetricValue},
        name::MetricName,
        MetricEncoding,
    },
    MetricGroup,
};

// Copyright 2019 TiKV Project Authors. Licensed under Apache-2.0.
// https://github.com/tikv/rust-prometheus/blob/f49c724df0e123520554664436da68e555593af0/src/process_collector.rs
// With modifications by Conrad Ludgate for the transition to measured.

/// A collector which exports the current state of process metrics including
/// CPU, memory and file descriptor usage, thread count, as well as the process
/// start time for the given process id.
pub struct ProcessCollector {
    pid: pid_t,
    start_time: Option<i64>,
}

impl ProcessCollector {
    /// Create a `ProcessCollector` with the given process id and namespace.
    pub fn new(pid: pid_t) -> ProcessCollector {
        // proc_start_time init once because it is immutable
        let mut start_time = None;
        #[cfg(target_os = "linux")]
        if let Ok(boot_time) = procfs::boot_time_secs() {
            if let Ok(stat) = procfs::process::Process::myself().and_then(|p| p.stat()) {
                start_time = Some(stat.starttime as i64 / clk_tck() + boot_time as i64);
            }
        }

        ProcessCollector { pid, start_time }
    }

    /// Return a `ProcessCollector` of the calling process.
    pub fn for_self() -> ProcessCollector {
        let pid = unsafe { libc::getpid() };
        ProcessCollector::new(pid)
    }
}

impl<Enc: Encoding> MetricGroup<Enc> for ProcessCollector
where
    CounterState: MetricEncoding<Enc>,
    GaugeState: MetricEncoding<Enc>,
{
    fn collect_group_into(&self, enc: &mut Enc) -> Result<(), Enc::Err> {
        #[cfg(target_os = "linux")]
        {
            let Ok(p) = procfs::process::Process::new(self.pid) else {
                // we can't construct a Process object, so there's no stats to gather
                return Ok(());
            };

            // file descriptors
            if let Ok(fd_count) = p.fd_count() {
                let fd = MetricName::from_str("open_fds");
                enc.write_help(fd, "Number of open file descriptors.")?;
                enc.write_metric_value(fd, NoLabels, MetricValue::Int(fd_count as i64))?;
            }
            if let Ok(limits) = p.limits() {
                if let procfs::process::LimitValue::Value(max) = limits.max_open_files.soft_limit {
                    let fd = MetricName::from_str("max_fds");
                    enc.write_help(fd, "Maximum number of open file descriptors.")?;
                    enc.write_metric_value(fd, NoLabels, MetricValue::Int(max as i64))?;
                }
            }

            if let Ok(stat) = p.stat() {
                // memory
                let vmm = MetricName::from_str("virtual_memory_bytes");
                enc.write_help(vmm, "Virtual memory size in bytes.")?;
                enc.write_metric_value(vmm, NoLabels, MetricValue::Int(stat.vsize as i64))?;

                let rss = MetricName::from_str("resident_memory_bytes");
                enc.write_help(rss, "Resident memory size in bytes.")?;
                enc.write_metric_value(
                    rss,
                    NoLabels,
                    MetricValue::Int((stat.rss as i64) * pagesize()),
                )?;

                // cpu
                let cpu = MetricName::from_str("cpu_seconds_total");
                enc.write_help(cpu, "Total user and system CPU time spent in seconds.")?;
                enc.write_metric_value(
                    cpu,
                    NoLabels,
                    MetricValue::Int((stat.utime + stat.stime) as i64 / clk_tck()),
                )?;

                // threads
                let threads = MetricName::from_str("threads");
                enc.write_help(threads, "Number of OS threads in the process.")?;
                enc.write_metric_value(threads, NoLabels, MetricValue::Int(stat.num_threads))?;
            }

            if let Some(start_time) = self.start_time {
                let name = MetricName::from_str("start_time_seconds");
                enc.write_help(
                    name,
                    "Start time of the process since unix epoch in seconds.",
                )?;
                enc.write_metric_value(name, NoLabels, MetricValue::Int(start_time))?;
            }
        }

        Ok(())
    }
}

fn clk_tck() -> i64 {
    static CLK_TCK: OnceLock<i64> = OnceLock::new();
    *CLK_TCK.get_or_init(|| unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as i64)
}

fn pagesize() -> i64 {
    static PAGESIZE: OnceLock<i64> = OnceLock::new();
    *PAGESIZE.get_or_init(|| unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as i64)
}
