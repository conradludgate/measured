use proc_macro2::Ident;
use syn::{parse::ParseStream, spanned::Spanned, Attribute, Path, Token};

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
                args = attr.parse_args_with(|input: ParseStream| args.parse(input))?;
            }
        }
        Ok(args)
    }
}

impl ContainerAttrs {
    fn parse(mut self, input: ParseStream) -> syn::Result<Self> {
        let mut first = true;
        while !input.is_empty() {
            if !first {
                input.parse::<Token![,]>()?;
            }
            first = false;

            match () {
                () if input.peek(Token![crate]) => {
                    let _: Token![crate] = input.parse()?;
                    let _: Token![=] = input.parse()?;
                    if self.krate.replace(Krate(input.parse()?)).is_some() {
                        return Err(input.error("duplicate `crate` arg"));
                    }
                }
                () if input.peek(syn::Ident) => {
                    let name: syn::Ident = input.parse()?;
                    match &*name.to_string() {
                        "set" => {
                            let _: Token![=] = input.parse()?;
                            if self.set.replace(input.parse()?).is_some() {
                                return Err(input.error("duplicate `set` arg"));
                            }
                        }
                        _ => return Err(input.error("unknown argument found")),
                    }
                }
                () => return Err(input.error("unknown argument found")),
            }
        }
        Ok(self)
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
                attr.parse_args_with(|input: ParseStream| {
                    let mut first = true;
                    while !input.is_empty() {
                        if !first {
                            input.parse::<Token![,]>()?;
                        }
                        first = false;

                        let arg = match () {
                            () if input.peek(syn::Ident) => {
                                let name: syn::Ident = input.parse()?;
                                match &*name.to_string() {
                                    "fixed" => Self::Fixed,
                                    "fixed_with" => {
                                        let _: Token![=] = input.parse()?;
                                        Self::FixedWith(input.parse()?)
                                    }
                                    "dynamic_with" => {
                                        let _: Token![=] = input.parse()?;
                                        Self::DynamicWith(input.parse()?)
                                    }
                                    _ => return Err(input.error("unknown argument found")),
                                }
                            }
                            () => return Err(input.error("unknown argument found")),
                        };

                        if args.replace(arg).is_some() {
                            return Err(syn::Error::new(attr.span(), "duplicate `label` attr"));
                        }
                    }
                    Ok(())
                })?;
            }
        }
        Ok(args.unwrap_or(Self::Fixed))
    }
}
