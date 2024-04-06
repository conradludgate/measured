// #[global_allocator]
// static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::Divan::from_args()
        .threads([1])
        .sample_size(100)
        .sample_count(50000)
        .run_benches();
}

use std::{
    cell::RefCell,
    hash::{BuildHasher, BuildHasherDefault},
};

use divan::{black_box, Bencher};
use fake::{faker::name::raw::Name, locales::EN, Fake};
use lasso::{Rodeo, RodeoReader, Spur, ThreadedRodeo};
use measured::label::StaticLabelSet;
use measured_derive::{FixedCardinalityLabel, LabelGroup};
use metrics::SharedString;
use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rustc_hash::FxHasher;

fn thread_rng() -> SmallRng {
    SmallRng::seed_from_u64(
        BuildHasherDefault::<FxHasher>::default().hash_one(std::thread::current().id()),
    )
}

fn get(rng: &mut impl Rng) -> (ErrorKind, &'static str, String) {
    let route = rng.gen_range(0..routes().len());
    let error = rng.gen_range(0..errors().len());
    let name = Name(EN).fake_with_rng::<String, _>(rng);
    (errors()[error], routes()[route], name)
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

#[divan::bench]
fn measured(bencher: Bencher) {
    let error_set = ErrorsSet {
        kind: StaticLabelSet::new(),
        route: Rodeo::from_iter(routes()).into_reader(),
        user_name: ThreadedRodeo::with_hasher(Default::default()),
    };
    let counter_vec = measured::CounterVec::with_label_set(error_set);

    thread_local! {
        static RNG: RefCell<SmallRng> = RefCell::new(thread_rng());
    }

    bencher
        .with_inputs(|| RNG.with(|rng| get(&mut *rng.borrow_mut())))
        .bench_values(|(kind, route, name)| {
            counter_vec.inc(Error {
                kind,
                route,
                user_name: &name,
            });
            name
        });
}

#[divan::bench]
fn prometheus(bencher: Bencher) {
    let registry = prometheus::Registry::new();
    let counter_vec = prometheus::register_int_counter_vec_with_registry!(
        "http_request_errors_total",
        "help text",
        &["kind", "route", "user_name"],
        registry
    )
    .unwrap();

    thread_local! {
        static RNG: RefCell<SmallRng> = RefCell::new(thread_rng());
    }

    bencher
        .with_inputs(|| RNG.with(|rng| get(&mut *rng.borrow_mut())))
        .bench_values(|(kind, route, name)| {
            counter_vec
                .with_label_values(&[kind.to_str(), route, &name])
                .inc();
            name
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
        .bench_values(|(kind, route, name)| {
            metrics::with_local_recorder(&recorder, || {
                let labels = [
                    ("kind", SharedString::const_str(kind.to_str())),
                    ("route", SharedString::const_str(route)),
                    ("user_name", SharedString::from_owned(name)),
                ];
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
        .bench_values(|(kind, route, name)| {
            counter_vec
                .get_or_create(&ErrorStatic {
                    kind,
                    route,
                    user_name: name,
                })
                .inc();
        });
}

#[derive(Clone, Copy, PartialEq, Debug, LabelGroup)]
#[label(set = ErrorsSet)]
struct Error<'a> {
    kind: ErrorKind,
    #[label(fixed_with = RodeoReader<Spur, BuildHasherDefault<FxHasher>>)]
    route: &'a str,
    #[label(dynamic_with = ThreadedRodeo<Spur, BuildHasherDefault<FxHasher>>)]
    user_name: &'a str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ErrorStatic {
    kind: ErrorKind,
    route: &'static str,
    user_name: String,
}
