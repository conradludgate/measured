use std::hash::Hash;

mod impls;
pub use impls::ComposedGroup;

pub trait LabelVisitor {
    fn write_int(&mut self, x: u64);
    fn write_float(&mut self, x: f64);
    fn write_str(&mut self, x: &str);
}

pub trait LabelGroup {
    /// Get all the label names in order
    fn label_names() -> impl IntoIterator<Item = &'static str>;

    fn label_values(&self, v: &mut impl LabelVisitor);

    fn by_ref(&self) -> &Self {
        self
    }
    fn compose_with<T: LabelGroup>(self, other: T) -> ComposedGroup<Self, T>
    where
        Self: Sized,
    {
        ComposedGroup(self, other)
    }
}

pub struct NoLabels;

impl LabelGroup for NoLabels {
    fn label_names() -> impl IntoIterator<Item = &'static str> {
        std::iter::empty()
    }

    fn label_values(&self, _v: &mut impl LabelVisitor) {}
}

pub trait LabelGroupSet {
    type Group<'a>: LabelGroup
    where
        Self: 'a;

    /// The number of possible labels
    fn cardinality(&self) -> Option<usize>;

    /// If the label set is fixed in cardinality, it must return a value here in the range of
    /// 0..cardinality
    fn encode_dense(&self, _value: Self::Unique) -> Option<usize>;
    fn decode_dense(&self, value: usize) -> Self::Group<'_>;

    /// A type that can uniquely represent all possible labels
    type Unique: Copy + Hash + Eq;

    fn encode(&self, value: Self::Group<'_>) -> Option<Self::Unique>;
    fn decode(&self, value: &Self::Unique) -> Self::Group<'_>;
}

pub trait LabelValue {
    fn visit(&self, v: &mut impl LabelVisitor);
}

pub trait FixedCardinalityLabel: LabelValue {
    /// The number of possible label values
    fn cardinality() -> usize;

    fn encode(&self) -> usize;
    fn decode(value: usize) -> Self;
}

pub trait FixedCardinalityDynamicLabel {
    type Value<'a>: LabelValue
    where
        Self: 'a;

    /// The number of possible label values
    fn cardinality(&self) -> usize;
    fn encode<'a>(&'a self, value: Self::Value<'a>) -> Option<usize>;
    fn decode(&self, value: usize) -> Self::Value<'_>;
}

pub trait DynamicLabel {
    type Value<'a>: LabelValue
    where
        Self: 'a;

    fn encode<'a>(&'a self, value: Self::Value<'a>) -> Option<usize>;
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
