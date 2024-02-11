use syn::{
    parenthesized, punctuated::Punctuated, spanned::Spanned, Attribute, Expr, ExprLit, FnArg, Lit,
    LitStr, Token,
};

use crate::Krate;

const LABEL_ATTR: &str = "metric";

#[derive(Default)]
pub struct ContainerAttrs {
    /// Optional `crate = $:path` arg
    pub krate: Option<Krate>,
    pub inputs: Option<Punctuated<FnArg, Token![,]>>,
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
                                return Err(meta.error("duplicate `metric(crate)` arg"));
                            }
                        }
                        () if meta.path.is_ident("new") => {
                            let content;
                            parenthesized!(content in meta.input);
                            let inputs =
                                Punctuated::<FnArg, Token![,]>::parse_terminated(&content)?;

                            if args.inputs.replace(inputs).is_some() {
                                return Err(meta.error("duplicate `metric(new)` arg"));
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
pub struct MetricGroupFieldAttrs {
    pub kind: MetricGroupFieldAttrsKind,
    pub docs: Option<String>,
    pub init: Option<Expr>,
}

#[derive(Clone)]
pub enum MetricGroupFieldAttrsKind {
    Metric { rename: Option<LitStr> },
    Group { namespace: Option<LitStr> },
}

impl MetricGroupFieldAttrs {
    pub fn parse_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut args = None;
        let mut docs = None;
        let mut init = None;

        for attr in attrs {
            if attr.path().is_ident(LABEL_ATTR) {
                attr.meta.require_list()?.parse_nested_meta(|meta| {
                    match () {
                        () if meta.path.is_ident("rename") => {
                            let arg = MetricGroupFieldAttrsKind::Metric {
                                rename: Some(meta.value()?.parse()?),
                            };
                            if args.replace(arg).is_some() {
                                return Err(meta.error("duplicate `metric(rename)` attr"));
                            }
                        }
                        () if meta.path.is_ident("namespace") => {
                            let arg = MetricGroupFieldAttrsKind::Group {
                                namespace: Some(meta.value()?.parse()?),
                            };
                            if args.replace(arg).is_some() {
                                return Err(meta.error("duplicate `metric(namespace)` attr"));
                            }
                        }
                        () if meta.path.is_ident("flatten") => {
                            let arg = MetricGroupFieldAttrsKind::Group { namespace: None };
                            if args.replace(arg).is_some() {
                                return Err(meta.error("duplicate `metric(flatten)` attr"));
                            }
                        }
                        () if meta.path.is_ident("init") => {
                            if init.replace(meta.value()?.parse()?).is_some() {
                                return Err(meta.error("duplicate `metric(init)` attr"));
                            }
                        }
                        () => return Err(meta.error("unknown argument found")),
                    }

                    Ok(())
                })?;
            } else if attr.path().is_ident("doc") {
                let expr = &attr.meta.require_name_value()?.value;
                let s = match expr {
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) => s,
                    _ => return Err(syn::Error::new(attr.span(), "invalid doc comment")),
                };
                docs.get_or_insert_with(String::new).push_str(&s.value());
            }
        }
        Ok(Self {
            kind: args.unwrap_or(MetricGroupFieldAttrsKind::Metric { rename: None }),
            docs,
            init,
        })
    }
}
