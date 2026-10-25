//! Groups of metrics

use std::sync::Arc;

pub use crate::label::ComposedGroup;

use super::{
    MetricEncoding,
    name::{MetricNameEncoder, WithNamespace},
};

/// Base trait of a metric encoder.
pub trait Encoding {
    /// The error type that this type might produce when encoding metric values.
    type Err;

    /// The prometheus Content-Type that this Encoding exports as
    /// <https://prometheus.io/docs/instrumenting/content_negotiation/#protocol-headers>
    const MIME_TYPE: &'static str;

    /// Write the help text for a metric
    fn write_help(&mut self, name: impl MetricNameEncoder, help: &str) -> Result<(), Self::Err>;
}

impl<E: Encoding> Encoding for &mut E {
    type Err = E::Err;

    const MIME_TYPE: &'static str = E::MIME_TYPE;

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

    const MIME_TYPE: &'static str = E::MIME_TYPE;

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
