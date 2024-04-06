use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{parse_quote, parse_quote_spanned};

use super::{
    attr::{MetricGroupFieldAttrsInit, MetricGroupFieldAttrsKind},
    MetricGroup, MetricGroupField,
};

impl ToTokens for MetricGroup {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            krate,
            ident,
            generics,
            fields,
            inputs,
        } = self;

        let enc = format_ident!("__MetricGroupEncodingT");

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let mut group_generics = generics.clone();
        group_generics
            .params
            .push(parse_quote!(#enc: #krate::metric::group::Encoding));
        let wc = group_generics.make_where_clause();
        for field in fields {
            let MetricGroupField { ty, attrs, .. } = field;
            match attrs.kind {
                MetricGroupFieldAttrsKind::Metric { .. } => {
                    wc.predicates.push(parse_quote_spanned!(field.span => #ty: #krate::metric::MetricFamilyEncoding<#enc> ));
                }
                MetricGroupFieldAttrsKind::Group { namespace: None } => {
                    wc.predicates.push(parse_quote_spanned!(field.span => #ty: #krate::metric::group::MetricGroup<#enc> ));
                }
                MetricGroupFieldAttrsKind::Group { namespace: Some(_) } => {
                    wc.predicates.push(parse_quote_spanned!(field.span =>
                        #ty: for<'__enc_tmp_lt> #krate::metric::group::MetricGroup<
                            #krate::metric::name::WithNamespace<&'__enc_tmp_lt mut #enc>,
                        >
                    ));
                }
            }
        }

        let (group_impl_generics, _, group_where_clause) = group_generics.split_for_impl();

        let visits = fields.iter().map(|x| {
            let MetricGroupField { name,ty, attrs, .. } = x;
            match &attrs.kind {
                MetricGroupFieldAttrsKind::Metric { rename } => {
                    let name_string = rename.as_ref().map_or_else(|| name.to_string(), |l| l.value());
                    let ident = format_ident!("{}", name_string.to_shouty_snake_case(), span = x.span);

                    let help = attrs.docs.as_deref().map(|doc|{
                        let doc = doc.trim();
                        quote_spanned!(x.span => {
                            <#enc as #krate::metric::group::Encoding>::write_help(enc, #ident, #doc)?;
                        })
                    });

                    quote_spanned! { x.span =>
                        const #ident: &#krate::metric::name::MetricName = #krate::metric::name::MetricName::from_str(#name_string);
                        #help
                        <#ty as #krate::metric::MetricFamilyEncoding<#enc>>::collect_family_into(&self.#name, #ident, enc)?;
                    }
                },
                MetricGroupFieldAttrsKind::Group { namespace: None } => {
                    quote_spanned! { x.span =>
                        <#ty as #krate::metric::group::MetricGroup<#enc>>::collect_group_into(&self.#name, enc)?;
                    }
                },
                MetricGroupFieldAttrsKind::Group { namespace: Some(ns) } => {
                    quote_spanned! { x.span =>
                        <#krate::metric::name::WithNamespace<&#ty> as #krate::metric::group::MetricGroup<#enc>>::collect_group_into(
                            &#krate::metric::name::WithNamespace::new(#ns, &self.#name),
                            enc,
                        )?;
                    }
                },
            }
        });

        tokens.extend(quote! {
            #[automatically_derived]
            impl #group_impl_generics #krate::metric::group::MetricGroup<#enc> for #ident #ty_generics #group_where_clause {
                fn collect_group_into(&self, enc: &mut #enc) -> Result<(), #enc::Err>{
                    #(#visits)*
                    Ok(())
                }
            }
        });

        if let Some(inputs) = inputs {
            let inits = fields.iter().map(|x| {
                let MetricGroupField { name,ty, attrs, .. } = x;
                match &attrs.init {
                    Some(MetricGroupFieldAttrsInit::Raw(init)) => quote_spanned!(x.span => #name: #init,),
                    Some(MetricGroupFieldAttrsInit::Metric { metadata, label_set }) => {
                        let default: syn::Expr = parse_quote!{::core::default::Default::default()};
                        let metadata = metadata.as_ref().unwrap_or(&default);
                        if let Some(ls) = label_set {
                            quote_spanned!(x.span => #name: <#ty>::with_label_set_and_metadata(#ls, #metadata),)
                        } else {
                            quote_spanned!(x.span => #name: <#ty>::with_metadata(#label_set, #metadata),)
                        }
                    }
                    None => quote_spanned!(x.span => #name: <#ty as ::core::default::Default>::default(),),
                }
            });

            tokens.extend(quote! {
                impl #impl_generics #ident #ty_generics #where_clause {
                    pub fn new(#inputs) -> Self {
                        Self {
                            #(#inits)*
                        }
                    }
                }
            });
        }
    }
}
