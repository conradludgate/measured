use measured::{FloatCounter, InfoMetric, LabelGroup};

#[test]
fn invalid_float_counter_increments_preserve_the_value() {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    for invalid in [-1.0, f64::NEG_INFINITY, f64::NAN] {
        let mut counter = FloatCounter::new();
        counter.inc_by(2.5);
        let atomic = catch_unwind(AssertUnwindSafe(|| counter.inc_by(invalid)));
        assert_eq!(atomic.is_err(), cfg!(debug_assertions));
        assert_eq!(counter.get(), 2.5);

        let exclusive = catch_unwind(AssertUnwindSafe(|| counter.inc_by_mut(invalid)));
        assert_eq!(exclusive.is_err(), cfg!(debug_assertions));
        assert_eq!(counter.get(), 2.5);
        counter.inc_mut();
        counter.inc_by(0.0);
        assert_eq!(counter.get(), 3.5);
        counter.inc_by_mut(0.25);
        assert_eq!(counter.get(), 3.75);
    }
}

#[test]
fn mutable_float_counter_vec_updates_the_selected_series() {
    use measured::label::StaticLabelSet;
    use measured::{FixedCardinalityLabel, FloatCounterVec};

    #[derive(FixedCardinalityLabel, Clone, Copy)]
    #[label(singleton = "kind")]
    enum Kind {
        A,
        B,
    }

    for mut counters in [
        FloatCounterVec::<StaticLabelSet<Kind>>::dense(),
        FloatCounterVec::sparse(),
    ] {
        counters.inc_by_mut(Kind::B, 2.5);
        counters.inc_mut(Kind::B);
        assert_eq!(counters.get(Kind::B), 3.5);
        assert_eq!(counters.get(Kind::A), 0.0);
        assert_eq!(counters.get_cardinality().0, 1);
    }
}

#[test]
fn default_and_static_reference_label_lookup_preserve_existing_metrics() {
    use measured::CounterVec;
    use measured::label::{LabelGroupSet, NoLabels};

    // A downstream label set can rely on the default `find` implementation.
    struct Labels;
    impl LabelGroupSet for Labels {
        type Group<'a> = NoLabels;
        type Unique = usize;
        fn cardinality(&self) -> Option<usize> {
            None
        }
        fn encode_dense(&self, _: usize) -> Option<usize> {
            None
        }
        fn decode_dense(&self, _: usize) -> NoLabels {
            unreachable!()
        }
        fn encode(&self, _: NoLabels) -> Option<usize> {
            Some(7)
        }
        fn decode(&self, _: &usize) -> NoLabels {
            NoLabels
        }
    }

    let owned = CounterVec::with_label_set(Labels);
    owned.inc_by(NoLabels, 3);
    assert_eq!(owned.get(NoLabels), 3);

    static LABELS: Labels = Labels;
    let borrowed = CounterVec::with_label_set(&LABELS);
    borrowed.inc_by(NoLabels, 4);
    assert_eq!(borrowed.get(NoLabels), 4);
}

#[test]
fn info_labels_are_independent_snapshots() {
    #[derive(Clone, Default, LabelGroup, Debug, PartialEq)]
    struct Version {
        version: String,
    }

    let info = InfoMetric::<Version>::default();
    let mut snapshot = info.get_labels();
    snapshot.version.push_str("local");
    assert_eq!(info.get_labels(), Version::default());
    info.set_labels(Version { version: "1.0".into() });
    assert_eq!(snapshot.version, "local");
    assert_eq!(info.get_labels().version, "1.0");
}

#[cfg(feature = "lasso")]
#[test]
fn all_metric_reads_leave_dynamic_labels_and_series_unchanged() {
    use std::sync::Arc;

    use measured::metric::histogram::Thresholds;

    #[derive(LabelGroup, Clone, Copy)]
    #[label(set = Names)]
    struct Name<'a> {
        #[label(dynamic_with = Arc<measured::lasso::ThreadedRodeo>)]
        name: &'a str,
    }

    macro_rules! check {
        ($metric:ty, $metadata:expr, $read:ident, $write:ident, $value:expr, $expected:expr) => {{
            let labels = Arc::new(measured::lasso::ThreadedRodeo::default());
            let metric = <$metric>::with_label_set_and_metadata(Names::new(labels.clone()), $metadata);
            let missing = metric.$read(Name { name: "missing" });
            assert_eq!(missing, Default::default());
            assert_eq!(labels.len(), 0);
            assert_eq!(metric.get_cardinality().0, 0);
            assert!(
                metric.find_labels(Name { name: "missing" }).is_none(),
                "{}: missing labels",
                stringify!($metric)
            );

            // A label can exist in a shared interner without having a metric.
            labels.get_or_intern("known");
            assert_eq!(metric.$read(Name { name: "known" }), Default::default());
            assert_eq!(metric.get_cardinality().0, 0);
            metric.$write(Name { name: "known" }, $value);
            assert_eq!(
                metric.$read(Name { name: "known" }),
                $expected,
                "{}: present series",
                stringify!($metric)
            );
            assert_eq!(labels.len(), 1);
            assert_eq!(metric.get_cardinality().0, 1);

            let id = metric.find_labels(Name { name: "known" }).unwrap();
            assert!(metric.remove_metric(id).is_some());
            assert_eq!(metric.$read(Name { name: "known" }), Default::default());
            assert_eq!(metric.get_cardinality().0, 0);
        }};
    }

    check!(measured::CounterVec<Names>, (), get, inc_by, 3, 3);
    check!(measured::FloatCounterVec<Names>, (), get, inc_by, 1.5, 1.5);
    check!(measured::GaugeVec<Names>, (), get, set, -3, -3);
    check!(measured::FloatGaugeVec<Names>, (), get, set, -1.5, -1.5);
    check!(measured::CreatedCounterVec<Names>, (), get, inc_by, 3, 3);
    check!(
        measured::CreatedFloatCounterVec<Names>,
        (),
        get,
        inc_by,
        1.5,
        1.5
    );
    check!(measured::ExemplarCounterVec<Names>, (), get, inc_by, 3, 3);

    let thresholds = Thresholds::with_buckets([1.0]);
    let reference = measured::Histogram::with_metadata(thresholds.clone());
    reference.observe(0.5);
    let expected = reference.sample();
    check!(measured::HistogramVec<Names, 1>, thresholds.clone(), sample, observe, 0.5, expected);
    check!(measured::CreatedHistogramVec<Names, 1>, thresholds.clone(), sample, observe, 0.5, expected);
    check!(measured::ExemplarHistogramVec<Names, 1>, thresholds.clone(), sample, observe, 0.5, expected);
    let reference = measured::GaugeHistogram::with_metadata(thresholds.clone());
    reference.observe(0.5);
    let expected = reference.sample();
    check!(measured::GaugeHistogramVec<Names, 1>, thresholds, sample, observe, 0.5, expected);
}

#[test]
fn projection_preserves_histogram_metadata_and_labels() {
    use measured::FixedCardinalityLabel;
    use measured::label::StaticLabelSet;
    use measured::metric::histogram::{HistogramState, Thresholds};
    use measured::metric::name::MetricName;
    use measured::metric::{MetricFamilyEncoding, MetricType, MetricVec};
    use measured_prometheus_text::BufferedTextEncoder;

    #[derive(Default)]
    struct State {
        histogram: HistogramState<1>,
    }
    impl MetricType for State {
        type Metadata = Thresholds<1>;
    }
    #[derive(FixedCardinalityLabel, Clone, Copy)]
    #[label(singleton = "kind")]
    enum Kind {
        A,
        B,
    }

    for vec in [
        MetricVec::<State, StaticLabelSet<Kind>>::dense_with_label_set_and_metadata(
            StaticLabelSet::new(),
            Thresholds::with_buckets([2.0]),
        ),
        MetricVec::sparse_with_label_set_and_metadata(StaticLabelSet::new(), Thresholds::with_buckets([2.0])),
    ] {
        vec.get_metric(vec.with_labels(Kind::B));
        let mut enc = BufferedTextEncoder::new();
        vec.project(|state| &state.histogram)
            .collect_family_into(
                MetricName::from_str("latency"),
                measured::metric::group::FamilyMeta::new().with_help("latency"),
                &mut enc,
            )
            .unwrap();
        assert_eq!(
            enc.finish(),
            "# HELP latency latency\n# TYPE latency histogram\nlatency_bucket{kind=\"b\",le=\"2.0\"} 0\nlatency_bucket{kind=\"b\",le=\"+Inf\"} 0\nlatency_sum{kind=\"b\"} 0.0\nlatency_count{kind=\"b\"} 0\n"
        );
    }
}

#[cfg(feature = "paracord")]
#[test]
fn composed_shared_label_sets_find_without_interning() {
    use std::sync::Arc;

    use measured::CounterVec;
    use measured::label::{ComposedGroup, LabelGroupSet};

    #[derive(LabelGroup, Clone, Copy)]
    #[label(set = NameSet)]
    struct Name<'a> {
        #[label(dynamic_with = Arc<paracord::ParaCord>)]
        name: &'a str,
    }
    #[derive(LabelGroup, Clone, Copy)]
    #[label(set = WorkerSet)]
    struct Worker {
        #[label(fixed_with = std::ops::Range<usize>)]
        worker: usize,
    }

    let interner = Arc::new(paracord::ParaCord::default());
    let labels = Arc::new(ComposedGroup(
        NameSet::new(interner.clone()),
        WorkerSet::new(2..4),
    ));
    let metric = CounterVec::with_label_set(labels.clone());
    let missing = ComposedGroup(Name { name: "missing" }, Worker { worker: 2 });
    assert_eq!(metric.get(missing), 0);
    assert!(labels.find(missing).is_none());
    assert!(interner.is_empty());

    let present = ComposedGroup(Name { name: "present" }, Worker { worker: 3 });
    metric.inc_by(present, 4);
    assert_eq!(metric.get(present), 4);
    assert_eq!(labels.find(present), labels.encode(present));
    let invalid_worker = ComposedGroup(Name { name: "present" }, Worker { worker: 4 });
    assert_eq!(metric.get(invalid_worker), 0);
    assert_eq!(interner.len(), 1);
}

#[test]
fn static_reference_forwards_find_without_calling_encode() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use measured::label::{LabelGroupSet, NoLabels};
    struct Labels {
        encodes: AtomicUsize,
    }
    impl LabelGroupSet for Labels {
        type Group<'a> = NoLabels;
        type Unique = usize;
        fn cardinality(&self) -> Option<usize> {
            None
        }
        fn encode_dense(&self, _: usize) -> Option<usize> {
            None
        }
        fn decode_dense(&self, _: usize) -> NoLabels {
            unreachable!()
        }
        fn encode(&self, _: NoLabels) -> Option<usize> {
            self.encodes.fetch_add(1, Ordering::Relaxed);
            Some(7)
        }
        fn find(&self, _: NoLabels) -> Option<usize> {
            Some(7)
        }
        fn decode(&self, _: &usize) -> NoLabels {
            NoLabels
        }
    }
    static LABELS: Labels = Labels { encodes: AtomicUsize::new(0) };
    let counters = measured::CounterVec::with_label_set(&LABELS);
    counters.inc_by(NoLabels, 4);
    assert_eq!(LABELS.encodes.load(Ordering::Relaxed), 1);
    assert_eq!(counters.get(NoLabels), 4);
    assert_eq!(LABELS.encodes.load(Ordering::Relaxed), 1);
}
