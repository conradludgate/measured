use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote, quote_spanned};

use super::attr::{LabelGroupFieldAttrsKind, LabelGroupFieldAttrsSortKey};
use super::{LabelGroup, LabelGroupField};

impl ToTokens for LabelGroup {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            krate,
            ident,
            generics,
            fields,
            ..
        } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let visits = fields.iter().map(|x| {
            let LabelGroupField { name, attrs, .. } = x;
            let name_string = attrs.rename.as_ref().map_or_else(|| name.to_string(), |r| r.value());
            let ident = format_ident!("{}", name_string.to_shouty_snake_case(), span = x.span);
            quote_spanned! { x.span =>
                const #ident: &#krate::label::LabelName = #krate::label::LabelName::from_str(#name_string);
                #krate::label::LabelGroupVisitor::write_value(v, #ident, &self.#name);
            }
        });

        let label_group_set = Set(self);

        tokens.extend(quote! {
            #[automatically_derived]
            impl #impl_generics #krate::label::LabelGroup for #ident #ty_generics #where_clause {
                fn visit_values(&self, v: &mut impl #krate::label::LabelGroupVisitor) {
                    #(#visits)*
                }
            }

            #label_group_set
        });
    }
}

struct Set<'a>(&'a LabelGroup);

impl ToTokens for Set<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let LabelGroup {
            vis,
            krate,
            ident,
            generics,
            fields,
            set_ident,
        } = self.0;

        let mut sorted_fields = fields.clone();
        sorted_fields.sort_by_key(|x| x.attrs.get_sort_key());

        let (_, ty_generics, _) = generics.split_for_impl();

        let set_fields = sorted_fields.iter().map(|x| {
            let LabelGroupField {
                vis,
                name,
                attrs,
                ty,
                ..
            } = x;
            match &attrs.kind {
                LabelGroupFieldAttrsKind::Fixed => {
                    quote_spanned!( x.span => #vis #name: #krate::label::StaticLabelSet<#ty>, )
                }
                LabelGroupFieldAttrsKind::FixedWith(ty) => {
                    quote_spanned!( x.span => #vis #name: #ty, )
                }
                LabelGroupFieldAttrsKind::DynamicWith(ty) => {
                    quote_spanned!( x.span => #vis #name: #ty, )
                }
            }
        });

        let part = sorted_fields
            .partition_point(|x| x.attrs.get_sort_key() == LabelGroupFieldAttrsSortKey::Dynamic);
        let (dynamics, fixed) = sorted_fields.split_at(part);

        // this is to reverse the order of the encoded fields
        let mut fixed = fixed.to_vec();
        fixed.reverse();
        let fixed = &fixed;

        let cardinalities: Vec<TokenStream> = fixed
            .iter()
            .map(|x| {
                let LabelGroupField {
                    name, attrs, ty, ..
                } = x;
                match &attrs.kind {
                    LabelGroupFieldAttrsKind::Fixed => quote_spanned!( x.span => <#krate::label::StaticLabelSet<#ty> as #krate::label::FixedCardinalitySet>::cardinality(&self.#name)),
                    LabelGroupFieldAttrsKind::FixedWith(ty) => quote_spanned!( x.span => <#ty as #krate::label::FixedCardinalitySet>::cardinality(&self.#name)),
                    LabelGroupFieldAttrsKind::DynamicWith(_) => unreachable!(),
                }
            })
            .collect();

        let defaults = sorted_fields.iter().map(|x| {
            let name = &x.name;
            if x.attrs.default {
                match &x.attrs.kind {
                    LabelGroupFieldAttrsKind::Fixed => {
                        quote_spanned!(x.span => #name: #krate::label::StaticLabelSet::new(),)
                    }
                    LabelGroupFieldAttrsKind::FixedWith(path)
                    | LabelGroupFieldAttrsKind::DynamicWith(path) => {
                        quote_spanned!(x.span => #name: <#path as ::core::default::Default>::default(),)
                    }
                }
            } else {
                quote_spanned!(x.span => #name,)
            }
        });
        let default = if sorted_fields.iter().all(|x| x.attrs.default) {
            quote! {
                impl #set_ident {
                    pub fn new() -> Self {
                        Self {
                            #(#defaults)*
                        }
                    }
                }
                impl ::core::default::Default for #set_ident {
                    fn default() -> Self {
                        Self::new()
                    }
                }
            }
        } else {
            let args = sorted_fields.iter().filter(|x| !x.attrs.default).map(|x| {
                let name = &x.name;
                match &x.attrs.kind {
                    LabelGroupFieldAttrsKind::Fixed => unreachable!("fixed is always default"),
                    LabelGroupFieldAttrsKind::FixedWith(path)
                    | LabelGroupFieldAttrsKind::DynamicWith(path) => {
                        quote_spanned!(x.span => #name: #path)
                    }
                }
            });
            quote! {
                impl #set_ident {
                    pub fn new(#(#args),*) -> Self {
                        Self {
                            #(#defaults)*
                        }
                    }
                }
            }
        };

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

        let encode_fn = SetEncode {
            group: self.0,
            fixed,
            dynamics,
            cardinalities: &cardinalities,
        };

        let decode_fn = SetDecode {
            group: self.0,
            fixed,
            dynamics,
            cardinalities: &cardinalities,
        };

        tokens.extend(quote! {
            #vis struct #set_ident {
                #(#set_fields)*
            }

            #default

            #[automatically_derived]
            impl #krate::label::LabelGroupSet for #set_ident {
                // TODO: fix these generics
                type Group<'a> = #ident #ty_generics;

                #cardinality_fns

                #encode_fn

                #decode_fn
            }
        });
    }
}

struct SetEncode<'a> {
    group: &'a LabelGroup,
    fixed: &'a [LabelGroupField],
    dynamics: &'a [LabelGroupField],
    cardinalities: &'a [TokenStream],
}

impl ToTokens for SetEncode<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            group: LabelGroup { krate, .. },
            fixed,
            dynamics,
            cardinalities,
        } = self;

        let fixed_encodes: Vec<TokenStream> = fixed
            .iter()
            .map(|x| {
                let LabelGroupField {
                    name, attrs, ty, ..
                } = x;

                match &attrs.kind {
                    LabelGroupFieldAttrsKind::Fixed => {
                        quote_spanned!(x.span => <#krate::label::StaticLabelSet<#ty> as #krate::label::LabelSet>::encode(&self.#name, value.#name)?)
                    }
                    LabelGroupFieldAttrsKind::FixedWith(ty) => {
                        quote_spanned!(x.span => <#ty as #krate::label::LabelSet>::encode(&self.#name, value.#name)?)
                    }
                    LabelGroupFieldAttrsKind::DynamicWith(_) => unreachable!(),
                }
            })
            .collect();

        let dynamic_encodes: Vec<TokenStream> = dynamics
            .iter()
            .map(|x| {
                let LabelGroupField { name, attrs, .. } = x;

                match &attrs.kind {
                    LabelGroupFieldAttrsKind::DynamicWith(ty) => {
                        quote_spanned!(x.span => {
                            <#ty as #krate::label::DynamicLabelSet>::__private_check_dynamic();
                            <#ty as #krate::label::LabelSet>::encode(&self.#name, value.#name)?
                        })
                    }
                    _ => unreachable!(),
                }
            })
            .collect();

        tokens.extend(quote! {
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
        });
    }
}

struct SetDecode<'a> {
    group: &'a LabelGroup,
    fixed: &'a [LabelGroupField],
    dynamics: &'a [LabelGroupField],
    cardinalities: &'a [TokenStream],
}

impl ToTokens for SetDecode<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            group: LabelGroup { krate, .. },
            fixed,
            dynamics,
            cardinalities,
        } = *self;

        let fixed_decodes: Vec<TokenStream> = fixed
            .iter()
            .map(|x| {
                let LabelGroupField {
                    name, attrs, ty, ..
                } = x;

                match &attrs.kind {
                    LabelGroupFieldAttrsKind::Fixed => quote_spanned!(x.span => let #name = <#krate::label::StaticLabelSet<#ty> as #krate::label::LabelSet>::decode(&self.#name, index1);),
                    LabelGroupFieldAttrsKind::FixedWith(ty) => quote_spanned!(x.span => let #name = <#ty as #krate::label::LabelSet>::decode(&self.#name, index1);),
                    LabelGroupFieldAttrsKind::DynamicWith(_) =>unreachable!(),
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
                match &attrs.kind {
                    LabelGroupFieldAttrsKind::DynamicWith(ty) => quote_spanned!(x.span => let #name = <#ty as #krate::label::LabelSet>::decode(&self.#name, #index);),
                    _ => unreachable!(),
                }
            })
            .collect();

        let field_names = fixed.iter().chain(dynamics).map(|x| &x.name);

        tokens.extend(quote! {
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
        });
    }
}
