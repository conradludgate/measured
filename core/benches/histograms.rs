#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::Divan::from_args().threads([0]).run_benches();
}

#[divan::bench_group(sample_size = 5, sample_count = 500)]
mod fixed_cardinality {
    use std::hash::BuildHasherDefault;

    use bytes::Bytes;
    use divan::{black_box, Bencher};
    use lasso::{Rodeo, RodeoReader, Spur};
    use measured::{label::StaticLabelSet, metric::histogram::Thresholds};
    use measured_derive::{FixedCardinalityLabel, LabelGroup};
    use prometheus::exponential_buckets;
    use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};
    use rustc_hash::FxHasher;

    const LOOPS: usize = 2000;
    const N: usize = 8;

    #[inline(never)]
    fn measured_inner(
        encoder: &mut measured::text::TextEncoder,
        h: &measured::HistogramVec<ErrorsSet, N>,
    ) -> Bytes {
        use measured::metric::name::MetricName;

        let mut n = latencies().iter().cycle();
        for _ in 0..black_box(LOOPS) {
            for &kind in errors() {
                for route in routes() {
                    h.observe(Error { kind, route }, *n.next().unwrap());
                }
            }
        }

        const NAME: &MetricName = MetricName::from_static("http_request_errors");
        encoder.write_help(&NAME, "help text");
        h.collect_into(NAME, encoder);
        encoder.finish()
    }

    #[divan::bench]
    fn measured(bencher: Bencher) {
        let error_set = ErrorsSet {
            kind: StaticLabelSet::new(),
            route: Rodeo::from_iter(routes()).into_reader(),
        };
        let h = measured::HistogramVec::new_metric_vec(
            error_set,
            Thresholds::<N>::exponential_buckets(1.0, 2.0),
        );

        bencher
            .with_inputs(measured::text::TextEncoder::new)
            .bench_refs(|encoder| measured_inner(encoder, &h));
    }

    #[divan::bench]
    fn measured_sparse(bencher: Bencher) {
        let error_set = ErrorsSet {
            kind: StaticLabelSet::new(),
            route: Rodeo::from_iter(routes()).into_reader(),
        };
        let h = measured::HistogramVec::new_sparse_metric_vec(
            error_set,
            Thresholds::<N>::exponential_buckets(1.0, 2.0),
        );
        bencher
            .with_inputs(measured::text::TextEncoder::new)
            .bench_refs(|encoder| measured_inner(encoder, &h));
    }

    #[divan::bench]
    fn prometheus(bencher: Bencher) {
        let registry = prometheus::Registry::new();
        let counter_vec = prometheus::register_histogram_vec_with_registry!(
            "http_request_errors",
            "help text",
            &["kind", "route"],
            exponential_buckets(1.0, 2.0, N).unwrap(),
            registry
        )
        .unwrap();

        bencher.with_inputs(String::new).bench_refs(|string| {
            let mut n = latencies().iter().cycle();
            for _ in 0..black_box(LOOPS) {
                for &kind in errors() {
                    for route in routes() {
                        counter_vec
                            .with_label_values(&[kind.to_str(), route])
                            .observe(*n.next().unwrap());
                    }
                }
            }

            string.clear();
            prometheus::TextEncoder
                .encode_utf8(&registry.gather(), string)
                .unwrap();
        });
    }

    #[divan::bench]
    fn metrics(bencher: Bencher) {
        let recorder = metrics_exporter_prometheus::PrometheusBuilder::new()
            .set_buckets_for_metric(
                metrics_exporter_prometheus::Matcher::Full("http_request_errors".to_string()),
                &exponential_buckets(1.0, 2.0, N).unwrap(),
            )
            .unwrap()
            .build_recorder();

        metrics::with_local_recorder(&recorder, || {
            metrics::describe_histogram!("http_request_errors", "help text");
        });

        bencher.bench(|| {
            metrics::with_local_recorder(&recorder, || {
                let mut n = latencies().iter().cycle();
                for _ in 0..black_box(LOOPS) {
                    for &kind in errors() {
                        for route in routes() {
                            let labels = [("kind", kind.to_str()), ("route", route)];
                            metrics::histogram!("http_request_errors", &labels)
                                .record(*n.next().unwrap());
                        }
                    }
                }
            });

            recorder.handle().render()
        });
    }

    #[divan::bench]
    fn prometheus_client(bencher: Bencher) {
        use prometheus_client::encoding::text::encode;
        use prometheus_client::metrics::family::Family;
        use prometheus_client::metrics::histogram::exponential_buckets;
        use prometheus_client::metrics::histogram::Histogram;
        use prometheus_client::registry::Registry;

        let mut registry = <Registry>::default();

        let h = Family::<ErrorStatic, Histogram>::new_with_constructor(|| {
            Histogram::new(exponential_buckets(1.0, 2.0, N as u16))
        });

        // Register the metric family with the registry.
        registry.register(
            // With the metric name.
            "http_request_errors",
            // And the metric help text.
            "Number of HTTP requests received",
            h.clone(),
        );

        bencher.with_inputs(String::new).bench_refs(|string| {
            let mut n = latencies().iter().cycle();
            for _ in 0..black_box(LOOPS) {
                for &kind in errors() {
                    for route in routes() {
                        h.get_or_create(&ErrorStatic { kind, route })
                            .observe(*n.next().unwrap());
                    }
                }
            }

            string.clear();
            encode(string, &registry).unwrap();
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

    fn latencies() -> &'static [f64] {
        black_box(&[
            0.2, 1.2, 0.5, 2.6, 7.0, 50.0, 4.0, 100.0, 5.0, 64.0, 54.4, 24.0, 16.7, 15.5, 1000.0,
            77.7, 1.0,
        ])
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
}
