use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use measured::{
    CallbackCounter, CallbackFloatGauge, CallbackGauge, Counter, CounterVec, FixedCardinalityLabel,
    FloatGauge, Gauge, GaugeVec, Histogram, HistogramVec, InfoMetric, LabelGroup, MetricGroup,
    metric::histogram::Thresholds,
};
use measured_prometheus_text::BufferedTextEncoder;

#[derive(FixedCardinalityLabel, Clone, Copy)]
enum Kind {
    A,
    B,
}

#[derive(LabelGroup, Clone, Copy)]
#[label(set = KindSet)]
struct KindLabels {
    kind: Kind,
}

#[derive(LabelGroup)]
#[label(set = WorkerSet)]
struct WorkerLabels {
    #[label(fixed_with = std::ops::Range<usize>)]
    worker: usize,
}

fn encode(group: &impl MetricGroup<BufferedTextEncoder>) -> String {
    let mut enc = BufferedTextEncoder::new();
    group.collect_group_into(&mut enc).unwrap();
    String::from_utf8(enc.finish().to_vec()).unwrap()
}

#[test]
fn read_values() {
    let counter = Counter::new();
    counter.inc_by(3);
    assert_eq!(counter.get(), 3);

    let mut gauge = Gauge::new();
    gauge.set(-2);
    gauge.inc_mut();
    assert_eq!(gauge.get(), -1);

    let float = FloatGauge::new();
    float.set(1.5);
    assert_eq!(float.get(), 1.5);

    for vec in [CounterVec::<KindSet>::dense(), CounterVec::sparse()] {
        vec.inc_by(KindLabels { kind: Kind::A }, 2);
        assert_eq!(vec.get(KindLabels { kind: Kind::A }), 2);

        // reading a metric does not create it
        assert_eq!(vec.get(KindLabels { kind: Kind::B }), 0);
        assert_eq!(vec.get_cardinality().0, 1);
    }

    let mut gauges = GaugeVec::<KindSet>::new();
    gauges.set_mut(KindLabels { kind: Kind::B }, 7);
    gauges.dec_by_mut(KindLabels { kind: Kind::B }, 2);
    assert_eq!(gauges.get(KindLabels { kind: Kind::B }), 5);
}

#[test]
fn histogram_sample() {
    let histogram = Histogram::with_metadata(Thresholds::with_buckets([1.0, 2.0]));
    histogram.observe(0.5);
    histogram.observe(1.5);
    histogram.observe(1.5);
    histogram.observe(10.0);
    histogram.observe_duration(std::time::Duration::from_millis(500));

    let sample = histogram.sample();
    assert_eq!(sample.buckets, [2, 2]);
    assert_eq!(sample.inf, 1);
    assert_eq!(sample.count(), 5);
    assert_eq!(sample.sum, 14.0);

    let vec = HistogramVec::<KindSet, 2>::with_metadata(Thresholds::with_buckets([1.0, 2.0]));
    vec.observe(KindLabels { kind: Kind::A }, 1.0);
    assert_eq!(vec.sample(KindLabels { kind: Kind::A }).count(), 1);
    assert_eq!(vec.sample(KindLabels { kind: Kind::B }).count(), 0);
    assert_eq!(vec.get_cardinality().0, 1);
}

#[test]
#[should_panic = "must not contain +Inf"]
fn exponential_buckets_overflow() {
    Thresholds::<4>::exponential_buckets(f64::MAX / 2.0, 4.0);
}

#[test]
fn range_label_set() {
    let workers = CounterVec::with_label_set(WorkerSet::new(2..4));
    workers.inc(WorkerLabels { worker: 3 });
    assert!(
        workers
            .try_with_labels(WorkerLabels { worker: 4 })
            .is_none()
    );

    #[derive(MetricGroup)]
    #[metric(new(workers: CounterVec<WorkerSet>))]
    struct Metrics {
        #[metric(init = workers)]
        workers: CounterVec<WorkerSet>,
    }

    assert_eq!(
        encode(&Metrics::new(workers)),
        "# TYPE workers counter\nworkers{worker=\"3\"} 1\n"
    );
}

#[test]
fn callback_and_info() {
    #[derive(LabelGroup)]
    struct BuildInfo {
        version: &'static str,
        debug: bool,
        id: u64,
    }

    #[derive(MetricGroup)]
    #[metric(new(value: Arc<AtomicU64>))]
    struct Metrics {
        /// the value
        #[metric(init = CallbackGauge::gauge({ let value = value.clone(); move || value.load(Ordering::Relaxed) as i64 }))]
        value: CallbackGauge,
        #[metric(init = CallbackCounter::counter(move || value.load(Ordering::Relaxed)))]
        value_total: CallbackCounter,
        #[metric(init = CallbackFloatGauge::float_gauge(|| 0.5))]
        half: CallbackFloatGauge,
        #[metric(init = InfoMetric::new(BuildInfo { version: "1.0", debug: true, id: u64::MAX }))]
        build_info: InfoMetric<BuildInfo>,
    }

    let value = Arc::new(AtomicU64::new(1));
    let metrics = Metrics::new(value.clone());
    value.store(5, Ordering::Relaxed);
    metrics.build_info.set_labels(BuildInfo {
        version: "1.1",
        debug: false,
        id: u64::MAX,
    });

    assert_eq!(
        encode(&metrics),
        r#"# HELP value the value
# TYPE value gauge
value 5

# TYPE value_total counter
value_total 5

# TYPE half gauge
half 0.5

# TYPE build_info gauge
build_info{version="1.1",debug="false",id="18446744073709551615"} 1
"#
    );
}

#[test]
fn new_metric_types() {
    let cpu = measured::FloatCounter::new();
    cpu.inc_by(0.25);
    cpu.inc();
    assert_eq!(cpu.get(), 1.25);

    let cpus = measured::FloatCounterVec::<KindSet>::new();
    cpus.inc_by(KindLabels { kind: Kind::A }, 0.5);
    assert_eq!(cpus.get(KindLabels { kind: Kind::A }), 0.5);
    assert_eq!(cpus.get(KindLabels { kind: Kind::B }), 0.0);

    let ages = measured::GaugeHistogram::with_metadata(Thresholds::with_buckets([1.0, 2.0]));
    ages.observe(0.5);
    ages.observe(1.5);
    ages.observe(10.0);
    ages.remove(1.5);
    let sample = ages.sample();
    assert_eq!(sample.buckets, [1, 0]);
    assert_eq!(sample.inf, 1);
    assert_eq!(sample.count(), 2);
    assert_eq!(sample.sum, 10.5);

    let status = measured::StateSet::new(Kind::A);
    assert!(matches!(status.get(), Kind::A));
    status.set(Kind::B);
    assert!(matches!(status.get(), Kind::B));
}

#[test]
fn state_set_in_namespace() {
    #[derive(MetricGroup)]
    #[metric(new())]
    struct Inner {
        #[metric(init = measured::StateSet::new(Kind::B))]
        kind: measured::StateSet<Kind>,
    }

    #[derive(MetricGroup)]
    #[metric(new())]
    struct Outer {
        #[metric(namespace = "app:job")]
        #[metric(init = Inner::new())]
        inner: Inner,
    }

    // `:` is not valid in label names, so it is replaced in the state label
    assert_eq!(
        encode(&Outer::new()),
        "# TYPE app:job_kind gauge\napp:job_kind{app_job_kind=\"a\"} 0\napp:job_kind{app_job_kind=\"b\"} 1\n"
    );
}
