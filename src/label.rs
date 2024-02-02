use std::hash::Hash;

mod impls;

trait LabelVisitor {
    fn write_int(&mut self, x: u64);
    fn write_float(&mut self, x: f64);
    fn write_str(&mut self, x: &str);
}

trait LabelGroup {
    /// Get all the label names in order
    fn label_names() -> impl IntoIterator<Item = &'static str>;

    fn label_values(self, v: &mut impl LabelVisitor);
}

trait LabelGroupSet {
    type Group<'a>: LabelGroup
    where
        Self: 'a;

    /// The number of possible labels
    fn cardinality(&self) -> Option<usize>;

    /// A type that can uniquely represent all possible labels
    type Unique: Hash + Eq;

    fn encode<'a>(&'a self, value: Self::Group<'a>) -> Self::Unique;
    fn decode(&self, value: Self::Unique) -> Self::Group<'_>;
}

trait LabelValue {
    fn visit(&self, v: &mut impl LabelVisitor);
}

trait FixedCardinalityLabel: LabelValue {
    /// The number of possible label values
    fn cardinality() -> usize;

    fn encode(&self) -> usize;
    fn decode(value: usize) -> Self;
}

trait FixedCardinalityDynamicLabel {
    type Value<'a>: LabelValue
    where
        Self: 'a;

    /// The number of possible label values
    fn cardinality(&self) -> usize;
    fn encode<'a>(&'a self, value: Self::Value<'a>) -> usize;
    fn decode(&self, value: usize) -> Self::Value<'_>;
}

#[cfg(test)]
mod tests {
    use lasso::{Rodeo, RodeoReader};

    use super::{
        FixedCardinalityDynamicLabel, FixedCardinalityLabel, LabelGroup, LabelGroupSet, LabelValue,
    };

    #[derive(Clone, Copy, PartialEq, Debug)]
    // #[derive(LabelGroup)] #[label_set(ErrorsSet)]
    struct Error<'a> {
        // #[label(fixed)]
        kind: ErrorKind,
        // #[label(fixed_with = IndexSet<&'static str>)]
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
                let index = set.encode(error);
                let error2 = set.decode(index);
                assert_eq!(error, error2);
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

        type Unique = (usize,);

        #[allow(unused_assignments)]
        fn encode<'a>(&'a self, value: Self::Group<'a>) -> Self::Unique {
            let mut mul = 1;
            let mut index = 0;

            index += value.kind.encode() * mul;
            mul *= ErrorKind::cardinality();

            index += self.routes.encode(value.route) * mul;
            mul *= self.routes.cardinality();

            (index,)
        }

        fn decode(&self, value: Self::Unique) -> Self::Group<'_> {
            let (index,) = value;
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
}
