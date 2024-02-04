use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::group::LabelGroupField;

use super::attr::{LabelGroupFieldAttrs, LabelGroupFieldAttrsSortKey};
use super::LabelGroup;

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
