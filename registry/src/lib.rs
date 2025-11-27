use std::convert::Infallible;

use measured::{
    MetricGroup,
    metric::{MetricFamilyEncoding, group::Encoding, name::MetricName},
    text::BufferedTextEncoder,
};

#[doc(hidden)]
pub mod __private {
    pub use inventory::submit;
    pub use measured::metric::name::MetricName;
}

trait MetricEncoding {
    fn collect_family_into(&self, name: &'static MetricName, enc: &mut BufferedTextEncoder);
}

impl<M: MetricFamilyEncoding<BufferedTextEncoder>> MetricEncoding for M {
    fn collect_family_into(&self, name: &'static MetricName, enc: &mut BufferedTextEncoder) {
        let Ok(()) = self.collect_family_into(name, enc);
    }
}

pub struct Metric {
    name: &'static MetricName,
    description: Option<&'static str>,
    metric: &'static dyn MetricEncoding,
}

inventory::collect!(Metric);

#[macro_export]
macro_rules! metric {
    ($(
        $(#[doc = $doc:literal])?
        $vis:vis static $name:ident: $ty:ty = $value:expr;
    )*) => {
        $(
            $(#[doc = $doc])?
            #[allow(non_upper_case_globals)]
            $vis static $name: $ty = $value;

            $crate::__private::submit!($crate::Metric {
                name: $crate::__private::MetricName::from_str(stringify!($name)),
                description: 'desc: {
                    $(break 'desc Some($doc.trim());)?
                    #[allow(unreachable_code)]
                    None
                },
                metric: &$name,
            });
        )*
    };
}

pub struct GlobalRegistry;

impl MetricGroup<BufferedTextEncoder> for GlobalRegistry {
    fn collect_group_into(&self, enc: &mut BufferedTextEncoder) -> Result<(), Infallible> {
        for metric in inventory::iter::<Metric> {
            if let Some(desc) = metric.description {
                enc.write_help(metric.name, desc)?;
            }
            metric.metric.collect_family_into(metric.name, enc);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{GlobalRegistry, metric};
    use measured::{MetricGroup, text::BufferedTextEncoder};

    use bstr::ByteSlice;
    use bytes::{Buf, Bytes};

    metric!(
        /// The number of HTTP requests
        pub static http_requests: measured::Counter = measured::Counter::const_new();

        /// The number of HTTP errors
        pub static http_errors: measured::Counter = measured::Counter::const_new();
    );

    #[test]
    fn test_http_requests() {
        http_requests.inc();
        http_requests.inc();
        http_errors.inc();

        let metrics = encode();

        check_sections(metrics, &[
            b"# HELP http_errors The number of HTTP errors\n# TYPE http_errors counter\nhttp_errors 1\n",
            b"# HELP http_requests The number of HTTP requests\n# TYPE http_requests counter\nhttp_requests 2\n",
        ]);
    }

    fn encode() -> Bytes {
        let mut text_encoder = BufferedTextEncoder::new();
        let Ok(()) = GlobalRegistry.collect_group_into(&mut text_encoder);
        text_encoder.finish()
    }

    fn check_sections(mut bytes: Bytes, expected: &[&'static [u8]]) {
        let mut sections = Vec::new();
        while let Some(pos) = bytes.find(b"\n\n") {
            sections.push(bytes.split_to(pos + 1));
            bytes.advance(1); // skip the newline
        }
        sections.push(bytes);

        assert_eq!(sections.len(), expected.len());
        for e in expected {
            let e = Bytes::from_static(e);
            assert!(sections.contains(&e), "sections: {sections:?}");
        }
    }
}
