use std::hash::Hash;

mod impls;

pub trait LabelVisitor {
    fn write_int(&mut self, x: u64);
    fn write_float(&mut self, x: f64);
    fn write_str(&mut self, x: &str);
}

pub trait LabelGroup {
    /// Get all the label names in order
    fn label_names() -> impl IntoIterator<Item = &'static str>;

    fn label_values(self, v: &mut impl LabelVisitor);
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

    /// A type that can uniquely represent all possible labels
    type Unique: Hash + Eq;

    fn encode<'a>(&'a self, value: Self::Group<'a>) -> Option<Self::Unique>;
    fn decode(&self, value: Self::Unique) -> Self::Group<'_>;
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

    use super::{
        DynamicLabel, FixedCardinalityDynamicLabel, FixedCardinalityLabel, LabelGroup,
        LabelGroupSet, LabelValue,
    };

    #[derive(Clone, Copy, PartialEq, Debug)]
    // #[derive(LabelGroup)] #[label_set(ErrorsSet)]
    struct Error<'a> {
        // #[label(fixed)]
        kind: ErrorKind,
        // #[label(fixed_with = RodeoReader)]
        route: &'a str,
    }

    struct ErrorsSet {
        routes: RodeoReader,
    }

    #[derive(Clone, Copy, PartialEq, Debug)]
    // #[derive(FixedCardinalityLabel)] #[label(rename_all = "kebab-case")]
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
            routes: rodeo.into_reader(),
        };
        assert_eq!(set.cardinality(), Some(12));

        let error_kinds = [ErrorKind::User, ErrorKind::Internal, ErrorKind::Network];
        for route in set.routes.strings() {
            for kind in error_kinds {
                let error = Error { kind, route };
                let index: usize = set.encode(error).unwrap();
                let error2 = set.decode(index);
                assert_eq!(error, error2);
            }
        }
    }

    #[derive(Clone, Copy, PartialEq, Debug)]
    // #[derive(LabelGroup)] #[label_set(ErrorsSet2)]
    struct Error2<'a> {
        // #[label(fixed)]
        kind: ErrorKind,
        // #[label(fixed_with = IndexSet<&'static str>)]
        route: &'a str,
        // #[label(dynamic_with = ThreadedRodeo)]
        user: &'a str,
    }

    struct ErrorsSet2 {
        routes: RodeoReader,
        users: ThreadedRodeo,
    }

    #[test]
    fn dynamic_labels() {
        let mut rodeo = Rodeo::new();

        rodeo.get_or_intern("/user/:id/home");
        rodeo.get_or_intern("/playlist/:id");
        rodeo.get_or_intern("/user/:id/subscribe");
        rodeo.get_or_intern("/user/:id/videos");

        let set = ErrorsSet2 {
            routes: rodeo.into_reader(),
            users: ThreadedRodeo::new(),
        };
        assert_eq!(set.cardinality(), None);

        let error_kinds = [ErrorKind::User, ErrorKind::Internal, ErrorKind::Network];
        for route in set.routes.strings() {
            for kind in error_kinds {
                for _ in 0..8 {
                    let error = Error2 {
                        kind,
                        route,
                        user: &Name(EN).fake::<String>(),
                    };
                    let index: (usize, usize) = set.encode(error).unwrap();
                    let error2 = set.decode(index);
                    assert_eq!(error, error2);
                }
            }
        }
    }

    // TODO: generate with macros

    impl LabelGroupSet for ErrorsSet {
        type Group<'a> = Error<'a>;

        fn cardinality(&self) -> Option<usize> {
            Some(1usize)
                .and_then(|x| x.checked_mul(ErrorKind::cardinality()))
                .and_then(|x| x.checked_mul(self.routes.cardinality()))
        }

        type Unique = usize;

        #[allow(unused_assignments)]
        fn encode<'a>(&'a self, value: Self::Group<'a>) -> Option<Self::Unique> {
            let mut mul = 1;
            let mut index = 0;

            index += value.kind.encode() * mul;
            mul *= ErrorKind::cardinality();

            index += self.routes.encode(value.route)? * mul;
            mul *= self.routes.cardinality();

            Some(index)
        }

        fn decode(&self, value: Self::Unique) -> Self::Group<'_> {
            let index = value;
            let (index, index1) = (
                index / ErrorKind::cardinality(),
                index % ErrorKind::cardinality(),
            );
            let kind = ErrorKind::decode(index1);
            let (index, index1) = (
                index / self.routes.cardinality(),
                index % self.routes.cardinality(),
            );
            let route = self.routes.decode(index1);
            debug_assert_eq!(index, 0);
            Self::Group { kind, route }
        }

        fn encode_dense(&self, value: Self::Unique) -> Option<usize> {
            Some(value)
        }
    }

    impl LabelGroup for Error<'_> {
        fn label_names() -> impl IntoIterator<Item = &'static str> {
            ["kind", "route"]
        }

        fn label_values(self, v: &mut impl super::LabelVisitor) {
            self.kind.visit(v);
            self.route.visit(v);
        }
    }

    impl FixedCardinalityLabel for ErrorKind {
        fn cardinality() -> usize {
            3
        }

        fn encode(&self) -> usize {
            match self {
                ErrorKind::User => 0,
                ErrorKind::Internal => 1,
                ErrorKind::Network => 2,
            }
        }

        fn decode(value: usize) -> Self {
            match value {
                0 => ErrorKind::User,
                1 => ErrorKind::Internal,
                2 => ErrorKind::Network,
                _ => panic!("invalid value"),
            }
        }
    }

    impl LabelValue for ErrorKind {
        fn visit(&self, v: &mut impl super::LabelVisitor) {
            match self {
                ErrorKind::User => v.write_str("user"),
                ErrorKind::Internal => v.write_str("internal"),
                ErrorKind::Network => v.write_str("network"),
            }
        }
    }

    impl LabelGroupSet for ErrorsSet2 {
        type Group<'a> = Error2<'a>;

        fn cardinality(&self) -> Option<usize> {
            None
        }

        fn encode_dense(&self, _value: Self::Unique) -> Option<usize> {
            None
        }

        type Unique = (usize, usize);

        #[allow(unused_assignments)]
        fn encode<'a>(&'a self, value: Self::Group<'a>) -> Option<Self::Unique> {
            let mut mul = 1;
            let mut index = 0;

            index += FixedCardinalityLabel::encode(&value.kind) * mul;
            mul *= ErrorKind::cardinality();

            index += FixedCardinalityDynamicLabel::encode(&self.routes, value.route)? * mul;
            mul *= self.routes.cardinality();

            let dynamic_index0 = DynamicLabel::encode(&self.users, value.user)?;

            Some((index, dynamic_index0))
        }

        fn decode(&self, value: Self::Unique) -> Self::Group<'_> {
            let (index, dynamic_index0) = value;
            let (index, index1) = (
                index / ErrorKind::cardinality(),
                index % ErrorKind::cardinality(),
            );
            let kind = <ErrorKind as FixedCardinalityLabel>::decode(index1);
            let (index, index1) = (
                index / self.routes.cardinality(),
                index % self.routes.cardinality(),
            );
            let route = FixedCardinalityDynamicLabel::decode(&self.routes, index1);
            debug_assert_eq!(index, 0);

            let user = DynamicLabel::decode(&self.users, dynamic_index0);

            Self::Group { kind, route, user }
        }
    }

    impl LabelGroup for Error2<'_> {
        fn label_names() -> impl IntoIterator<Item = &'static str> {
            ["kind", "route", "user"]
        }

        fn label_values(self, v: &mut impl super::LabelVisitor) {
            self.kind.visit(v);
            self.route.visit(v);
            self.user.visit(v);
        }
    }
}
