use syn::{parse::ParseStream, spanned::Spanned, Attribute, LitStr, Token};

use crate::Krate;

const LABEL_ATTR: &str = "metric";

#[derive(Default)]
pub struct ContainerAttrs {
    /// Optional `crate = $:path` arg
    pub krate: Option<Krate>,
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
                () => return Err(input.error("unknown argument found")),
            }
        }
        Ok(self)
    }
}

#[derive(Clone)]
pub enum MetricGroupFieldAttrs {
    Metric { rename: Option<LitStr> },
    Group { namespace: Option<LitStr> },
}

impl MetricGroupFieldAttrs {
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
                                    "rename" => {
                                        let _: Token![=] = input.parse()?;

                                        Self::Metric {
                                            rename: Some(input.parse()?),
                                        }
                                    }
                                    "namespace" => {
                                        let _: Token![=] = input.parse()?;
                                        Self::Group {
                                            namespace: Some(input.parse()?),
                                        }
                                    }
                                    "flatten" => Self::Group { namespace: None },
                                    _ => return Err(input.error("unknown argument found")),
                                }
                            }
                            () => return Err(input.error("unknown argument found")),
                        };

                        if args.replace(arg).is_some() {
                            return Err(syn::Error::new(attr.span(), "duplicate `metric` attr"));
                        }
                    }
                    Ok(())
                })?;
            }
        }
        Ok(args.unwrap_or(Self::Metric { rename: None }))
    }
}
