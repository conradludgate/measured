use std::hash::{BuildHasher, BuildHasherDefault};

use divan::black_box;
use measured_derive::FixedCardinalityLabel;
use prometheus_client::encoding::EncodeLabelValue;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rustc_hash::FxHasher;

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::Divan::from_args().threads([0]).run_benches();
}

fn thread_rng() -> SmallRng {
    SmallRng::seed_from_u64(
        BuildHasherDefault::<FxHasher>::default().hash_one(std::thread::current().id()),
    )
}

fn iter() -> impl Iterator<Item = (ErrorKind, &'static str)> {
    let mut rng = thread_rng();
    std::iter::from_fn(move || {
        let route = rng.gen_range(0..routes().len());
        let error = rng.gen_range(0..errors().len());
        Some((errors()[error], routes()[route]))
    })
    .take(20000)
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

#[divan::bench_group(sample_size = 5, sample_count = 500)]
mod fixed_cardinality {
    use std::hash::BuildHasherDefault;

    use divan::Bencher;
    use lasso::{Rodeo, RodeoReader, Spur};
    use measured::{
        label::StaticLabelSet,
        metric::{group::Encoding, MetricFamilyEncoding},
    };
    use measured_derive::LabelGroup;
    use prometheus_client::encoding::EncodeLabelSet;
    use rustc_hash::FxHasher;

    use crate::{iter, routes, ErrorKind};

    #[divan::bench]
    fn measured(bencher: Bencher) {
        use measured::metric::name::{MetricName, Total};

        let error_set = ErrorsSet {
            kind: StaticLabelSet::new(),
            route: Rodeo::from_iter(routes()).into_reader(),
        };
        let counter_vec = measured::CounterVec::new(error_set);

        bencher
            .with_inputs(measured::text::TextEncoder::new)
            .bench_refs(|encoder| {
                for (kind, route) in iter() {
                    counter_vec.inc(Error { kind, route });
                }

                let metric = MetricName::from_static("http_request_errors").with_suffix(Total);
                encoder.write_help(&metric, "help text");
                counter_vec.collect_into(&metric, encoder);
                encoder.finish();
            });
    }

    #[divan::bench]
    fn measured_sparse(bencher: Bencher) {
        use measured::metric::name::{MetricName, Total};

        let error_set = ErrorsSet {
            kind: StaticLabelSet::new(),
            route: Rodeo::from_iter(routes()).into_reader(),
        };
        let counter_vec = measured::CounterVec::new_sparse(error_set);

        bencher
            .with_inputs(measured::text::TextEncoder::new)
            .bench_refs(|encoder| {
                for (kind, route) in iter() {
                    counter_vec.inc(Error { kind, route });
                }

                let metric = MetricName::from_static("http_request_errors").with_suffix(Total);
                encoder.write_help(&metric, "help text");
                counter_vec.collect_into(&metric, encoder);
                encoder.finish();
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

        bencher.with_inputs(String::new).bench_refs(|string| {
            for (kind, route) in iter() {
                counter_vec.with_label_values(&[kind.to_str(), route]).inc();
            }

            string.clear();
            prometheus::TextEncoder
                .encode_utf8(&registry.gather(), string)
                .unwrap();
        });
    }

    #[divan::bench]
    fn metrics(bencher: Bencher) {
        let recorder = metrics_exporter_prometheus::PrometheusBuilder::new().build_recorder();

        metrics::with_local_recorder(&recorder, || {
            metrics::describe_counter!("http_request_errors", "help text");
        });

        bencher.bench(|| {
            metrics::with_local_recorder(&recorder, || {
                for (kind, route) in iter() {
                    let labels = [("kind", kind.to_str()), ("route", route)];
                    metrics::counter!("http_request_errors", &labels).increment(1);
                }
            });

            recorder.handle().render()
        });
    }

    #[divan::bench]
    fn prometheus_client(bencher: Bencher) {
        use prometheus_client::encoding::text::encode;
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

        bencher.with_inputs(String::new).bench_refs(|string| {
            for (kind, route) in iter() {
                counter_vec
                    .get_or_create(&ErrorStatic { kind, route })
                    .inc();
            }

            string.clear();
            encode(string, &registry).unwrap();
        });
    }

    #[derive(Clone, Copy, PartialEq, Debug, LabelGroup)]
    #[label(set = ErrorsSet)]
    struct Error<'a> {
        kind: ErrorKind,
        #[label(fixed_with = RodeoReader<Spur, BuildHasherDefault<FxHasher>>)]
        route: &'a str,
    }

    #[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
    struct ErrorStatic {
        kind: ErrorKind,
        route: &'static str,
    }
}

#[divan::bench_group(sample_size = 2, sample_count = 100)]
mod high_cardinality {
    use std::hash::BuildHasherDefault;

    use divan::Bencher;
    use fake::{faker::name::raw::Name, locales::EN, Fake};
    use lasso::{Rodeo, RodeoReader, Spur, ThreadedRodeo};
    use measured::{
        label::StaticLabelSet,
        metric::{group::Encoding, MetricFamilyEncoding},
    };
    use measured_derive::LabelGroup;
    use metrics::SharedString;
    use prometheus_client::encoding::EncodeLabelSet;
    use rustc_hash::FxHasher;

    use crate::{iter, routes, thread_rng, ErrorKind};

    fn get_names() -> Vec<String> {
        let mut rng = thread_rng();
        std::iter::repeat_with(|| Name(EN).fake_with_rng::<String, _>(&mut rng))
            .take(2000)
            .collect()
    }

    #[divan::bench]
    fn measured(bencher: Bencher) {
        use measured::metric::name::{MetricName, Total};

        let error_set = ErrorsSet {
            kind: StaticLabelSet::new(),
            route: Rodeo::from_iter(routes()).into_reader(),
            user_name: ThreadedRodeo::with_hasher(Default::default()),
        };
        let counter_vec = measured::CounterVec::new(error_set);

        bencher
            .with_inputs(|| (measured::text::TextEncoder::new(), get_names()))
            .bench_refs(|(encoder, names)| {
                for ((kind, route), name) in iter().zip(names) {
                    counter_vec.inc(Error {
                        kind,
                        route,
                        user_name: name,
                    });
                }

                let metric = MetricName::from_static("http_request_errors").with_suffix(Total);
                encoder.write_help(&metric, "help text");
                counter_vec.collect_into(&metric, encoder);
                encoder.finish();
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

        bencher
            .with_inputs(|| (String::new(), get_names()))
            .bench_refs(|(string, names)| {
                for ((kind, route), name) in iter().zip(names) {
                    counter_vec
                        .with_label_values(&[kind.to_str(), route, &name])
                        .inc();
                }

                string.clear();
                prometheus::TextEncoder
                    .encode_utf8(&registry.gather(), string)
                    .unwrap();
            });
    }

    #[divan::bench]
    fn metrics(bencher: Bencher) {
        let recorder = metrics_exporter_prometheus::PrometheusBuilder::new().build_recorder();

        metrics::with_local_recorder(&recorder, || {
            metrics::describe_counter!("http_request_errors", "help text");
        });

        bencher.with_inputs(get_names).bench_refs(|names| {
            metrics::with_local_recorder(&recorder, || {
                for ((kind, route), name) in iter().zip(names) {
                    let labels = [
                        ("kind", SharedString::const_str(kind.to_str())),
                        ("route", SharedString::const_str(route)),
                        ("user_name", SharedString::from_owned(name.to_owned())),
                    ];
                    metrics::counter!("http_request_errors", &labels).increment(1);
                }
            });

            recorder.handle().render()
        });
    }

    #[divan::bench]
    fn prometheus_client(bencher: Bencher) {
        use prometheus_client::encoding::text::encode;
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

        bencher
            .with_inputs(|| (String::new(), get_names()))
            .bench_refs(|(string, names)| {
                for ((kind, route), name) in iter().zip(names) {
                    counter_vec
                        .get_or_create(&ErrorStatic {
                            kind,
                            route,
                            user_name: name.to_owned(),
                        })
                        .inc();
                }

                string.clear();
                encode(string, &registry).unwrap();
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
}
