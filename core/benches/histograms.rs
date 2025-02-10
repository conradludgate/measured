fn main() {
    divan::Divan::from_args().threads([0]).run_benches();
}

#[divan::bench_group(sample_size = 100000, sample_count = 500)]
mod fixed_cardinality {
    use std::{cell::RefCell, hash::BuildHasher};

    use divan::{black_box, Bencher};
    use foldhash::fast::{FixedState, RandomState};
    use lasso::{Rodeo, RodeoReader, Spur};
    use measured::{label::StaticLabelSet, metric::histogram::Thresholds};
    use measured_derive::{FixedCardinalityLabel, LabelGroup};
    use prometheus::exponential_buckets;
    use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};
    use rand::{rngs::SmallRng, Rng, SeedableRng};

    const N: usize = 8;

    #[divan::bench]
    fn measured(bencher: Bencher) {
        let error_set = ErrorsSet {
            kind: StaticLabelSet::new(),
            route: Rodeo::from_iter(routes()).into_reader(),
        };
        let h = measured::HistogramVec::with_label_set_and_metadata(
            error_set,
            Thresholds::<N>::exponential_buckets(0.001, 2.0),
        );

        thread_local! {
            static RNG: RefCell<SmallRng> = RefCell::new(thread_rng());
        }

        bencher.bench(|| {
            let (kind, route, latency) = RNG.with(|rng| get(&mut *rng.borrow_mut()));
            h.observe(Error { kind, route }, latency);
        });
    }

    #[divan::bench]
    fn measured_sparse(bencher: Bencher) {
        let error_set = ErrorsSet {
            kind: StaticLabelSet::new(),
            route: Rodeo::from_iter(routes()).into_reader(),
        };
        let h = measured::HistogramVec::sparse_with_label_set_and_metadata(
            error_set,
            Thresholds::<N>::exponential_buckets(0.001, 2.0),
        );

        thread_local! {
            static RNG: RefCell<SmallRng> = RefCell::new(thread_rng());
        }

        bencher.bench(|| {
            let (kind, route, latency) = RNG.with(|rng| get(&mut *rng.borrow_mut()));
            h.observe(Error { kind, route }, latency);
        });
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

        thread_local! {
            static RNG: RefCell<SmallRng> = RefCell::new(thread_rng());
        }

        bencher.bench(|| {
            let (kind, route, latency) = RNG.with(|rng| get(&mut *rng.borrow_mut()));
            counter_vec
                .with_label_values(&[kind.to_str(), route])
                .observe(latency);
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

        let h = recorder.handle();
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || loop {
            if rx
                .recv_timeout(std::time::Duration::from_millis(200))
                .is_ok()
            {
                return;
            }
            h.run_upkeep();
        });

        metrics::with_local_recorder(&recorder, || {
            metrics::describe_histogram!("http_request_errors", "help text");
        });

        thread_local! {
            static RNG: RefCell<SmallRng> = RefCell::new(thread_rng());
        }

        bencher.bench(|| {
            metrics::with_local_recorder(&recorder, || {
                let (kind, route, latency) = RNG.with(|rng| get(&mut *rng.borrow_mut()));
                let labels = [("kind", kind.to_str()), ("route", route)];
                metrics::histogram!("http_request_errors", &labels).record(latency);
            });
        });

        _ = tx.send(());
    }

    #[divan::bench]
    fn prometheus_client(bencher: Bencher) {
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

        thread_local! {
            static RNG: RefCell<SmallRng> = RefCell::new(thread_rng());
        }

        bencher.bench(|| {
            let (kind, route, latency) = RNG.with(|rng| get(&mut *rng.borrow_mut()));
            h.get_or_create(&ErrorStatic { kind, route })
                .observe(latency);
        });
    }

    fn thread_rng() -> SmallRng {
        SmallRng::seed_from_u64(FixedState::with_seed(0).hash_one(std::thread::current().id()))
    }

    fn get(rng: &mut impl Rng) -> (ErrorKind, &'static str, f64) {
        let route = rng.gen_range(0..routes().len());
        let error = rng.gen_range(0..errors().len());
        (errors()[error], routes()[route], rng.gen())
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
        #[label(fixed_with = RodeoReader<Spur, RandomState>)]
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

#[divan::bench_group(sample_size = 100000, sample_count = 500)]
mod no_cardinality {
    use std::time::Instant;

    use divan::Bencher;
    use measured::metric::histogram::Thresholds;
    use prometheus::exponential_buckets;

    const N: usize = 8;

    #[divan::bench]
    fn measured(bencher: Bencher) {
        let h =
            measured::Histogram::with_metadata(Thresholds::<N>::exponential_buckets(0.00001, 2.0));

        bencher.bench(|| drop(h.start_timer()));
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

        bencher.bench(|| h.start_timer().stop_and_record());
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

        let h = recorder.handle();
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || loop {
            if rx
                .recv_timeout(std::time::Duration::from_millis(200))
                .is_ok()
            {
                return;
            }
            h.run_upkeep();
        });

        let h = metrics::with_local_recorder(&recorder, || {
            metrics::describe_histogram!("http_request_errors", "help text");
            metrics::histogram!("http_request_errors")
        });

        bencher.bench(|| {
            let start = Instant::now();
            h.record(start.elapsed().as_secs_f64());
        });

        _ = tx.send(());
    }

    #[divan::bench]
    fn prometheus_client(bencher: Bencher) {
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

        bencher.bench(|| {
            let start = Instant::now();
            h.observe(start.elapsed().as_secs_f64());
        });
    }
}
