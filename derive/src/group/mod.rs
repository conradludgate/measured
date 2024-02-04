use proc_macro2::Ident;
use syn::{Generics, Path, Type, Visibility};

use self::attr::LabelGroupFieldAttrs;

mod attr;
mod parse;
mod to_tokens;

pub struct LabelGroup {
    vis: Visibility,
    krate: Path,
    set_ident: Ident,
    ident: Ident,
    sorted_fields: Vec<LabelGroupField>,
    generics: Generics,
}

#[derive(Clone)]
struct LabelGroupField {
    vis: Visibility,
    name: Ident,
    attrs: LabelGroupFieldAttrs,
    ty: Type,
}
