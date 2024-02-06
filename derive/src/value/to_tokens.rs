use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};

use super::FixedCardinalityLabel;

impl ToTokens for FixedCardinalityLabel {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            krate,
            ident,
            rename_all,
            variants,
        } = self;

        let cardinality = variants.len();
        let var_idents1 = variants.iter().map(|x| &x.ident);
        let var_idents2 = variants.iter().map(|x| &x.ident);
        let count1 = 0..cardinality;
        let count2 = 0..cardinality;

        let visits = variants.iter().map(|var| {
            let var_ident = &var.ident;
            let write = if let Some(int) = &var.value {
                quote_spanned!(int.span() => v.write_int(#int))
            } else {
                let name = var.attrs.rename.as_ref().map_or_else(
                    || rename_all.apply(&var.ident.to_string()),
                    syn::LitStr::value,
                );
                quote_spanned!(var.span => v.write_str(#name))
            };
            quote_spanned!(var.span => #ident :: #var_ident => #write,)
        });

        tokens.extend(quote! {
            #[automatically_derived]
            impl #krate::label::FixedCardinalityLabel for #ident {
                fn cardinality() -> usize {
                    #cardinality
                }

                fn encode(&self) -> usize {
                    match self {
                        #(#ident :: #var_idents1 => #count1,)*
                    }
                }

                fn decode(value: usize) -> Self {
                    match value {
                        #(#count2 => #ident :: #var_idents2,)*
                        _ => panic!("invalid value"),
                    }
                }
            }

            #[automatically_derived]
            impl #krate::label::LabelValue for #ident {
                fn visit(&self, v: &mut impl #krate::label::LabelVisitor) {
                    match self {
                        #(#visits)*
                    }
                }
            }
        });
    }
}
