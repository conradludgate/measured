use proc_macro2::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::ParseStream, parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Fields,
    Generics, Path, PathSegment, Token,
};
use syn::{Field, Type, Visibility};

// this is some of the worst macro code I have ever written.
// to me in 5 months time, I am so sorry.
// i was really not feeling like dealing with syn so you have to deal with this monstosity of copy-pasta

#[proc_macro_derive(LabelGroup, attributes(label))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match LabelGroup::try_from(parse_macro_input!(input as DeriveInput)) {
        Ok(output) => output.to_token_stream(),
        Err(err) => err.into_compile_error().into_token_stream(),
    }
    .into()
}

struct LabelGroup {
    vis: Visibility,
    krate: Path,
    set_ident: Ident,
    ident: Ident,
    sorted_fields: Vec<LabelGroupField>,
    generics: Generics,
}

#[derive(Clone)]
struct LabelGroupField {
    vis: Visibility,
    name: Ident,
    attrs: LabelGroupFieldAttrs,
    ty: Type,
}

impl TryFrom<Field> for LabelGroupField {
    type Error = syn::Error;
    fn try_from(input: Field) -> syn::Result<Self> {
        let attrs = LabelGroupFieldAttrs::parse_attrs(&input.attrs)?;
        Ok(LabelGroupField {
            vis: input.vis,
            name: input.ident.unwrap(),
            ty: input.ty,
            attrs,
        })
    }
}

#[derive(Clone)]
enum LabelGroupFieldAttrs {
    Fixed,
    FixedWith(Path),
    DynamicWith(Path),
}

impl LabelGroupFieldAttrs {
    fn get_sort_key(&self) -> LabelGroupFieldAttrsSortKey {
        match self {
            LabelGroupFieldAttrs::Fixed => LabelGroupFieldAttrsSortKey::Fixed,
            LabelGroupFieldAttrs::FixedWith(_) => LabelGroupFieldAttrsSortKey::Fixed,
            LabelGroupFieldAttrs::DynamicWith(_) => LabelGroupFieldAttrsSortKey::Dynamic,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum LabelGroupFieldAttrsSortKey {
    Dynamic,
    Fixed,
}

impl LabelGroupFieldAttrs {
    fn parse_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
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
                            _ if input.peek(syn::Ident) => {
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
                            _ => return Err(input.error("unknown argument found")),
                        };

                        if args.replace(arg).is_some() {
                            return Err(syn::Error::new(attr.span(), "duplicate `label` attr"));
                        }
                    }
                    Ok(())
                })?
            }
        }
        args.ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                "missing cardinality attribute (`fixed`/`fixed_with`/`dynamic_with`)",
            )
        })
    }
}

impl TryFrom<DeriveInput> for LabelGroup {
    type Error = syn::Error;
    fn try_from(input: DeriveInput) -> syn::Result<Self> {
        let span = input.span();
        let DeriveInput {
            ident,
            data,
            generics,
            attrs,
            vis,
            ..
        } = input;

        let args = ContainerAttrs::parse_attrs(&attrs)?;
        let Krate(krate) = args.krate.unwrap_or_default();
        let set_ident = args
            .set
            .ok_or_else(|| syn::Error::new(span, "missing `set` attribute"))?;

        // predicate(&mut generics, krate.clone());

        let mut sorted_fields = match data {
            Data::Enum(_) => return Err(syn::Error::new(span, "enums not supported")),
            Data::Union(_) => return Err(syn::Error::new(span, "unions not supported")),
            Data::Struct(s) => match s.fields {
                Fields::Named(n) => n
                    .named
                    .into_iter()
                    .map(LabelGroupField::try_from)
                    .collect::<Result<Vec<_>, syn::Error>>()?,
                Fields::Unnamed(_) => {
                    return Err(syn::Error::new(span, "tuple structs not supported"))
                }
                Fields::Unit => return Err(syn::Error::new(span, "unit structs not supported")),
            },
        };

        sorted_fields.sort_by_key(|x| x.attrs.get_sort_key());

        Ok(Self {
            vis,
            krate,
            ident,
            generics,
            sorted_fields,
            set_ident,
        })
    }
}

const LABEL_ATTR: &str = "label";
const CRATE: &str = "measured";

#[derive(Default)]
struct ContainerAttrs {
    /// Optional `crate = $:path` arg
    krate: Option<Krate>,
    set: Option<Ident>,
}

impl ContainerAttrs {
    fn parse_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
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
                _ if input.peek(Token![crate]) => {
                    let _: Token![crate] = input.parse()?;
                    let _: Token![=] = input.parse()?;
                    if self.krate.replace(Krate(input.parse()?)).is_some() {
                        return Err(input.error("duplicate `crate` arg"));
                    }
                }
                _ if input.peek(syn::Ident) => {
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
                _ => return Err(input.error("unknown argument found")),
            }
        }
        Ok(self)
    }
}

struct Krate(Path);

impl Default for Krate {
    fn default() -> Self {
        Self(Path {
            leading_colon: Some(Default::default()),
            segments: [PathSegment::from(Ident::new(CRATE, Span::call_site()))]
                .into_iter()
                .collect(),
        })
    }
}

impl ToTokens for LabelGroup {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            vis,
            krate,
            ident,
            generics,
            sorted_fields,
            set_ident,
        } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let names = sorted_fields.iter().map(|x| x.name.to_string());
        let visits = sorted_fields.iter().map(|x| {
            let LabelGroupField { name, ty, .. } = x;
            quote! {
                <#ty as #krate::label::LabelValue>::visit(&self.#name, v);
            }
        });

        let set_fields = sorted_fields.iter().filter_map(|x| {
            let LabelGroupField {
                vis, name, attrs, ..
            } = x;
            match attrs {
                LabelGroupFieldAttrs::Fixed => None,
                LabelGroupFieldAttrs::FixedWith(ty) => Some(quote!( #vis #name: #ty, )),
                LabelGroupFieldAttrs::DynamicWith(ty) => Some(quote!( #vis #name: #ty, )),
            }
        });

        let part = sorted_fields
            .partition_point(|x| x.attrs.get_sort_key() == LabelGroupFieldAttrsSortKey::Dynamic);
        let (dynamics, fixed) = sorted_fields.split_at(part);

        let cardinalities: Vec<TokenStream> = fixed
            .iter()
            .map(|x| {
                let LabelGroupField {
                    name, attrs, ty, ..
                } = x;
                match attrs {
                    LabelGroupFieldAttrs::Fixed => quote!(<#ty as #krate::label::FixedCardinalityLabel>::cardinality()),
                    LabelGroupFieldAttrs::FixedWith(ty) => quote!(<#ty as #krate::label::FixedCardinalityDynamicLabel>::cardinality(&self.#name)),
                    LabelGroupFieldAttrs::DynamicWith(_) =>unreachable!(),
                }
            })
            .collect();

        let fixed_encodes: Vec<TokenStream> = fixed
            .iter()
            .map(|x| {
                let LabelGroupField {
                    name, attrs, ty, ..
                } = x;

                match attrs {
                    LabelGroupFieldAttrs::Fixed => quote!(<#ty as #krate::label::FixedCardinalityLabel>::encode(&value.#name)),
                    LabelGroupFieldAttrs::FixedWith(ty) => quote!(<#ty as #krate::label::FixedCardinalityDynamicLabel>::encode(&self.#name, &value.#name)?),
                    LabelGroupFieldAttrs::DynamicWith(_) =>unreachable!(),
                }
            })
            .collect();

        let dynamic_encodes: Vec<TokenStream> = dynamics
            .iter()
            .map(|x| {
                let LabelGroupField {
                    name, attrs, ..
                } = x;

                match attrs {
                    LabelGroupFieldAttrs::DynamicWith(ty) => quote!(<#ty as #krate::label::DynamicLabel>::encode(&self.#name, &value.#name)?),
                    _ => unreachable!(),
                }
            })
            .collect();

        let fixed_decodes: Vec<TokenStream> = fixed
            .iter()
            .map(|x| {
                let LabelGroupField {
                    name, attrs, ty, ..
                } = x;

                match attrs {
                    LabelGroupFieldAttrs::Fixed => quote!(let #name = <#ty as #krate::label::FixedCardinalityLabel>::decode(index1);),
                    LabelGroupFieldAttrs::FixedWith(ty) => quote!(let #name = <#ty as #krate::label::FixedCardinalityDynamicLabel>::decode(&self.#name, index1);),
                    LabelGroupFieldAttrs::DynamicWith(_) =>unreachable!(),
                }
            })
            .collect();

        let dynamic_indices = dynamics
            .iter()
            .enumerate()
            .map(|(i, _)| format_ident!("dynamic_index{i}"))
            .collect::<Vec<_>>();

        let dynamic_decodes: Vec<TokenStream> = dynamics
            .iter()
            .enumerate()
            .map(|(i, x)| {
                let LabelGroupField {
                    name, attrs, ..
                } = x;

                let index = &dynamic_indices[i];
                match attrs {
                    LabelGroupFieldAttrs::DynamicWith(ty) => quote!(let #name = <#ty as #krate::label::DynamicLabel>::decode(&self.#name, #index);),
                    _ => unreachable!(),
                }
            })
            .collect();

        let field_names = sorted_fields.iter().map(|x| {
            let LabelGroupField { name, .. } = x;
            name
        });

        let cardinality_fns = if dynamics.is_empty() {
            quote!(
                fn cardinality(&self) -> Option<usize> {
                    Some(1usize)
                        #( .and_then(|x| x.checked_mul(#cardinalities)) )*
                }
                type Unique = usize;
                fn encode_dense(&self, value: Self::Unique) -> Option<usize> {
                    Some(value)
                }
                fn decode_dense(&self, value: usize) -> Self::Group<'_> {
                    self.decode(&value)
                }
            )
        } else {
            let dynamics = dynamics.iter().map(|_| quote!(usize));
            quote!(
                fn cardinality(&self) -> Option<usize> {
                    None
                }
                type Unique = (usize, #(#dynamics),*);

                fn encode_dense(&self, _value: Self::Unique) -> Option<usize> {
                    None
                }
                fn decode_dense(&self, _value: usize) -> Self::Group<'_> {
                    unreachable!("does not have a dense encoding")
                }
            )
        };

        tokens.extend(quote! {
            #[automatically_derived]
            impl #impl_generics #krate::label::LabelGroup for #ident #ty_generics #where_clause {
                fn label_names() -> impl ::std::iter::IntoIterator<Item = &'static ::std::primitive::str> {
                    [#(#names),*]
                }

                fn label_values(&self, v: &mut impl #krate::label::LabelVisitor) {
                    #(#visits)*
                }
            }

            #vis struct #set_ident {
                #(#set_fields)*
            }

            #[automatically_derived]
            impl #krate::label::LabelGroupSet for #set_ident {
                // TODO: fix these generics
                type Group<'a> = #ident #ty_generics;

                #cardinality_fns

                fn encode(&self, value: Self::Group<'_>) -> Option<Self::Unique> {
                    let mut mul = 1;
                    let mut index = 0;

                    #(
                        index += #fixed_encodes * mul;
                        mul *= #cardinalities;
                    )*

                    Some((
                        index
                        #(, #dynamic_encodes)*
                    ))
                }

                fn decode(&self, value: &Self::Unique) -> Self::Group<'_> {
                    let (index #(, #dynamic_indices)*) = *value;

                    #(
                        let card = #cardinalities;
                        let (index, index1) = (index / card, index % card);
                        #fixed_decodes;
                    )*

                    debug_assert_eq!(index, 0);

                    #(
                        #dynamic_decodes;
                    )*

                    Self::Group {
                        #(#field_names,)*
                    }
                }
            }
        })
    }
}
