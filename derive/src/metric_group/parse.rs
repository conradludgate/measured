use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Field, Fields};

use super::attr::{ContainerAttrs, MetricGroupFieldAttrs};
use super::{MetricGroup, MetricGroupField};
use crate::Krate;

impl TryFrom<Field> for MetricGroupField {
    type Error = syn::Error;
    fn try_from(input: Field) -> syn::Result<Self> {
        let span = input.span();
        let attrs = MetricGroupFieldAttrs::parse_attrs(&input.attrs)?;
        let name = input.ident.unwrap();

        if let Some(unit) = &attrs.unit {
            let metric_name = match &attrs.kind {
                super::attr::MetricGroupFieldAttrsKind::Metric { rename } => rename
                    .as_ref()
                    .map_or_else(|| name.unraw().to_string(), syn::LitStr::value),
                super::attr::MetricGroupFieldAttrsKind::Group { .. } => {
                    return Err(syn::Error::new(
                        unit.span(),
                        "`metric(unit)` can only be used on metric fields",
                    ));
                }
            };
            let unit_value = unit.value();
            let valid_unit = !unit_value.is_empty()
                && unit_value
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b':');
            let family_name = metric_name.strip_suffix("_total").unwrap_or(&metric_name);
            if !valid_unit || !family_name.ends_with(&format!("_{unit_value}")) {
                return Err(syn::Error::new(
                    unit.span(),
                    format!(
                        "metric name `{metric_name}` must end in `_{unit_value}` (before a `_total` suffix) when `metric(unit)` is set"
                    ),
                ));
            }
        }

        Ok(MetricGroupField { span, name, ty: input.ty, attrs })
    }
}

impl TryFrom<DeriveInput> for MetricGroup {
    type Error = syn::Error;
    fn try_from(input: DeriveInput) -> syn::Result<Self> {
        let span = input.span();
        let DeriveInput { ident, data, generics, attrs, .. } = input;

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
