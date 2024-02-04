use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use super::FixedCardinalityLabel;

// // impl FixedCardinalityLabel for ErrorKind {
// //     fn cardinality() -> usize {
// //         3
// //     }

// //     fn encode(&self) -> usize {
// //         match self {
// //             ErrorKind::User => 0,
// //             ErrorKind::Internal => 1,
// //             ErrorKind::Network => 2,
// //         }
// //     }

// //     fn decode(value: usize) -> Self {
// //         match value {
// //             0 => ErrorKind::User,
// //             1 => ErrorKind::Internal,
// //             2 => ErrorKind::Network,
// //             _ => panic!("invalid value"),
// //         }
// //     }
// // }

// // impl LabelValue for ErrorKind {
// //     fn visit(&self, v: &mut impl super::LabelVisitor) {
// //         match self {
// //             ErrorKind::User => v.write_str("user"),
// //             ErrorKind::Internal => v.write_str("internal"),
// //             ErrorKind::Network => v.write_str("network"),
// //         }
// //     }
// // }

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
                quote!(v.write_int(#int))
            } else {
                let name = var.attrs.rename.as_ref().map_or_else(
                    || {
                        let default_name = var.ident.to_string();
                        if let Some(rename_all) = rename_all {
                            rename_all.apply(&default_name)
                        } else {
                            default_name
                        }
                    },
                    |l| l.value(),
                );
                quote!(v.write_str(#name))
            };
            quote!(#ident :: #var_ident => #write,)
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
        })
    }
}
