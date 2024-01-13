use std::hash::{BuildHasher, Hash};

use indexmap::IndexSet;

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
    type Group: LabelGroup;

    /// The number of possible labels
    fn cardinality(&self) -> Option<usize>;

    /// A type that can uniquely represent all possible labels
    type Unique: Hash + Eq;

    fn encode(&self, value: Self::Group) -> Self::Unique;
    fn decode(&self, value: Self::Unique) -> Self::Group;
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
    type Value: LabelValue;

    /// The number of possible label values
    fn cardinality(&self) -> usize;
    fn encode(&self, value: Self::Value) -> usize;
    fn decode(&self, value: usize) -> Self::Value;
}

impl<T: LabelValue + Hash + Eq + Clone, S: BuildHasher> FixedCardinalityDynamicLabel
    for IndexSet<T, S>
{
    type Value = T;

    fn cardinality(&self) -> usize {
        self.len()
    }

    fn encode(&self, value: Self::Value) -> usize {
        self.get_index_of(&value).unwrap()
    }

    fn decode(&self, value: usize) -> Self::Value {
        self.get_index(value).unwrap().clone()
    }
}

impl LabelValue for String {
    fn visit(&self, v: &mut impl crate::LabelVisitor) {
        v.write_str(self)
    }
}

impl LabelValue for str {
    fn visit(&self, v: &mut impl crate::LabelVisitor) {
        v.write_str(self)
    }
}

impl<T: LabelValue + ?Sized> LabelValue for &T {
    fn visit(&self, v: &mut impl crate::LabelVisitor) {
        T::visit(self, v)
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexSet;

    use crate::{
        FixedCardinalityDynamicLabel, FixedCardinalityLabel, LabelGroup, LabelGroupSet, LabelValue,
    };

    #[derive(Clone, Copy, PartialEq, Debug)]
    struct Error {
        kind: ErrorKind,
        route: &'static str,
    }

    struct ErrorsSet {
        routes: IndexSet<&'static str>,
    }

    #[derive(Clone, Copy, PartialEq, Debug)]
    enum ErrorKind {
        User,
        Internal,
        Network,
    }

    #[test]
    fn encoding_happy_path() {
        let set = ErrorsSet {
            routes: [
                "/user/:id/home",
                "/playlist/:id",
                "/user/:id/subscribe",
                "/user/:id/videos",
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(set.cardinality(), Some(12));

        let error_kinds = [ErrorKind::User, ErrorKind::Internal, ErrorKind::Network];
        for route in &set.routes {
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
        type Group = Error;

        #[allow(clippy::needless_question_mark)]
        fn cardinality(&self) -> Option<usize> {
            Some(ErrorKind::cardinality().checked_mul(self.routes.cardinality())?)
        }

        type Unique = (usize,);

        #[allow(unused_assignments)]
        fn encode(&self, value: Self::Group) -> Self::Unique {
            let mut mul = 1;
            let mut index = 0;

            index += value.kind.encode() * mul;
            mul *= ErrorKind::cardinality();

            index += self.routes.encode(value.route) * mul;
            mul *= self.routes.cardinality();

            (index,)
        }

        fn decode(&self, value: Self::Unique) -> Self::Group {
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

    impl LabelGroup for Error {
        fn label_names() -> impl IntoIterator<Item = &'static str> {
            ["kind", "route"]
        }

        fn label_values(self, v: &mut impl crate::LabelVisitor) {
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
        fn visit(&self, v: &mut impl crate::LabelVisitor) {
            match self {
                ErrorKind::User => v.write_str("user"),
                ErrorKind::Internal => v.write_str("internal"),
                ErrorKind::Network => v.write_str("network"),
            }
        }
    }
}
