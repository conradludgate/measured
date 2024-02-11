use proc_macro2::Ident;
use syn::{Attribute, Path};

use crate::Krate;

const LABEL_ATTR: &str = "label";

#[derive(Default)]
pub struct ContainerAttrs {
    /// Optional `crate = $:path` arg
    pub krate: Option<Krate>,
    pub set: Option<Ident>,
}

impl ContainerAttrs {
    pub fn parse_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut args = ContainerAttrs::default();
        for attr in attrs {
            if attr.path().is_ident(LABEL_ATTR) {
                attr.meta.require_list()?.parse_nested_meta(|meta| {
                    match () {
                        () if meta.path.is_ident("crate") => {
                            if args.krate.replace(Krate(meta.value()?.parse()?)).is_some() {
                                return Err(meta.error("duplicate `label(crate)` arg"));
                            }
                        }
                        () if meta.path.is_ident("set") => {
                            if args.set.replace(meta.value()?.parse()?).is_some() {
                                return Err(meta.error("duplicate `label(set)` arg"));
                            }
                        }
                        () => return Err(meta.error("unknown argument found")),
                    }

                    Ok(())
                })?;
            }
        }
        Ok(args)
    }
}

#[derive(Clone)]
pub enum LabelGroupFieldAttrs {
    Fixed,
    FixedWith(Path),
    DynamicWith(Path),
}

impl LabelGroupFieldAttrs {
    pub fn get_sort_key(&self) -> LabelGroupFieldAttrsSortKey {
        match self {
            LabelGroupFieldAttrs::Fixed => LabelGroupFieldAttrsSortKey::Fixed,
            LabelGroupFieldAttrs::FixedWith(_) => LabelGroupFieldAttrsSortKey::Fixed,
            LabelGroupFieldAttrs::DynamicWith(_) => LabelGroupFieldAttrsSortKey::Dynamic,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum LabelGroupFieldAttrsSortKey {
    Dynamic,
    Fixed,
}

impl LabelGroupFieldAttrs {
    pub fn parse_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut args = None;
        for attr in attrs {
            if attr.path().is_ident(LABEL_ATTR) {
                attr.meta.require_list()?.parse_nested_meta(|meta| {
                    match () {
                        () if meta.path.is_ident("fixed") => {
                            if args.replace(Self::Fixed).is_some() {
                                return Err(meta.error("duplicate `label(fixed)` arg"));
                            }
                        }
                        () if meta.path.is_ident("fixed_with") => {
                            if args
                                .replace(Self::FixedWith(meta.value()?.parse()?))
                                .is_some()
                            {
                                return Err(meta.error("duplicate `label(fixed_with)` arg"));
                            }
                        }
                        () if meta.path.is_ident("dynamic_with") => {
                            if args
                                .replace(Self::DynamicWith(meta.value()?.parse()?))
                                .is_some()
                            {
                                return Err(meta.error("duplicate `label(dynamic_with)` arg"));
                            }
                        }
                        () => return Err(meta.error("unknown argument found")),
                    }

                    Ok(())
                })?;
            }
        }
        Ok(args.unwrap_or(Self::Fixed))
    }
}
