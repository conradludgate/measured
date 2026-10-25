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
    use measured::{FixedCardinalityLabel, FloatCounterVec, label::StaticLabelSet};

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
fn info_labels_are_independent_snapshots() {
    #[derive(Clone, Default, LabelGroup, Debug, PartialEq)]
    struct Version {
        version: String,
    }

    let info = InfoMetric::<Version>::default();
    let mut snapshot = info.get_labels();
    snapshot.version.push_str("local");
    assert_eq!(info.get_labels(), Version::default());
    info.set_labels(Version {
        version: "1.0".into(),
    });
    assert_eq!(snapshot.version, "local");
    assert_eq!(info.get_labels().version, "1.0");
}
