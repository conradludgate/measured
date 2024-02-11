use proc_macro2::{Ident, Span};
use syn::{punctuated::Punctuated, FnArg, Generics, Path, Token, Type};

mod attr;
mod parse;
mod to_tokens;

use attr::MetricGroupFieldAttrs;

pub struct MetricGroup {
    krate: Path,
    ident: Ident,
    fields: Vec<MetricGroupField>,
    generics: Generics,
    inputs: Option<Punctuated<FnArg, Token![,]>>
}

#[derive(Clone)]
struct MetricGroupField {
    span: Span,
    name: Ident,
    attrs: MetricGroupFieldAttrs,
    ty: Type,
}
