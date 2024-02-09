#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::Divan::from_args().threads([0]).run_benches();
}

#[divan::bench_group(sample_size = 5, sample_count = 500)]
mod fixed_cardinality {
    use std::hash::{BuildHasher, BuildHasherDefault};

    use bytes::Bytes;
    use divan::{black_box, Bencher};
    use lasso::{Rodeo, RodeoReader, Spur};
    use measured::{
        label::StaticLabelSet,
        metric::{group::Encoding, histogram::Thresholds, MetricFamilyEncoding},
    };
    use measured_derive::{FixedCardinalityLabel, LabelGroup};
    use prometheus::exponential_buckets;
    use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};
    use rand::{rngs::SmallRng, Rng, SeedableRng};
    use rustc_hash::FxHasher;

    const N: usize = 8;

    #[inline(never)]
    fn measured_inner(
        encoder: &mut measured::text::TextEncoder,
        h: &measured::HistogramVec<ErrorsSet, N>,
    ) -> Bytes {
        use measured::metric::name::MetricName;

        for (kind, route, latency) in iter() {
            h.observe(Error { kind, route }, latency);
        }

        const NAME: &MetricName = MetricName::from_static("http_request_errors");
        encoder.write_help(NAME, "help text");
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
            Thresholds::<N>::exponential_buckets(0.001, 2.0),
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
            Thresholds::<N>::exponential_buckets(0.001, 2.0),
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
            exponential_buckets(0.001, 2.0, N).unwrap(),
            registry
        )
        .unwrap();

        bencher.with_inputs(String::new).bench_refs(|string| {
            for (kind, route, latency) in iter() {
                counter_vec
                    .with_label_values(&[kind.to_str(), route])
                    .observe(latency);
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
                &exponential_buckets(0.001, 2.0, N).unwrap(),
            )
            .unwrap()
            .build_recorder();

        metrics::with_local_recorder(&recorder, || {
            metrics::describe_histogram!("http_request_errors", "help text");
        });

        bencher.bench(|| {
            metrics::with_local_recorder(&recorder, || {
                for (kind, route, latency) in iter() {
                    let labels = [("kind", kind.to_str()), ("route", route)];
                    metrics::histogram!("http_request_errors", &labels).record(latency);
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
            Histogram::new(exponential_buckets(0.001, 2.0, N as u16))
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
            for (kind, route, latency) in iter() {
                h.get_or_create(&ErrorStatic { kind, route })
                    .observe(latency);
            }

            string.clear();
            encode(string, &registry).unwrap();
        });
    }

    fn thread_rng() -> SmallRng {
        SmallRng::seed_from_u64(
            BuildHasherDefault::<FxHasher>::default().hash_one(std::thread::current().id()),
        )
    }

    fn iter() -> impl Iterator<Item = (ErrorKind, &'static str, f64)> {
        let mut rng = thread_rng();
        std::iter::from_fn(move || {
            let route = rng.gen_range(0..routes().len());
            let error = rng.gen_range(0..errors().len());
            Some((errors()[error], routes()[route], rng.gen()))
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

#[divan::bench_group(sample_size = 5, sample_count = 500)]
mod no_cardinality {
    use std::time::Instant;

    use bytes::Bytes;
    use divan::{black_box, Bencher};
    use measured::metric::{group::Encoding, histogram::Thresholds, MetricFamilyEncoding};
    use prometheus::exponential_buckets;

    const LOOPS: usize = 20000;
    const N: usize = 8;

    #[inline(never)]
    fn measured_inner(
        encoder: &mut measured::text::TextEncoder,
        h: &measured::Histogram<N>,
    ) -> Bytes {
        use measured::metric::name::MetricName;

        for _ in 0..black_box(LOOPS) {
            h.start_timer();
        }

        const NAME: &MetricName = MetricName::from_static("http_request_errors");
        encoder.write_help(NAME, "help text");
        h.collect_into(NAME, encoder);
        encoder.finish()
    }

    #[divan::bench]
    fn measured(bencher: Bencher) {
        let h = measured::Histogram::new_metric(Thresholds::<N>::exponential_buckets(0.00001, 2.0));

        bencher
            .with_inputs(measured::text::TextEncoder::new)
            .bench_refs(|encoder| measured_inner(encoder, &h));
    }

    #[divan::bench]
    fn prometheus(bencher: Bencher) {
        let registry = prometheus::Registry::new();
        let h = prometheus::register_histogram_with_registry!(
            "http_request_errors",
            "help text",
            exponential_buckets(0.00001, 2.0, N).unwrap(),
            registry
        )
        .unwrap();

        bencher.with_inputs(String::new).bench_refs(|string| {
            for _ in 0..black_box(LOOPS) {
                let timer = h.start_timer();
                timer.stop_and_record();
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
                &exponential_buckets(0.00001, 2.0, N).unwrap(),
            )
            .unwrap()
            .build_recorder();

        metrics::with_local_recorder(&recorder, || {
            metrics::describe_histogram!("http_request_errors", "help text");
        });

        bencher.bench(|| {
            metrics::with_local_recorder(&recorder, || {
                for _ in 0..black_box(LOOPS) {
                    let start = Instant::now();
                    metrics::histogram!("http_request_errors")
                        .record(start.elapsed().as_secs_f64());
                }
            });

            recorder.handle().render()
        });
    }

    #[divan::bench]
    fn prometheus_client(bencher: Bencher) {
        use prometheus_client::encoding::text::encode;
        use prometheus_client::metrics::histogram::exponential_buckets;
        use prometheus_client::metrics::histogram::Histogram;
        use prometheus_client::registry::Registry;

        let mut registry = <Registry>::default();

        let h = Histogram::new(exponential_buckets(0.00001, 2.0, N as u16));

        // Register the metric family with the registry.
        registry.register(
            // With the metric name.
            "http_request_errors",
            // And the metric help text.
            "Number of HTTP requests received",
            h.clone(),
        );

        bencher.with_inputs(String::new).bench_refs(|string| {
            for _ in 0..black_box(LOOPS) {
                let start = Instant::now();
                h.observe(start.elapsed().as_secs_f64());
            }

            string.clear();
            encode(string, &registry).unwrap();
        });
    }
}
