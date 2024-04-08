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

use std::sync::{
    atomic::{AtomicI64, AtomicU64},
    OnceLock,
};

use measured::{
    label::NoLabels,
    metric::{
        counter::CounterState,
        gauge::GaugeState,
        group::Encoding,
        name::{MetricName, MetricNameEncoder},
        MetricEncoding,
    },
    MetricGroup,
};
use nix::unistd::Pid;

// Copyright 2019 TiKV Project Authors. Licensed under Apache-2.0.
// https://github.com/tikv/rust-prometheus/blob/f49c724df0e123520554664436da68e555593af0/src/process_collector.rs
// With modifications by Conrad Ludgate for the transition to measured.

/// A collector which exports the current state of process metrics including
/// CPU, memory and file descriptor usage, thread count, as well as the process
/// start time for the given process id.
pub struct ProcessCollector {
    pid: Pid,
    start_time: Option<i64>,
}

impl ProcessCollector {
    /// Create a `ProcessCollector` with the given process id and namespace.
    pub fn new(pid: Pid) -> ProcessCollector {
        // proc_start_time init once because it is immutable
        let mut start_time = None;
        if let Ok(boot_time) = procfs::boot_time_secs() {
            if let Ok(stat) = procfs::process::Process::myself().and_then(|p| p.stat()) {
                start_time = Some(stat.starttime as i64 / clk_tck() + boot_time as i64);
            }
        }

        ProcessCollector { pid, start_time }
    }

    /// Return a `ProcessCollector` of the calling process.
    pub fn for_self() -> ProcessCollector {
        ProcessCollector::new(Pid::this())
    }
}

fn write_count<Enc: Encoding>(
    x: u64,
    name: impl MetricNameEncoder,
    enc: &mut Enc,
) -> Result<(), Enc::Err>
where
    CounterState: MetricEncoding<Enc>,
{
    CounterState {
        count: AtomicU64::new(x),
    }
    .collect_into(&(), NoLabels, name, enc)
}

fn write_gauge<Enc: Encoding>(
    x: i64,
    name: impl MetricNameEncoder,
    enc: &mut Enc,
) -> Result<(), Enc::Err>
where
    GaugeState: MetricEncoding<Enc>,
{
    GaugeState {
        count: AtomicI64::new(x),
    }
    .collect_into(&(), NoLabels, name, enc)
}

impl<Enc: Encoding> MetricGroup<Enc> for ProcessCollector
where
    CounterState: MetricEncoding<Enc>,
    GaugeState: MetricEncoding<Enc>,
{
    fn collect_group_into(&self, enc: &mut Enc) -> Result<(), Enc::Err> {
        let Ok(p) = procfs::process::Process::new(self.pid.as_raw()) else {
            // we can't construct a Process object, so there's no stats to gather
            return Ok(());
        };

        // file descriptors
        if let Ok(fd_count) = p.fd_count() {
            let fd = MetricName::from_str("open_fds");
            enc.write_help(fd, "Number of open file descriptors.")?;
            write_gauge(fd_count as i64, fd, &mut *enc)?;
        }
        if let Ok(limits) = p.limits() {
            if let procfs::process::LimitValue::Value(max) = limits.max_open_files.soft_limit {
                let fd = MetricName::from_str("max_fds");
                enc.write_help(fd, "Maximum number of open file descriptors.")?;
                write_gauge(max as i64, fd, &mut *enc)?;
            }
        }

        if let Ok(stat) = p.stat() {
            // memory
            let vmm = MetricName::from_str("virtual_memory_bytes");
            enc.write_help(vmm, "Virtual memory size in bytes.")?;
            write_gauge(stat.vsize as i64, vmm, &mut *enc)?;

            let rss = MetricName::from_str("resident_memory_bytes");
            enc.write_help(vmm, "Resident memory size in bytes.")?;
            write_gauge((stat.rss as i64) * pagesize(), rss, &mut *enc)?;

            // cpu
            let cpu = MetricName::from_str("cpu_seconds_total");
            enc.write_help(cpu, "Total user and system CPU time spent in seconds.")?;
            write_count((stat.utime + stat.stime) / clk_tck() as u64, cpu, &mut *enc)?;

            // threads
            let threads = MetricName::from_str("threads");
            enc.write_help(vmm, "Number of OS threads in the process.")?;
            write_gauge(stat.num_threads, threads, &mut *enc)?;
        }

        if let Some(start_time) = self.start_time {
            let name = MetricName::from_str("start_time_seconds");
            enc.write_help(
                name,
                "Start time of the process since unix epoch in seconds.",
            )?;
            write_gauge(start_time, name, &mut *enc)?;
        }

        Ok(())
    }
}

fn clk_tck() -> i64 {
    static CLK_TCK: OnceLock<i64> = OnceLock::new();
    *CLK_TCK.get_or_init(|| {
        nix::unistd::sysconf(nix::unistd::SysconfVar::CLK_TCK)
            .unwrap()
            .unwrap()
    })
}

fn pagesize() -> i64 {
    static PAGESIZE: OnceLock<i64> = OnceLock::new();
    *PAGESIZE.get_or_init(|| {
        nix::unistd::sysconf(nix::unistd::SysconfVar::PAGE_SIZE)
            .unwrap()
            .unwrap()
    })
}
