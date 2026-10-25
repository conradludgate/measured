use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Field, Fields};

use super::attr::{ContainerAttrs, MetricGroupFieldAttrs};
use super::{MetricGroup, MetricGroupField};
use crate::Krate;

impl TryFrom<Field> for MetricGroupField {
    type Error = syn::Error;
    fn try_from(input: Field) -> syn::Result<Self> {
        let attrs = MetricGroupFieldAttrs::parse_attrs(&input.attrs)?;
        Ok(MetricGroupField {
            span: input.span(),
            name: input.ident.unwrap(),
            ty: input.ty,
            attrs,
        })
    }
}

impl TryFrom<DeriveInput> for MetricGroup {
    type Error = syn::Error;
    fn try_from(input: DeriveInput) -> syn::Result<Self> {
        let span = input.span();
        let DeriveInput {
            ident,
            data,
            generics,
            attrs,
            ..
        } = input;

        let args = ContainerAttrs::parse_attrs(&attrs)?;
        let Krate(krate) = args.krate.unwrap_or_default();

        let fields = match data {
            Data::Enum(_) => return Err(syn::Error::new(span, "enums not supported")),
            Data::Union(_) => return Err(syn::Error::new(span, "unions not supported")),
            Data::Struct(s) => match s.fields {
                Fields::Named(n) => n
                    .named
                    .into_iter()
                    .map(MetricGroupField::try_from)
                    .collect::<Result<Vec<_>, syn::Error>>()?,
                Fields::Unnamed(_) => {
                    return Err(syn::Error::new(span, "tuple structs not supported"));
                }
                Fields::Unit => return Err(syn::Error::new(span, "unit structs not supported")),
            },
        };

        if args.inputs.is_none()
            && let Some(field) = fields.iter().find(|f| f.attrs.init.is_some())
        {
            return Err(syn::Error::new(
                field.span,
                "`metric(init)`, `metric(metadata)` and `metric(label_set)` require `#[metric(new(...))]` on the struct",
            ));
        }

        Ok(Self {
            krate,
            ident,
            fields,
            generics,
            inputs: args.inputs,
        })
    }
}
