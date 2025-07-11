use syn::{
    Attribute, LitStr, Token,
    parse::{Parse, ParseStream},
};

use crate::Krate;

const LABEL_ATTR: &str = "label";

#[derive(Default)]
pub struct ContainerAttrs {
    /// Optional `crate = $:path` arg
    pub krate: Option<Krate>,
    pub rename_all: Option<RenameAll>,
    pub singleton: Option<LitStr>,
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
                        "rename_all" => {
                            let _: Token![=] = input.parse()?;
                            if self.rename_all.replace(input.parse()?).is_some() {
                                return Err(input.error("duplicate `rename_all` arg"));
                            }
                        }
                        "singleton" => {
                            let _: Token![=] = input.parse()?;
                            if self.singleton.replace(input.parse()?).is_some() {
                                return Err(input.error("duplicate `singleton` arg"));
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
pub struct VariantAttrs {
    pub rename: Option<LitStr>,
}

impl VariantAttrs {
    pub fn parse_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut args = VariantAttrs { rename: None };
        for attr in attrs {
            if attr.path().is_ident(LABEL_ATTR) {
                attr.meta.require_list()?.parse_nested_meta(|meta| {
                    match () {
                        () if meta.path.is_ident("rename") => {
                            if args.rename.replace(meta.value()?.parse()?).is_some() {
                                return Err(meta.error("duplicate `label(rename)` arg"));
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

pub enum RenameAll {
    UpperCamel,
    LowerCamel,
    Snake,
    Kebab,
    ShoutySnake,
    ShoutyKebab,
    Title,
    Train,
}

impl RenameAll {
    pub fn apply(&self, s: &str) -> String {
        use heck::{
            ToKebabCase, ToLowerCamelCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
            ToTitleCase, ToTrainCase, ToUpperCamelCase,
        };
        match self {
            RenameAll::UpperCamel => s.to_upper_camel_case(),
            RenameAll::LowerCamel => s.to_lower_camel_case(),
            RenameAll::Snake => s.to_snake_case(),
            RenameAll::Kebab => s.to_kebab_case(),
            RenameAll::ShoutySnake => s.to_shouty_snake_case(),
            RenameAll::ShoutyKebab => s.to_shouty_kebab_case(),
            RenameAll::Title => s.to_title_case(),
            RenameAll::Train => s.to_train_case(),
        }
    }
}

impl Parse for RenameAll {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::LitStr = input.parse()?;
        match &*name.value() {
            "UpperCamelCase" => Ok(RenameAll::UpperCamel),
            "lowerCamelCase" => Ok(RenameAll::LowerCamel),
            "snake_case" => Ok(RenameAll::Snake),
            "kebab-case" => Ok(RenameAll::Kebab),
            "SHOUTY_SNAKE_CASE" => Ok(RenameAll::ShoutySnake),
            "SHOUTY-KEBAB-CASE" => Ok(RenameAll::ShoutyKebab),
            "Title Case" => Ok(RenameAll::Title),
            "Train-Case" => Ok(RenameAll::Train),

            _ => Err(input.error("unknown rename_all found")),
        }
    }
}
