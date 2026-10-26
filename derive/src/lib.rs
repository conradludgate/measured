//! Derive macros for [`measured`](https://docs.rs/measured/latest/measured/).
//!
//! Most users should depend on `measured`, which re-exports these macros. Add `measured-derive`
//! directly only when you need the proc-macro crate itself.
//!
//! The crate provides three derives:
//!
//! - [`FixedCardinalityLabel`] implements compact encoding for enums with a finite set of values.
//!   Use `#[label(rename = "...")]`, `#[label(rename_all = "...")]`, or
//!   `#[label(singleton = "...")]` to customize it.
//! - [`LabelGroup`] turns a struct into a typed collection of labels. Its `#[label(...)]` attributes
//!   configure the generated label set and how each field is stored.
//! - [`MetricGroup`] groups metrics for collection. Its `#[metric(...)]` attributes configure
//!   names, namespaces, constructors, initialization, labels, help text, and units.
//!
//! See the [derive attribute reference](https://docs.rs/measured/latest/measured/) for the full
//! syntax and examples.

use quote::ToTokens;
use syn::{DeriveInput, parse_macro_input};

mod label_group;
mod metric_group;
mod value;

#[proc_macro_derive(FixedCardinalityLabel, attributes(label))]
/// Derive `FixedCardinalityLabel` and `LabelValue` for a finite set of label values.
pub fn derive_label(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match value::FixedCardinalityLabel::try_from(parse_macro_input!(input as DeriveInput)) {
        Ok(output) => output.to_token_stream(),
        Err(err) => err.into_compile_error().into_token_stream(),
    }
    .into()
}

#[proc_macro_derive(LabelGroup, attributes(label))]
/// Derive `LabelGroup` and its associated label set for a struct.
pub fn derive_label_group(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match label_group::LabelGroup::try_from(parse_macro_input!(input as DeriveInput)) {
        Ok(output) => output.to_token_stream(),
        Err(err) => err.into_compile_error().into_token_stream(),
    }
    .into()
}

#[proc_macro_derive(MetricGroup, attributes(metric))]
/// Derive metric collection for a struct containing metrics and nested groups.
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
