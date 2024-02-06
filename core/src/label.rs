//! Traits and types used for representing groups of label-pairs

use core::hash::Hash;

mod impls;
pub use impls::ComposedGroup;

pub enum InvalidMetricName {
    InvalidChars,
    Empty,
    StartsWithNumber,
}

#[repr(transparent)]
pub struct LabelName(str);

impl LabelName {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub const fn from_static(value: &'static str) -> &'static Self {
        // > Metric names may contain ASCII letters, digits, underscores, and colons. It must match the regex [a-zA-Z_:][a-zA-Z0-9_:]*
        if value.is_empty() {
            panic!("string should not be empty")
        }

        let mut i = 0;
        while i < value.len() {
            match value.as_bytes()[i] {
                b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'_' | b':' => {}
                _ => panic!("string should only contain [a-zA-Z0-9_:]"),
            }
            i += 1;
        }

        if value.as_bytes()[0].is_ascii_digit() {
            panic!("string should not start with a digit")
        }

        // SAFETY: `LabelName` is transparent over `str`. There's no way to do this safely.
        // I could use bytemuck::TransparentWrapper, but the trait enabled users to skip this validation function.
        unsafe { &*(value as *const str as *const LabelName) }
    }
}

// impl<'a> TryFrom<&'a str> for &'a LabelName {
//     type Error = InvalidMetricName;

//     fn try_from(value: &'a str) -> Result<Self, Self::Error> {
//         // > Metric names may contain ASCII letters, digits, underscores, and colons. It must match the regex [a-zA-Z_:][a-zA-Z0-9_:]*
//         if value.is_empty() {
//             return Err(InvalidMetricName::Empty);
//         }

//         value.bytes().try_fold((), |(), b| match b {
//             b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'_' | b':' => Ok(()),
//             _ => Err(InvalidMetricName::InvalidChars),
//         })?;

//         if value.as_bytes()[0].is_ascii_digit() {
//             return Err(InvalidMetricName::StartsWithNumber);
//         }

//         // SAFETY: `LabelName` is transparent over `str`. There's no way to do this safely.
//         // I could use bytemuck::TransparentWrapper, but the trait enabled users to skip this validation function.
//         Ok(unsafe { &*(value as *const str as *const LabelName) })
//     }
// }

/// A trait for visiting the value of a label
pub trait LabelVisitor {
    fn write_int(&mut self, x: u64);
    fn write_float(&mut self, x: f64);
    fn write_str(&mut self, x: &str);
}

/// `LabelGroup` represents a group of label-pairs
pub trait LabelGroup {
    /// Get all the label names in order
    fn label_names() -> impl IntoIterator<Item = &'static LabelName>;

    /// Writes all the label values into the visitor in the same order as the names
    fn label_values(&self, v: &mut impl LabelVisitor);

    /// Borrow this label group
    fn by_ref(&self) -> &Self {
        self
    }

    /// Combine this group with another
    fn compose_with<T: LabelGroup>(self, other: T) -> ComposedGroup<Self, T>
    where
        Self: Sized,
    {
        ComposedGroup(self, other)
    }
}

/// A [`LabelGroup`] with no label pairs
pub struct NoLabels;

impl LabelGroup for NoLabels {
    fn label_names() -> impl IntoIterator<Item = &'static LabelName> {
        core::iter::empty()
    }

    fn label_values(&self, _v: &mut impl LabelVisitor) {}
}

/// `LabelGroupSet` is a helper for [`LabelGroup`]s.
///
/// The `LabelGroup` pairs might need some extra data in order to encode/decode the values into their
/// compressed integer form.
pub trait LabelGroupSet {
    type Group<'a>: LabelGroup;

    /// The number of possible label-pairs the associated group can represent
    fn cardinality(&self) -> Option<usize>;

    /// If the label set is fixed in cardinality, it must return a value here in the range of `0..cardinality`
    fn encode_dense(&self, _value: Self::Unique) -> Option<usize>;
    /// If the label set is fixed in cardinality, a value in the range of `0..cardinality` should decode without panicking.
    fn decode_dense(&self, value: usize) -> Self::Group<'_>;

    /// A type that can uniquely represent all possible labels
    type Unique: Copy + Hash + Eq;

    /// Encode the label groups into the unique compressed representation
    fn encode(&self, value: Self::Group<'_>) -> Option<Self::Unique>;
    /// Decodes the compressed representation into the label values
    fn decode(&self, value: &Self::Unique) -> Self::Group<'_>;
}

/// A type that contains a label value
pub trait LabelValue {
    fn visit(&self, v: &mut impl LabelVisitor);
}

/// `FixedCardinalityLabel` represents a label value with a value<-> integer encoding known at compile time.
///
/// This is usually implemented by enums with the [`FixedCardinalityLabel`](crate::FixedCardinalityLabel) derive macro
pub trait FixedCardinalityLabel: LabelValue {
    /// The number of possible label values
    fn cardinality() -> usize;

    fn encode(&self) -> usize;
    fn decode(value: usize) -> Self;
}

/// `FixedCardinalityDynamicLabel` is a helper trait to represent a dynamic label with a fixed size.
///
/// An example of a dynamic label that has a fixed capacity is an API path with parameters removed
/// * `/api/v1/users`
/// * `/api/v1/users/:id`
/// * `/api/v1/products`
/// * `/api/v1/products/:id`
/// * `/api/v1/products/:id/owner`
/// * `/api/v1/products/:id/purchase`
///
/// These values can be awkward to set up as an enum for a compile time metric, but might be easier to build
/// as a runtime quantity.
///
/// Additionally, sometimes the set of label values can only be known based on some startup configuration, but never changes.
pub trait FixedCardinalityDynamicLabel {
    type Value<'a>: LabelValue;

    /// The number of possible label values
    fn cardinality(&self) -> usize;

    /// Find the integer that represents this value, if any
    fn encode(&self, value: Self::Value<'_>) -> Option<usize>;

    /// Extract the value uniquely represented by this integer
    fn decode(&self, value: usize) -> Self::Value<'_>;
}

/// `DynamicLabel` is a helper trait to represent a dynamic label with an unknown collection of values.
///
/// This is not recommended to be used, but provided for completeness sake.
/// [Prometheus recommends against high-cardinality metrics](https://grafana.com/blog/2022/02/15/what-are-cardinality-spikes-and-why-do-they-matter/)
/// but there might be cases where you still want to use this
///
/// 1. Compatibility with your existing setup
/// 2. Not exporting to prometheus
/// 3. You know there wont be many labels but you just don't know what they are
pub trait DynamicLabel {
    type Value<'a>: LabelValue;

    fn encode(&self, value: Self::Value<'_>) -> Option<usize>;
    fn decode(&self, value: usize) -> Self::Value<'_>;
}

#[cfg(test)]
mod tests {
    use fake::{faker::name::raw::Name, locales::EN, Fake};
    use lasso::{Rodeo, RodeoReader, ThreadedRodeo};

    use super::LabelGroupSet;

    #[derive(Clone, Copy, PartialEq, Debug, measured_derive::LabelGroup)]
    #[label(crate = crate, set = ErrorsSet)]
    struct Error<'a> {
        #[label(fixed)]
        kind: ErrorKind,
        #[label(fixed_with = RodeoReader)]
        route: &'a str,
    }

    #[derive(Clone, Copy, PartialEq, Debug, measured_derive::FixedCardinalityLabel)]
    #[label(crate = crate, rename_all = "kebab-case")]
    enum ErrorKind {
        User,
        Internal,
        Network,
    }

    #[test]
    fn encoding_happy_path() {
        let mut rodeo = Rodeo::new();

        rodeo.get_or_intern("/user/:id/home");
        rodeo.get_or_intern("/playlist/:id");
        rodeo.get_or_intern("/user/:id/subscribe");
        rodeo.get_or_intern("/user/:id/videos");

        let set = ErrorsSet {
            route: rodeo.into_reader(),
        };
        assert_eq!(set.cardinality(), Some(12));

        let error_kinds = [ErrorKind::User, ErrorKind::Internal, ErrorKind::Network];
        for route in set.route.strings() {
            for kind in error_kinds {
                let error = Error { kind, route };
                let index: usize = set.encode(error).unwrap();
                let error2 = set.decode(&index);
                assert_eq!(error, error2);
            }
        }
    }

    #[derive(Clone, Copy, PartialEq, Debug, measured_derive::LabelGroup)]
    #[label(crate = crate, set = ErrorsSet2)]
    struct Error2<'a> {
        #[label(fixed)]
        kind: ErrorKind,
        #[label(fixed_with = RodeoReader)]
        route: &'a str,
        #[label(dynamic_with = ThreadedRodeo)]
        user: &'a str,
    }

    #[test]
    fn dynamic_labels() {
        let mut rodeo = Rodeo::new();

        rodeo.get_or_intern("/user/:id/home");
        rodeo.get_or_intern("/playlist/:id");
        rodeo.get_or_intern("/user/:id/subscribe");
        rodeo.get_or_intern("/user/:id/videos");

        let set = ErrorsSet2 {
            route: rodeo.into_reader(),
            user: ThreadedRodeo::new(),
        };
        assert_eq!(set.cardinality(), None);

        let error_kinds = [ErrorKind::User, ErrorKind::Internal, ErrorKind::Network];
        for route in set.route.strings() {
            for kind in error_kinds {
                for _ in 0..8 {
                    let error = Error2 {
                        kind,
                        route,
                        user: &Name(EN).fake::<String>(),
                    };
                    let index: (usize, usize) = set.encode(error).unwrap();
                    let error2 = set.decode(&index);
                    assert_eq!(error, error2);
                }
            }
        }
    }
}
