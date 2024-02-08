pub use crate::label::ComposedGroup;

use super::{
    name::{MetricNameEncoder, WithNamespace},
    MetricEncoding,
};

pub trait Encoding {
    /// Write the help text for a metric
    fn write_help(&mut self, name: impl MetricNameEncoder, help: &str);
}

impl<E: Encoding> Encoding for &mut E {
    fn write_help(&mut self, name: impl MetricNameEncoder, help: &str) {
        E::write_help(self, name, help);
    }
}

pub trait MetricGroupEncoding<Enc: Encoding> {
    fn encode(&self, enc: &mut Enc);
}

impl<A, B, E> MetricGroupEncoding<E> for ComposedGroup<A, B>
where
    A: MetricGroupEncoding<E>,
    B: MetricGroupEncoding<E>,
    E: Encoding,
{
    fn encode(&self, enc: &mut E) {
        self.0.encode(enc);
        self.1.encode(enc);
    }
}

impl<G, E> MetricGroupEncoding<E> for WithNamespace<G>
where
    G: for<'a> MetricGroupEncoding<WithNamespace<&'a mut E>>,
    E: Encoding,
{
    fn encode(&self, enc: &mut E) {
        self.inner.encode(&mut WithNamespace {
            namespace: self.namespace,
            inner: enc,
        });
    }
}

impl<E: Encoding> Encoding for WithNamespace<E> {
    fn write_help(&mut self, name: impl MetricNameEncoder, help: &str) {
        self.inner.write_help(
            WithNamespace {
                namespace: self.namespace,
                inner: name,
            },
            help,
        )
    }
}

impl<M: MetricEncoding<E>, E> MetricEncoding<WithNamespace<E>> for M {
    fn write_type(name: impl MetricNameEncoder, enc: &mut WithNamespace<E>) {
        M::write_type(
            WithNamespace {
                namespace: enc.namespace,
                inner: name,
            },
            &mut enc.inner,
        );
    }
    fn collect_into(
        &self,
        metadata: &M::Metadata,
        labels: impl crate::label::LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut WithNamespace<E>,
    ) {
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
impl<'a, M: MetricEncoding<E>, E> MetricEncoding<&'a mut E> for M {
    fn write_type(name: impl MetricNameEncoder, enc: &mut &'a mut E) {
        M::write_type(name, *enc);
    }
    fn collect_into(
        &self,
        metadata: &M::Metadata,
        labels: impl crate::label::LabelGroup,
        name: impl MetricNameEncoder,
        enc: &mut &'a mut E,
    ) {
        self.collect_into(metadata, labels, name, *enc)
    }
}

#[cfg(all(feature = "lasso", test))]
mod tests {
    use lasso::{Rodeo, RodeoReader};
    use measured_derive::{FixedCardinalityLabel, LabelGroup};
    use prometheus_client::encoding::EncodeLabelValue;

    use crate::{
        label::StaticLabelSet,
        metric::{
            name::{MetricName, WithNamespace},
            MetricFamilyEncoding,
        },
        text::TextEncoder,
        CounterVec,
    };

    use super::{Encoding, MetricGroupEncoding};

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

    // #[derive(MetricGroupEncoder)]
    struct MyMetrics {
        /// help text
        errors: CounterVec<ErrorsSet>,
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

    #[test]
    fn http_errors() {
        let group = MyMetrics {
            errors: CounterVec::new(ErrorsSet {
                kind: StaticLabelSet::new(),
                route: Rodeo::from_iter(routes()).into_reader(),
            }),
        };
        let http_request_group = WithNamespace::new("http_request", group);

        let mut text_encoder = TextEncoder::new();
        http_request_group.encode(&mut text_encoder);
        assert_eq!(
            text_encoder.finish(),
            br#"# HELP http_request_errors help text
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
"#[..]
        );
    }

    // TODO: macro

    impl<Enc> MetricGroupEncoding<Enc> for MyMetrics
    where
        Enc: Encoding,
        CounterVec<ErrorsSet>: MetricFamilyEncoding<Enc>,
    {
        fn encode(&self, enc: &mut Enc) {
            const ERRORS: &MetricName = MetricName::from_static("errors");
            enc.write_help(ERRORS, "help text");
            self.errors.collect_into(ERRORS, enc);
        }
    }
}
