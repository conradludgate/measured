use divan::{black_box, AllocProfiler, Bencher, Divan};
use lasso::{Rodeo, RodeoReader};
use measured_derive::{FixedCardinalityLabel, LabelGroup};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

const LOOPS: usize = 2000;

fn main() {
    Divan::from_args().threads([0]).run_benches();
}

#[divan::bench(sample_size = 5, sample_count = 500)]
fn measured(bencher: Bencher) {
    use measured::metric::name::{MetricName, Total};

    let error_set = ErrorsSet {
        route: Rodeo::from_iter(routes()).into_reader(),
    };
    let counter_vec = measured::CounterVec::new_counter_vec(error_set);

    bencher
        .with_inputs(measured::text::TextEncoder::new)
        .bench_refs(|encoder| {
            for _ in 0..black_box(LOOPS) {
                for &kind in errors() {
                    for route in routes() {
                        counter_vec.inc(Error { kind, route })
                    }
                }
            }

            let metric = "http_request_errors".with_suffix(Total);
            encoder.write_help(&metric, "help text");
            counter_vec.collect_into(&metric, encoder);
            encoder.finish();
        });
}

#[divan::bench(sample_size = 5, sample_count = 500)]
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
        for _ in 0..black_box(LOOPS) {
            for &kind in errors() {
                for route in routes() {
                    counter_vec.with_label_values(&[kind.to_str(), route]).inc()
                }
            }
        }

        string.clear();
        prometheus::TextEncoder
            .encode_utf8(&registry.gather(), string)
            .unwrap();
    });
}

#[divan::bench(sample_size = 5, sample_count = 500)]
fn metrics(bencher: Bencher) {
    let recorder = metrics_exporter_prometheus::PrometheusBuilder::new().build_recorder();

    metrics::with_local_recorder(&recorder, || {
        metrics::describe_counter!("http_request_errors", "help text")
    });

    bencher.bench(|| {
        metrics::with_local_recorder(&recorder, || {
            for _ in 0..black_box(LOOPS) {
                for &kind in errors() {
                    for route in routes() {
                        let labels = [("kind", kind.to_str()), ("route", route)];
                        metrics::counter!("http_request_errors", &labels).increment(1);
                    }
                }
            }
        });

        recorder.handle().render()
    });
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

#[derive(Clone, Copy, PartialEq, Debug, LabelGroup)]
#[label(set = ErrorsSet)]
struct Error<'a> {
    #[label(fixed)]
    kind: ErrorKind,
    #[label(fixed_with = RodeoReader)]
    route: &'a str,
}

#[derive(Clone, Copy, PartialEq, Debug, FixedCardinalityLabel)]
#[label(rename_all = "kebab-case")]
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
