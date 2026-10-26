//! Every histogram constructor must validate explicit and generated bounds.
use measured::metric::histogram::Thresholds;

#[test]
fn all_constructors_validate_generated_bounds() {
    for invalid in [f64::NAN, f64::INFINITY] {
        assert!(std::panic::catch_unwind(|| Thresholds::with_buckets([1.0, invalid])).is_err());
        assert!(std::panic::catch_unwind(|| Thresholds::<2>::linear_buckets(invalid, 1.0)).is_err());
    }
    assert!(std::panic::catch_unwind(|| Thresholds::<3>::exponential_buckets(1e300, 1e10)).is_err());
    assert!(std::panic::catch_unwind(|| Thresholds::<3>::linear_buckets(1e308, 1e308)).is_err());
}
