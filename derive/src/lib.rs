use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

mod label_group;
mod metric_group;
mod value;

#[proc_macro_derive(FixedCardinalityLabel, attributes(label))]
pub fn derive_label(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match value::FixedCardinalityLabel::try_from(parse_macro_input!(input as DeriveInput)) {
        Ok(output) => output.to_token_stream(),
        Err(err) => err.into_compile_error().into_token_stream(),
    }
    .into()
}

#[proc_macro_derive(LabelGroup, attributes(label))]
pub fn derive_label_group(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match label_group::LabelGroup::try_from(parse_macro_input!(input as DeriveInput)) {
        Ok(output) => output.to_token_stream(),
        Err(err) => err.into_compile_error().into_token_stream(),
    }
    .into()
}

#[proc_macro_derive(MetricGroup, attributes(metric))]
pub fn derive_metric_group(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match metric_group::MetricGroup::try_from(parse_macro_input!(input as DeriveInput)) {
        Ok(output) => output.to_token_stream(),
        Err(err) => err.into_compile_error().into_token_stream(),
    }
    .into()
}

const CRATE: &str = "measured";
struct Krate(pub syn::Path);

impl Default for Krate {
    fn default() -> Self {
        Self(syn::Path {
            leading_colon: Some(Default::default()),
            segments: [syn::PathSegment::from(syn::Ident::new(
                CRATE,
                proc_macro2::Span::call_site(),
            ))]
            .into_iter()
            .collect(),
        })
    }
}
