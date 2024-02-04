use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

// this is some of the worst macro code I have ever written.
// to me in 5 months time, I am so sorry.
// i was really not feeling like dealing with syn so you have to deal with this monstosity of copy-pasta

mod group;
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
pub fn derive_group(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match group::LabelGroup::try_from(parse_macro_input!(input as DeriveInput)) {
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
