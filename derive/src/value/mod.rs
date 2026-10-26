use proc_macro2::{Ident, Span};
use syn::{LitInt, LitStr, Path};

use self::attr::{RenameAll, VariantAttrs};

mod attr;
mod parse;
mod to_tokens;

pub struct FixedCardinalityLabel {
    krate: Path,
    rename_all: RenameAll,
    ident: Ident,
    variants: Vec<FixedCardinalityLabelVariant>,
    singleton: Option<LitStr>,
}

pub struct FixedCardinalityLabelVariant {
    span: Span,
    attrs: VariantAttrs,
    ident: Ident,
    value: Option<LitInt>,
}

#[cfg(test)]
mod tests {
    use super::FixedCardinalityLabel;
    fn parse(input: &str) -> Result<FixedCardinalityLabel, syn::Error> {
        FixedCardinalityLabel::try_from(syn::parse_str::<syn::DeriveInput>(input).unwrap())
    }
    #[test]
    fn empty_and_duplicate_label_values_are_rejected() {
        for (input, expected) in [
            ("enum Label {}", "no variants"),
            (
                "enum Label { #[label(rename = \"same\")] A, #[label(rename = \"same\")] B }",
                "already used",
            ),
        ] {
            let err = parse(input).err().unwrap();
            assert!(err.to_string().contains(expected), "{err}");
        }
        assert!(parse("enum Label { #[label(rename = \"a\")] A, #[label(rename = \"A\")] B }").is_ok());
    }
}
