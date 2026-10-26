use proc_macro2::{Ident, Span};
use syn::{Generics, Path, Type, Visibility};

use self::attr::LabelGroupFieldAttrs;

mod attr;
mod parse;
mod to_tokens;

pub struct LabelGroup {
    vis: Visibility,
    krate: Path,
    set_ident: Option<Ident>,
    ident: Ident,
    fields: Vec<LabelGroupField>,
    generics: Generics,
}

#[derive(Clone)]
struct LabelGroupField {
    span: Span,
    vis: Visibility,
    name: Ident,
    attrs: LabelGroupFieldAttrs,
    ty: Type,
}

#[cfg(test)]
mod tests {
    use super::LabelGroup;
    #[test]
    fn duplicate_names_include_renamed_fields() {
        let parse = |input| LabelGroup::try_from(syn::parse_str::<syn::DeriveInput>(input).unwrap());
        let err = parse("struct Labels { a: String, #[label(rename = \"a\")] b: String }")
            .err()
            .unwrap();
        assert!(err.to_string().contains("duplicate label name `a`"));
        assert!(parse("struct Labels { a: String, #[label(rename = \"A\")] b: String }").is_ok());
    }
}
