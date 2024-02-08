use measured::metric::{group::Encoding, histogram::Thresholds, MetricFamilyEncoding};
use prometheus::exponential_buckets;

#[test]
fn measured() {
    use measured::metric::name::MetricName;

    let h = measured::Histogram::new_metric(Thresholds::<8>::exponential_buckets(0.1, 2.0));

    let mut encoder = measured::text::TextEncoder::new();

    for l in latencies() {
        h.observe(*l)
    }

    let metric = MetricName::from_static("http_request_errors");
    encoder.write_help(metric, "help text");
    h.collect_into(metric, &mut encoder);
    assert_eq!(
        &*encoder.finish(),
        br#"# HELP http_request_errors help text
# TYPE http_request_errors histogram
http_request_errors_bucket{le="0.1"} 3
http_request_errors_bucket{le="0.2"} 4
http_request_errors_bucket{le="0.4"} 6
http_request_errors_bucket{le="0.8"} 8
http_request_errors_bucket{le="1.6"} 9
http_request_errors_bucket{le="3.2"} 11
http_request_errors_bucket{le="6.4"} 14
http_request_errors_bucket{le="12.8"} 16
http_request_errors_bucket{le="+Inf"} 17
http_request_errors_sum 142.38
http_request_errors_count 17
"#
    );
}

#[test]
fn prometheus() {
    let registry = prometheus::Registry::new();
    let h = prometheus::register_histogram_with_registry!(
        "http_request_errors",
        "help text",
        exponential_buckets(0.1, 2.0, 8).unwrap(),
        registry
    )
    .unwrap();

    for l in latencies() {
        h.observe(*l)
    }

    let s = prometheus::TextEncoder
        .encode_to_string(&registry.gather())
        .unwrap();

    assert_eq!(
        s,
        r#"# HELP http_request_errors help text
# TYPE http_request_errors histogram
http_request_errors_bucket{le="0.1"} 3
http_request_errors_bucket{le="0.2"} 4
http_request_errors_bucket{le="0.4"} 6
http_request_errors_bucket{le="0.8"} 8
http_request_errors_bucket{le="1.6"} 9
http_request_errors_bucket{le="3.2"} 11
http_request_errors_bucket{le="6.4"} 14
http_request_errors_bucket{le="12.8"} 16
http_request_errors_bucket{le="+Inf"} 17
http_request_errors_sum 142.38
http_request_errors_count 17
"#
    );
}

#[test]
fn metrics() {
    let recorder = metrics_exporter_prometheus::PrometheusBuilder::new()
        .set_buckets_for_metric(
            metrics_exporter_prometheus::Matcher::Full("http_request_errors".to_string()),
            &exponential_buckets(0.1, 2.0, 8).unwrap(),
        )
        .unwrap()
        .build_recorder();

    metrics::with_local_recorder(&recorder, || {
        metrics::describe_histogram!("http_request_errors", "help text");
    });

    metrics::with_local_recorder(&recorder, || {
        for l in latencies() {
            metrics::histogram!("http_request_errors").record(*l);
        }
    });

    let output = recorder.handle().render();

    assert_eq!(
        output,
        r#"# HELP http_request_errors help text
# TYPE http_request_errors histogram
http_request_errors_bucket{le="0.1"} 3
http_request_errors_bucket{le="0.2"} 4
http_request_errors_bucket{le="0.4"} 6
http_request_errors_bucket{le="0.8"} 8
http_request_errors_bucket{le="1.6"} 9
http_request_errors_bucket{le="3.2"} 11
http_request_errors_bucket{le="6.4"} 14
http_request_errors_bucket{le="12.8"} 16
http_request_errors_bucket{le="+Inf"} 17
http_request_errors_sum 142.38
http_request_errors_count 17

"#
    );
}

#[test]
fn prometheus_client() {
    use prometheus_client::encoding::text::encode;
    use prometheus_client::metrics::histogram::{exponential_buckets, Histogram};
    use prometheus_client::registry::Registry;

    let mut registry = Registry::default();
    let h = Histogram::new(exponential_buckets(0.1, 2.0, 8));

    // Register the metric family with the registry.
    registry.register(
        // With the metric name.
        "http_request_errors",
        // And the metric help text.
        "help text",
        h.clone(),
    );

    for l in latencies() {
        h.observe(*l)
    }

    let mut output = String::new();
    encode(&mut output, &registry).unwrap();

    assert_eq!(
        output,
        r#"# HELP http_request_errors help text.
# TYPE http_request_errors histogram
http_request_errors_sum 142.38
http_request_errors_count 17
http_request_errors_bucket{le="0.1"} 3
http_request_errors_bucket{le="0.2"} 4
http_request_errors_bucket{le="0.4"} 6
http_request_errors_bucket{le="0.8"} 8
http_request_errors_bucket{le="1.6"} 9
http_request_errors_bucket{le="3.2"} 11
http_request_errors_bucket{le="6.4"} 14
http_request_errors_bucket{le="12.8"} 16
http_request_errors_bucket{le="+Inf"} 17
# EOF
"#
    );
}

fn latencies() -> &'static [f64] {
    &[
        0.02, 0.12, 0.05, 0.26, 0.7, 5.0, 0.4, 10.0, 0.5, 6.4, 5.44, 2.4, 1.67, 1.55, 100.0, 7.77,
        0.1,
    ]
}
