use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{parse_quote, parse_quote_spanned};

use super::{attr::MetricGroupFieldAttrs, MetricGroup, MetricGroupField};

impl ToTokens for MetricGroup {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            krate,
            ident,
            generics,
            fields,
            ..
        } = self;

        let enc = format_ident!("__MetricGroupEncodingT");

        let (_, ty_generics, _) = generics.split_for_impl();

        let mut generics = generics.clone();
        generics
            .params
            .push(parse_quote!(#enc: #krate::metric::group::Encoding));
        let wc = generics.make_where_clause();
        for field in fields {
            let MetricGroupField { ty, attrs, .. } = field;
            match attrs {
                MetricGroupFieldAttrs::Metric { .. } => {
                    wc.predicates.push(parse_quote_spanned!(field.span => #ty: #krate::metric::MetricFamilyEncoding<#enc> ));
                }
                MetricGroupFieldAttrs::Group { namespace: None } => {
                    wc.predicates.push(parse_quote_spanned!(field.span => #ty: #krate::metric::group::MetricGroup<#enc> ));
                }
                MetricGroupFieldAttrs::Group { namespace: Some(_) } => {
                    wc.predicates.push(parse_quote_spanned!(field.span =>
                        #ty: for<'__enc_tmp_lt> #krate::metric::group::MetricGroup<
                            #krate::metric::name::WithNamespace<&'__enc_tmp_lt mut #enc>,
                        >
                    ));
                }
            }
        }

        let (impl_generics, _, where_clause) = generics.split_for_impl();

        let visits = fields.iter().map(|x| {
            let MetricGroupField { name,ty, attrs, .. } = x;
            match attrs {
                MetricGroupFieldAttrs::Metric { rename } => {
                    let name_string = rename.as_ref().map_or_else(|| name.to_string(), |l| l.value());
                    let ident = format_ident!("{}", name_string.to_shouty_snake_case(), span = x.span);
                    quote_spanned! { x.span =>
                        const #ident: &#krate::metric::name::MetricName = #krate::metric::name::MetricName::from_static(#name_string);
                        // enc.write_help(ERRORS, "help text");
                        <#ty as #krate::metric::MetricFamilyEncoding<#enc>>::collect_into(&self.#name, #ident, enc);
                    }
                },
                MetricGroupFieldAttrs::Group { namespace: None } => {
                    quote_spanned! { x.span =>
                        <#ty as #krate::metric::group::MetricGroup<#enc>>::collect_into(&self.#name, enc);
                    }
                },
                MetricGroupFieldAttrs::Group { namespace: Some(ns) } => {
                    quote_spanned! { x.span =>
                        <#krate::metric::name::WithNamespace<&#ty> as #krate::metric::group::MetricGroup<#enc>>::collect_into(
                            &#krate::metric::name::WithNamespace::new(#ns, &self.#name),
                            enc,
                        );
                    }
                },
            }
        });

        tokens.extend(quote! {
            #[automatically_derived]
            impl #impl_generics #krate::metric::group::MetricGroup<#enc> for #ident #ty_generics #where_clause {
                fn collect_into(&self, enc: &mut #enc) {
                    #(#visits)*
                }
            }
        });
    }
}
