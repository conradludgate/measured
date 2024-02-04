use proc_macro2::Ident;
use syn::{LitInt, Path};

use self::attr::{RenameAll, VariantAttrs};

mod attr;
mod parse;
mod to_tokens;

pub struct FixedCardinalityLabel {
    krate: Path,
    rename_all: Option<RenameAll>,
    ident: Ident,
    variants: Vec<FixedCardinalityLabelVariant>,
}

pub struct FixedCardinalityLabelVariant {
    attrs: VariantAttrs,
    ident: Ident,
    value: Option<LitInt>,
}
