use lasso::{Rodeo, RodeoReader};
use measured::{
    label::StaticLabelSet,
    metric::{group::Encoding, MetricFamilyEncoding},
};
use measured_derive::{FixedCardinalityLabel, LabelGroup};
use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};

const LOOPS: usize = 2000;

#[test]
fn measured() {
    use measured::metric::name::{MetricName, Total};

    let error_set = ErrorsSet {
        kind: StaticLabelSet::new(),
        route: Rodeo::from_iter(routes()).into_reader(),
    };
    let counter_vec = measured::CounterVec::new(error_set);

    let mut encoder = measured::text::TextEncoder::new();

    for _ in 0..LOOPS {
        for &kind in errors() {
            for route in routes() {
                counter_vec.inc(Error { kind, route });
            }
        }
    }

    let metric = MetricName::from_static("http_request_errors").with_suffix(Total);
    encoder.write_help(&metric, "help text");
    counter_vec.collect_into(&metric, &mut encoder);
    assert_eq!(
        &*encoder.finish(),
        br#"# HELP http_request_errors_total help text
# TYPE http_request_errors_total counter
http_request_errors_total{kind="user",route="/api/v1/users"} 2000
http_request_errors_total{kind="user",route="/api/v1/users/:id"} 2000
http_request_errors_total{kind="user",route="/api/v1/products"} 2000
http_request_errors_total{kind="user",route="/api/v1/products/:id"} 2000
http_request_errors_total{kind="user",route="/api/v1/products/:id/owner"} 2000
http_request_errors_total{kind="user",route="/api/v1/products/:id/purchase"} 2000
http_request_errors_total{kind="internal",route="/api/v1/users"} 2000
http_request_errors_total{kind="internal",route="/api/v1/users/:id"} 2000
http_request_errors_total{kind="internal",route="/api/v1/products"} 2000
http_request_errors_total{kind="internal",route="/api/v1/products/:id"} 2000
http_request_errors_total{kind="internal",route="/api/v1/products/:id/owner"} 2000
http_request_errors_total{kind="internal",route="/api/v1/products/:id/purchase"} 2000
http_request_errors_total{kind="network",route="/api/v1/users"} 2000
http_request_errors_total{kind="network",route="/api/v1/users/:id"} 2000
http_request_errors_total{kind="network",route="/api/v1/products"} 2000
http_request_errors_total{kind="network",route="/api/v1/products/:id"} 2000
http_request_errors_total{kind="network",route="/api/v1/products/:id/owner"} 2000
http_request_errors_total{kind="network",route="/api/v1/products/:id/purchase"} 2000
"#
    );
}

#[test]
fn prometheus() {
    let registry = prometheus::Registry::new();
    let counter_vec = prometheus::register_int_counter_vec_with_registry!(
        "http_request_errors_total",
        "help text",
        &["kind", "route"],
        registry
    )
    .unwrap();

    for _ in 0..LOOPS {
        for &kind in errors() {
            for route in routes() {
                counter_vec.with_label_values(&[kind.to_str(), route]).inc();
            }
        }
    }

    let s = prometheus::TextEncoder
        .encode_to_string(&registry.gather())
        .unwrap();

    assert_eq!(
        s,
        r#"# HELP http_request_errors_total help text
# TYPE http_request_errors_total counter
http_request_errors_total{kind="internal",route="/api/v1/products"} 2000
http_request_errors_total{kind="internal",route="/api/v1/products/:id"} 2000
http_request_errors_total{kind="internal",route="/api/v1/products/:id/owner"} 2000
http_request_errors_total{kind="internal",route="/api/v1/products/:id/purchase"} 2000
http_request_errors_total{kind="internal",route="/api/v1/users"} 2000
http_request_errors_total{kind="internal",route="/api/v1/users/:id"} 2000
http_request_errors_total{kind="network",route="/api/v1/products"} 2000
http_request_errors_total{kind="network",route="/api/v1/products/:id"} 2000
http_request_errors_total{kind="network",route="/api/v1/products/:id/owner"} 2000
http_request_errors_total{kind="network",route="/api/v1/products/:id/purchase"} 2000
http_request_errors_total{kind="network",route="/api/v1/users"} 2000
http_request_errors_total{kind="network",route="/api/v1/users/:id"} 2000
http_request_errors_total{kind="user",route="/api/v1/products"} 2000
http_request_errors_total{kind="user",route="/api/v1/products/:id"} 2000
http_request_errors_total{kind="user",route="/api/v1/products/:id/owner"} 2000
http_request_errors_total{kind="user",route="/api/v1/products/:id/purchase"} 2000
http_request_errors_total{kind="user",route="/api/v1/users"} 2000
http_request_errors_total{kind="user",route="/api/v1/users/:id"} 2000
"#
    );
}

#[test]
fn metrics() {
    let recorder = metrics_exporter_prometheus::PrometheusBuilder::new().build_recorder();

    metrics::with_local_recorder(&recorder, || {
        metrics::describe_counter!("http_request_errors_total", "help text");
    });

    metrics::with_local_recorder(&recorder, || {
        for _ in 0..LOOPS {
            for &kind in errors() {
                for route in routes() {
                    let labels = [("kind", kind.to_str()), ("route", route)];
                    metrics::counter!("http_request_errors_total", &labels).increment(1);
                }
            }
        }
    });

    // output is unstable
    let output = recorder.handle().render();
    let mut lines: Vec<&str> = output.lines().collect();
    lines.sort_unstable();
    let output = lines.join("\n");

    assert_eq!(
        output,
        r#"
# HELP http_request_errors_total help text
# TYPE http_request_errors_total counter
http_request_errors_total{kind="internal",route="/api/v1/products"} 2000
http_request_errors_total{kind="internal",route="/api/v1/products/:id"} 2000
http_request_errors_total{kind="internal",route="/api/v1/products/:id/owner"} 2000
http_request_errors_total{kind="internal",route="/api/v1/products/:id/purchase"} 2000
http_request_errors_total{kind="internal",route="/api/v1/users"} 2000
http_request_errors_total{kind="internal",route="/api/v1/users/:id"} 2000
http_request_errors_total{kind="network",route="/api/v1/products"} 2000
http_request_errors_total{kind="network",route="/api/v1/products/:id"} 2000
http_request_errors_total{kind="network",route="/api/v1/products/:id/owner"} 2000
http_request_errors_total{kind="network",route="/api/v1/products/:id/purchase"} 2000
http_request_errors_total{kind="network",route="/api/v1/users"} 2000
http_request_errors_total{kind="network",route="/api/v1/users/:id"} 2000
http_request_errors_total{kind="user",route="/api/v1/products"} 2000
http_request_errors_total{kind="user",route="/api/v1/products/:id"} 2000
http_request_errors_total{kind="user",route="/api/v1/products/:id/owner"} 2000
http_request_errors_total{kind="user",route="/api/v1/products/:id/purchase"} 2000
http_request_errors_total{kind="user",route="/api/v1/users"} 2000
http_request_errors_total{kind="user",route="/api/v1/users/:id"} 2000"#
    );
}

#[test]
fn prometheus_client() {
    use prometheus_client::encoding::text::encode;
    use prometheus_client::metrics::counter::Counter;
    use prometheus_client::metrics::family::Family;
    use prometheus_client::registry::Registry;

    let mut registry = Registry::default();

    let counter_vec = Family::<ErrorStatic, Counter>::default();

    // Register the metric family with the registry.
    registry.register(
        // With the metric name.
        "http_request_errors",
        // And the metric help text.
        "help text",
        counter_vec.clone(),
    );

    for _ in 0..LOOPS {
        for &kind in errors() {
            for route in routes() {
                counter_vec
                    .get_or_create(&ErrorStatic { kind, route })
                    .inc();
            }
        }
    }

    let mut output = String::new();
    encode(&mut output, &registry).unwrap();

    // output is unstable
    let mut lines: Vec<&str> = output.lines().collect();
    lines.sort_unstable();
    let output = lines.join("\n");

    assert_eq!(
        output,
        r#"# EOF
# HELP http_request_errors help text.
# TYPE http_request_errors counter
http_request_errors_total{kind="Internal",route="/api/v1/products"} 2000
http_request_errors_total{kind="Internal",route="/api/v1/products/:id"} 2000
http_request_errors_total{kind="Internal",route="/api/v1/products/:id/owner"} 2000
http_request_errors_total{kind="Internal",route="/api/v1/products/:id/purchase"} 2000
http_request_errors_total{kind="Internal",route="/api/v1/users"} 2000
http_request_errors_total{kind="Internal",route="/api/v1/users/:id"} 2000
http_request_errors_total{kind="Network",route="/api/v1/products"} 2000
http_request_errors_total{kind="Network",route="/api/v1/products/:id"} 2000
http_request_errors_total{kind="Network",route="/api/v1/products/:id/owner"} 2000
http_request_errors_total{kind="Network",route="/api/v1/products/:id/purchase"} 2000
http_request_errors_total{kind="Network",route="/api/v1/users"} 2000
http_request_errors_total{kind="Network",route="/api/v1/users/:id"} 2000
http_request_errors_total{kind="User",route="/api/v1/products"} 2000
http_request_errors_total{kind="User",route="/api/v1/products/:id"} 2000
http_request_errors_total{kind="User",route="/api/v1/products/:id/owner"} 2000
http_request_errors_total{kind="User",route="/api/v1/products/:id/purchase"} 2000
http_request_errors_total{kind="User",route="/api/v1/users"} 2000
http_request_errors_total{kind="User",route="/api/v1/users/:id"} 2000"#
    );
}

fn routes() -> &'static [&'static str] {
    &[
        "/api/v1/users",
        "/api/v1/users/:id",
        "/api/v1/products",
        "/api/v1/products/:id",
        "/api/v1/products/:id/owner",
        "/api/v1/products/:id/purchase",
    ]
}

fn errors() -> &'static [ErrorKind] {
    &[ErrorKind::User, ErrorKind::Internal, ErrorKind::Network]
}

#[derive(Clone, Copy, PartialEq, Debug, LabelGroup)]
#[label(set = ErrorsSet)]
struct Error<'a> {
    kind: ErrorKind,
    #[label(fixed_with = RodeoReader)]
    route: &'a str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ErrorStatic {
    kind: ErrorKind,
    route: &'static str,
}

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq, FixedCardinalityLabel, EncodeLabelValue)]
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
