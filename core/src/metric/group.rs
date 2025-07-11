//! Groups of metrics

use std::sync::Arc;

pub use crate::label::ComposedGroup;

use super::{
    MetricEncoding,
    name::{MetricNameEncoder, WithNamespace},
};

/// Values that prometheus supports in the text format
#[derive(Clone, Copy, Debug)]
pub enum MetricValue {
    Int(i64),
    Float(f64),
}

/// Base trait of a metric encoder.
pub trait Encoding {
    /// The error type that this type might produce when encoding metric values.
    type Err;

    /// Write the help text for a metric
    fn write_help(&mut self, name: impl MetricNameEncoder, help: &str) -> Result<(), Self::Err>;
}

impl<E: Encoding> Encoding for &mut E {
    type Err = E::Err;
    fn write_help(&mut self, name: impl MetricNameEncoder, help: &str) -> Result<(), Self::Err> {
        E::write_help(self, name, help)
    }
}

/// A `MetricGroup` defines a group of [`MetricFamilyEncoding`](super::MetricFamilyEncoding)s
pub trait MetricGroup<Enc: Encoding> {
    /// Collect the group of metric families into the encoder
    fn collect_group_into(&self, enc: &mut Enc) -> Result<(), Enc::Err>;
}

impl<G, E> MetricGroup<E> for &G
where
    G: MetricGroup<E>,
    E: Encoding,
{
    fn collect_group_into(&self, enc: &mut E) -> Result<(), E::Err> {
        G::collect_group_into(self, enc)
    }
}

impl<A, B, E> MetricGroup<E> for ComposedGroup<A, B>
where
    A: MetricGroup<E>,
    B: MetricGroup<E>,
    E: Encoding,
{
    fn collect_group_into(&self, enc: &mut E) -> Result<(), E::Err> {
        self.0.collect_group_into(enc)?;
        self.1.collect_group_into(enc)?;
        Ok(())
    }
}

impl<G, E> MetricGroup<E> for WithNamespace<G>
where
    G: for<'a> MetricGroup<WithNamespace<&'a mut E>>,
    E: Encoding,
{
    fn collect_group_into(&self, enc: &mut E) -> Result<(), E::Err> {
        self.inner.collect_group_into(&mut WithNamespace {
            namespace: self.namespace,
            inner: enc,
        })
    }
}

impl<M: MetricGroup<T>, T: Encoding> MetricGroup<T> for Option<M> {
    fn collect_group_into(&self, enc: &mut T) -> Result<(), T::Err> {
        if let Some(this) = self {
            this.collect_group_into(enc)?;
        }
        Ok(())
    }
}

impl<M: MetricGroup<T>, T: Encoding> MetricGroup<T> for Arc<M> {
    fn collect_group_into(&self, enc: &mut T) -> Result<(), T::Err> {
        M::collect_group_into(self, enc)
    }
}

impl<E: Encoding> Encoding for WithNamespace<E> {
    type Err = E::Err;
    fn write_help(&mut self, name: impl MetricNameEncoder, help: &str) -> Result<(), Self::Err> {
        self.inner.write_help(
            WithNamespace {
                namespace: self.namespace,
                inner: name,
            },
            help,
        )
    }
}

impl<M: MetricEncoding<E>, E: Encoding> MetricEncoding<WithNamespace<E>> for M {
    fn write_type(name: impl MetricNameEncoder, enc: &mut WithNamespace<E>) -> Result<(), E::Err> {
        M::write_type(
            WithNamespace {
                namespace: enc.namespace,
                inner: name,
            },
            &mut enc.inner,
        )
    }
    fn collect_into(
        &self,
        metadata: &M::Metadata,
        labels: impl crate::label::LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut WithNamespace<E>,
    ) -> Result<(), E::Err> {
        self.collect_into(
            metadata,
            labels,
            WithNamespace {
                namespace: enc.namespace,
                inner: name,
            },
            &mut enc.inner,
        )
    }
}

impl<'a, M: MetricEncoding<E>, E: Encoding> MetricEncoding<&'a mut E> for M {
    fn write_type(name: impl MetricNameEncoder, enc: &mut &'a mut E) -> Result<(), E::Err> {
        M::write_type(name, *enc)
    }
    fn collect_into(
        &self,
        metadata: &M::Metadata,
        labels: impl crate::label::LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut &'a mut E,
    ) -> Result<(), E::Err> {
        self.collect_into(metadata, labels, name, *enc)
    }
}

#[cfg(all(feature = "lasso", test))]
mod tests {
    use lasso::{Rodeo, RodeoReader};
    use measured_derive::{FixedCardinalityLabel, LabelGroup, MetricGroup};
    use prometheus_client::encoding::EncodeLabelValue;

    use crate::{
        Counter, CounterVec, Gauge, Histogram, metric::histogram::Thresholds,
        text::BufferedTextEncoder,
    };

    use super::MetricGroup;

    #[derive(Clone, Copy, PartialEq, Debug, LabelGroup)]
    #[label(crate = crate, set = ErrorsSet)]
    struct Error<'a> {
        kind: ErrorKind,
        #[label(fixed_with = RodeoReader)]
        route: &'a str,
    }

    #[derive(Clone, Copy, PartialEq, Debug, Hash, Eq, FixedCardinalityLabel, EncodeLabelValue)]
    #[label(crate = crate)]
    enum ErrorKind {
        User,
        Internal,
        Network,
    }

    #[derive(MetricGroup)]
    #[metric(crate = crate)]
    #[metric(new(route: RodeoReader))]
    struct MyHttpMetrics {
        /// more help wow
        #[metric(label_set = ErrorsSet::new(route))]
        errors: CounterVec<ErrorsSet>,
    }

    #[derive(MetricGroup)]
    #[metric(crate = crate)]
    #[metric(new(route: RodeoReader))]
    struct MyMetrics {
        /// help text
        events_total: Counter,

        /// help text
        cool_factor: Gauge,

        /// help text
        #[metric(metadata = Thresholds::exponential_buckets(1.0, 2.0))]
        latency: Histogram<8>,

        #[metric(namespace = "http_request")]
        #[metric(init = MyHttpMetrics::new(route))]
        http: MyHttpMetrics,
    }

    #[test]
    fn http_errors() {
        let route_array = [
            "/api/v1/users",
            "/api/v1/users/:id",
            "/api/v1/products",
            "/api/v1/products/:id",
            "/api/v1/products/:id/owner",
            "/api/v1/products/:id/purchase",
        ];
        let routes = Rodeo::from_iter(route_array).into_reader();
        let group = MyMetrics::new(routes);

        for kind in [ErrorKind::Internal, ErrorKind::Network, ErrorKind::User] {
            for route in route_array {
                group
                    .http
                    .errors
                    .get_metric(group.http.errors.with_labels(Error { kind, route }));
            }
        }

        group.cool_factor.set(42);

        group.events_total.inc();
        group.latency.observe(4.0);

        let mut text_encoder = BufferedTextEncoder::new();
        group.collect_group_into(&mut text_encoder).unwrap();
        assert_eq!(
            text_encoder.finish(),
            r#"# HELP events_total help text
# TYPE events_total counter
events_total 1

# HELP cool_factor help text
# TYPE cool_factor gauge
cool_factor 42

# HELP latency help text
# TYPE latency histogram
latency_bucket{le="1.0"} 0
latency_bucket{le="2.0"} 0
latency_bucket{le="4.0"} 1
latency_bucket{le="8.0"} 1
latency_bucket{le="16.0"} 1
latency_bucket{le="32.0"} 1
latency_bucket{le="64.0"} 1
latency_bucket{le="128.0"} 1
latency_bucket{le="+Inf"} 1
latency_sum 4.0
latency_count 1

# HELP http_request_errors more help wow
# TYPE http_request_errors counter
http_request_errors{kind="user",route="/api/v1/users"} 0
http_request_errors{kind="user",route="/api/v1/users/:id"} 0
http_request_errors{kind="user",route="/api/v1/products"} 0
http_request_errors{kind="user",route="/api/v1/products/:id"} 0
http_request_errors{kind="user",route="/api/v1/products/:id/owner"} 0
http_request_errors{kind="user",route="/api/v1/products/:id/purchase"} 0
http_request_errors{kind="internal",route="/api/v1/users"} 0
http_request_errors{kind="internal",route="/api/v1/users/:id"} 0
http_request_errors{kind="internal",route="/api/v1/products"} 0
http_request_errors{kind="internal",route="/api/v1/products/:id"} 0
http_request_errors{kind="internal",route="/api/v1/products/:id/owner"} 0
http_request_errors{kind="internal",route="/api/v1/products/:id/purchase"} 0
http_request_errors{kind="network",route="/api/v1/users"} 0
http_request_errors{kind="network",route="/api/v1/users/:id"} 0
http_request_errors{kind="network",route="/api/v1/products"} 0
http_request_errors{kind="network",route="/api/v1/products/:id"} 0
http_request_errors{kind="network",route="/api/v1/products/:id/owner"} 0
http_request_errors{kind="network",route="/api/v1/products/:id/purchase"} 0
"#
        );
    }
}
