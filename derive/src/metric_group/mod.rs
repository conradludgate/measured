use proc_macro2::{Ident, Span};
use syn::{Generics, Path, Type};

mod attr;
mod parse;
mod to_tokens;

use attr::MetricGroupFieldAttrs;

pub struct MetricGroup {
    krate: Path,
    ident: Ident,
    fields: Vec<MetricGroupField>,
    generics: Generics,
}

#[derive(Clone)]
struct MetricGroupField {
    span: Span,
    name: Ident,
    attrs: MetricGroupFieldAttrs,
    ty: Type,
}
