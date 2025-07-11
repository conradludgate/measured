use syn::{
    Attribute, Expr, ExprLit, FnArg, Lit, LitStr, Token, parenthesized, punctuated::Punctuated,
    spanned::Spanned,
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
    pub init: Option<MetricGroupFieldAttrsInit>,
}

#[derive(Clone)]
pub enum MetricGroupFieldAttrsKind {
    Metric { rename: Option<LitStr> },
    Group { namespace: Option<LitStr> },
}

#[derive(Clone)]
pub enum MetricGroupFieldAttrsInit {
    Raw(Expr),
    Metric {
        metadata: Option<Expr>,
        label_set: Option<Expr>,
    },
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
                            if init
                                .replace(MetricGroupFieldAttrsInit::Raw(meta.value()?.parse()?))
                                .is_some()
                            {
                                return Err(meta.error("duplicate `metric(init)` attr"));
                            }
                        }
                        () if meta.path.is_ident("metadata") => {
                            const DEFAULT: MetricGroupFieldAttrsInit =
                                MetricGroupFieldAttrsInit::Metric {
                                    metadata: None,
                                    label_set: None,
                                };
                            match init.get_or_insert(DEFAULT) {
                                MetricGroupFieldAttrsInit::Metric {
                                    metadata: metadata @ None,
                                    ..
                                } => {
                                    *metadata = Some(meta.value()?.parse()?);
                                }
                                MetricGroupFieldAttrsInit::Metric {
                                    metadata: Some(_), ..
                                } => {
                                    return Err(meta.error("duplicate `metric(metadata)` attr"));
                                }
                                MetricGroupFieldAttrsInit::Raw(_) => {
                                    return Err(meta.error("`metric(metadata)` and `metric(init)` attributes are not compatible"));
                                }
                            }
                        }
                        () if meta.path.is_ident("label_set") => {
                            const DEFAULT: MetricGroupFieldAttrsInit =
                                MetricGroupFieldAttrsInit::Metric {
                                    metadata: None,
                                    label_set: None,
                                };
                            match init.get_or_insert(DEFAULT) {
                                MetricGroupFieldAttrsInit::Metric {
                                    label_set: label_set @ None,
                                    ..
                                } => {
                                    *label_set = Some(meta.value()?.parse()?);
                                }
                                MetricGroupFieldAttrsInit::Metric {
                                    label_set: Some(_), ..
                                } => {
                                    return Err(meta.error("duplicate `metric(label_set)` attr"));
                                }
                                MetricGroupFieldAttrsInit::Raw(_) => {
                                    return Err(meta.error("`metric(label_set)` and `metric(init)` attributes are not compatible"));
                                }
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
