use syn::{spanned::Spanned, Data, DeriveInput, Fields, Lit, Variant};

use crate::Krate;

use super::{
    attr::{ContainerAttrs, VariantAttrs},
    FixedCardinalityLabel, FixedCardinalityLabelVariant,
};

impl TryFrom<Variant> for FixedCardinalityLabelVariant {
    type Error = syn::Error;
    fn try_from(input: Variant) -> syn::Result<Self> {
        let span = input.span();
        let attrs = VariantAttrs::parse_attrs(&input.attrs)?;

        match input.fields {
            Fields::Named(_) | Fields::Unnamed(_) => {
                return Err(syn::Error::new(span, "variants with values not supported"))
            }
            Fields::Unit => {}
        }

        Ok(FixedCardinalityLabelVariant {
            span,
            attrs,
            ident: input.ident,
            value: input
                .discriminant
                .map(|(_, expr)| match expr {
                    syn::Expr::Lit(expr_lit) => match expr_lit.lit {
                        Lit::Int(int) => Ok(int),
                        _ => Err(syn::Error::new(expr_lit.span(), "unsupported discriminant")),
                    },
                    _ => Err(syn::Error::new(expr.span(), "unsupported discriminant")),
                })
                .transpose()?,
        })
    }
}

impl TryFrom<DeriveInput> for FixedCardinalityLabel {
    type Error = syn::Error;
    fn try_from(input: DeriveInput) -> syn::Result<Self> {
        let span = input.span();
        let DeriveInput {
            ident, data, attrs, ..
        } = input;

        let args = ContainerAttrs::parse_attrs(&attrs)?;
        let Krate(krate) = args.krate.unwrap_or_default();

        // <https://prometheus.io/docs/instrumenting/writing_exporters/#naming>
        // > Prometheus metrics and label names are written in snake_case. Converting camelCase to snake_case is desirable
        let rename_all = args.rename_all.unwrap_or(super::attr::RenameAll::Snake);

        let variants = match data {
            Data::Enum(e) => e
                .variants
                .into_iter()
                .map(FixedCardinalityLabelVariant::try_from)
                .collect::<Result<Vec<_>, syn::Error>>()?,
            Data::Union(_) => return Err(syn::Error::new(span, "unions not supported")),
            Data::Struct(_) => return Err(syn::Error::new(span, "structs not supported")),
        };

        Ok(Self {
            krate,
            rename_all,
            ident,
            variants,
            singleton: args.singleton,
        })
    }
}
