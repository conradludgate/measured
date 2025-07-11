use syn::{Data, DeriveInput, Field, Fields, spanned::Spanned};

use crate::Krate;

use super::attr::{ContainerAttrs, LabelGroupFieldAttrs};
use super::{LabelGroup, LabelGroupField};

impl TryFrom<Field> for LabelGroupField {
    type Error = syn::Error;
    fn try_from(input: Field) -> syn::Result<Self> {
        let attrs = LabelGroupFieldAttrs::parse_attrs(&input.attrs)?;
        Ok(LabelGroupField {
            span: input.span(),
            vis: input.vis,
            name: input.ident.unwrap(),
            ty: input.ty,
            attrs,
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

        let fields = match data {
            Data::Enum(_) => return Err(syn::Error::new(span, "enums not supported")),
            Data::Union(_) => return Err(syn::Error::new(span, "unions not supported")),
            Data::Struct(s) => match s.fields {
                Fields::Named(n) => n
                    .named
                    .into_iter()
                    .map(LabelGroupField::try_from)
                    .collect::<Result<Vec<_>, syn::Error>>()?,
                Fields::Unnamed(_) => {
                    return Err(syn::Error::new(span, "tuple structs not supported"));
                }
                Fields::Unit => return Err(syn::Error::new(span, "unit structs not supported")),
            },
        };

        Ok(Self {
            vis,
            krate,
            set_ident,
            ident,
            fields,
            generics,
        })
    }
}
