//! Traits and types used for representing groups of label-pairs

mod impls;

pub(crate) mod group;
pub(crate) mod name;
pub(crate) mod value;

pub use group::{ComposedGroup, LabelGroup, LabelGroupSet, LabelGroupVisitor, NoLabels};
pub use name::LabelName;
pub use value::{
    DynamicLabelSet, FixedCardinalityLabel, FixedCardinalitySet, LabelSet, LabelTestVisitor,
    LabelValue, LabelVisitor, StaticLabelSet,
};

#[cfg(all(test, feature = "lasso"))]
mod tests {
    use fake::{Fake, faker::name::raw::Name, locales::EN};
    use lasso::{Rodeo, RodeoReader, ThreadedRodeo};

    use super::LabelGroupSet;

    #[derive(Clone, Copy, PartialEq, Debug, measured_derive::LabelGroup)]
    #[label(crate = crate, set = ErrorsSet)]
    struct Error<'a> {
        kind: ErrorKind,
        #[label(fixed_with = RodeoReader)]
        route: &'a str,
    }

    #[derive(Clone, Copy, PartialEq, Debug, measured_derive::FixedCardinalityLabel)]
    #[label(crate = crate, rename_all = "kebab-case", singleton = "kind")]
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

        let set = ErrorsSet::new(rodeo.into_reader());
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
        kind: ErrorKind,
        #[label(fixed_with = RodeoReader)]
        route: &'a str,
        #[label(dynamic_with = ThreadedRodeo, default)]
        user: &'a str,
    }

    #[test]
    fn dynamic_labels() {
        let mut rodeo = Rodeo::new();

        rodeo.get_or_intern("/user/:id/home");
        rodeo.get_or_intern("/playlist/:id");
        rodeo.get_or_intern("/user/:id/subscribe");
        rodeo.get_or_intern("/user/:id/videos");

        let set = ErrorsSet2::new(rodeo.into_reader());
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
