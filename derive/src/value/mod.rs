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
