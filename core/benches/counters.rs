use std::cell::RefCell;
use std::hash::{BuildHasher, BuildHasherDefault};

use divan::black_box;
use divan::Bencher;
use lasso::{Rodeo, RodeoReader, Spur};
use measured::label::StaticLabelSet;
use measured_derive::FixedCardinalityLabel;
use measured_derive::LabelGroup;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::encoding::EncodeLabelValue;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rustc_hash::FxHasher;

fn main() {
    divan::Divan::from_args()
        .threads([0])
        .sample_size(100000)
        .sample_count(500)
        .run_benches();
}

#[divan::bench]
fn measured(bencher: Bencher) {
    let error_set = ErrorsSet {
        kind: StaticLabelSet::new(),
        route: Rodeo::from_iter(routes()).into_reader(),
    };
    let counter_vec = measured::CounterVec::with_label_set(error_set);

    thread_local! {
        static RNG: RefCell<SmallRng> = RefCell::new(thread_rng());
    }

    bencher
        .with_inputs(|| RNG.with(|rng| get(&mut *rng.borrow_mut())))
        .bench_values(|(kind, route)| {
            counter_vec.inc(Error { kind, route });
        });
}

#[divan::bench]
fn measured_sparse(bencher: Bencher) {
    let error_set = ErrorsSet {
        kind: StaticLabelSet::new(),
        route: Rodeo::from_iter(routes()).into_reader(),
    };
    let counter_vec = measured::CounterVec::sparse_with_label_set(error_set);

    thread_local! {
        static RNG: RefCell<SmallRng> = RefCell::new(thread_rng());
    }

    bencher
        .with_inputs(|| RNG.with(|rng| get(&mut *rng.borrow_mut())))
        .bench_values(|(kind, route)| {
            counter_vec.inc(Error { kind, route });
        });
}

#[divan::bench]
fn measured_papaya(bencher: Bencher) {
    let error_set = ErrorsSet {
        kind: StaticLabelSet::new(),
        route: Rodeo::from_iter(routes()).into_reader(),
    };
    let counter_vec = measured::CounterVec::sparse2_with_label_set_and_metadata(error_set, ());

    thread_local! {
        static RNG: RefCell<SmallRng> = RefCell::new(thread_rng());
    }

    bencher
        .with_inputs(|| RNG.with(|rng| get(&mut *rng.borrow_mut())))
        .bench_values(|(kind, route)| {
            counter_vec.inc(Error { kind, route });
        });
}

#[divan::bench]
fn prometheus(bencher: Bencher) {
    let registry = prometheus::Registry::new();
    let counter_vec = prometheus::register_int_counter_vec_with_registry!(
        "http_request_errors",
        "help text",
        &["kind", "route"],
        registry
    )
    .unwrap();

    thread_local! {
        static RNG: RefCell<SmallRng> = RefCell::new(thread_rng());
    }

    bencher
        .with_inputs(|| RNG.with(|rng| get(&mut *rng.borrow_mut())))
        .bench_values(|(kind, route)| {
            counter_vec.with_label_values(&[kind.to_str(), route]).inc();
        });
}

#[divan::bench]
fn metrics(bencher: Bencher) {
    let recorder = metrics_exporter_prometheus::PrometheusBuilder::new().build_recorder();

    metrics::with_local_recorder(&recorder, || {
        metrics::describe_counter!("http_request_errors", "help text");
    });

    thread_local! {
        static RNG: RefCell<SmallRng> = RefCell::new(thread_rng());
    }

    bencher
        .with_inputs(|| RNG.with(|rng| get(&mut *rng.borrow_mut())))
        .bench_values(|(kind, route)| {
            metrics::with_local_recorder(&recorder, || {
                let labels = [("kind", kind.to_str()), ("route", route)];
                metrics::counter!("http_request_errors", &labels).increment(1);
            });
        });
}

#[divan::bench]
fn prometheus_client(bencher: Bencher) {
    use prometheus_client::metrics::counter::Counter;
    use prometheus_client::metrics::family::Family;
    use prometheus_client::registry::Registry;

    let mut registry = <Registry>::default();

    let counter_vec = Family::<ErrorStatic, Counter>::default();

    // Register the metric family with the registry.
    registry.register(
        // With the metric name.
        "http_request_errors",
        // And the metric help text.
        "Number of HTTP requests received",
        counter_vec.clone(),
    );

    thread_local! {
        static RNG: RefCell<SmallRng> = RefCell::new(thread_rng());
    }

    bencher
        .with_inputs(|| RNG.with(|rng| get(&mut *rng.borrow_mut())))
        .bench_values(|(kind, route)| {
            counter_vec
                .get_or_create(&ErrorStatic { kind, route })
                .inc();
        });
}

fn thread_rng() -> SmallRng {
    SmallRng::seed_from_u64(
        BuildHasherDefault::<FxHasher>::default().hash_one(std::thread::current().id()),
    )
}

fn get(rng: &mut impl Rng) -> (ErrorKind, &'static str) {
    let route = rng.gen_range(0..routes().len());
    let error = rng.gen_range(0..errors().len());
    (errors()[error], routes()[route])
}

fn routes() -> &'static [&'static str] {
    black_box(&[
        "/api/v1/users",
        "/api/v1/users/:id",
        "/api/v1/products",
        "/api/v1/products/:id",
        "/api/v1/products/:id/owner",
        "/api/v1/products/:id/purchase",
    ])
}

fn errors() -> &'static [ErrorKind] {
    black_box(&[ErrorKind::User, ErrorKind::Internal, ErrorKind::Network])
}

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq, FixedCardinalityLabel, EncodeLabelValue)]
enum ErrorKind {
    User,
    Internal,
    Network,
}

impl ErrorKind {
    fn to_str(self) -> &'static str {
        match self {
            ErrorKind::User => "user",
            ErrorKind::Internal => "internal",
            ErrorKind::Network => "network",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, LabelGroup)]
#[label(set = ErrorsSet)]
struct Error<'a> {
    kind: ErrorKind,
    #[label(fixed_with = RodeoReader<Spur, ahash::RandomState>)]
    route: &'a str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ErrorStatic {
    kind: ErrorKind,
    route: &'static str,
}
