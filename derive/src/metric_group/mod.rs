use proc_macro2::{Ident, Span};
use syn::punctuated::Punctuated;
use syn::{FnArg, Generics, Path, Token, Type};

mod attr;
mod parse;
mod to_tokens;

use attr::MetricGroupFieldAttrs;

pub struct MetricGroup {
    krate: Path,
    ident: Ident,
    fields: Vec<MetricGroupField>,
    generics: Generics,
    inputs: Option<Punctuated<FnArg, Token![,]>>,
}

#[derive(Clone)]
struct MetricGroupField {
    span: Span,
    name: Ident,
    attrs: MetricGroupFieldAttrs,
    ty: Type,
}

#[cfg(test)]
mod tests {
    use super::MetricGroup;
    fn parse(input: &str) -> Result<MetricGroup, syn::Error> {
        MetricGroup::try_from(syn::parse_str::<syn::DeriveInput>(input).unwrap())
    }
    #[test]
    fn initialization_requires_a_constructor() {
        for attr in [
            "init = Counter::new()",
            "metadata = ()",
            "label_set = Labels::new()",
        ] {
            let field = format!("struct Metrics {{ #[metric({attr})] count: Counter }}");
            let err = parse(&field).err().unwrap();
            assert!(
                err.to_string().contains("require `#[metric(new(...))]`"),
                "{err}"
            );
            assert!(
                parse(&format!("#[metric(new())] {field}")).is_ok(),
                "{attr}"
            );
        }
    }
    #[test]
    fn incompatible_and_duplicate_attributes_are_rejected() {
        for (attrs, expected) in [
            (
                "metadata = (), init = Counter::new()",
                "duplicate `metric(init)`",
            ),
            ("init = Counter::new(), metadata = ()", "not compatible"),
            (
                "init = Counter::new(), label_set = Labels::new()",
                "not compatible",
            ),
            (
                "rename = \"one\", rename = \"two\"",
                "duplicate `metric(rename)`",
            ),
        ] {
            let err = parse(&format!(
                "#[metric(new())] struct Metrics {{ #[metric({attrs})] time_seconds: Counter }}"
            ))
            .err()
            .unwrap();
            assert!(err.to_string().contains(expected), "{attrs}: {err}");
        }
        assert!(
            parse(
                "#[metric(new())] struct Metrics { #[metric(metadata = (), label_set = Labels::new())] count: Counter }"
            )
            .is_ok()
        );
    }
}
