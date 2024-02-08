use alloc::sync::Arc;
use core::hash::{BuildHasher, Hash};

use super::{
    DynamicLabelSet, FixedCardinalitySet, LabelGroup, LabelGroupSet, LabelSet, LabelValue,
    LabelVisitor,
};

#[cfg(feature = "indexmap")]
impl<T: LabelValue + Hash + Eq + Clone, S: BuildHasher> FixedCardinalitySet
    for indexmap::IndexSet<T, S>
{
    fn cardinality(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "indexmap")]
impl<T: LabelValue + Hash + Eq + Clone, S: BuildHasher> LabelSet for indexmap::IndexSet<T, S> {
    type Value<'a> = T;

    fn encode(&self, value: Self::Value<'_>) -> Option<usize> {
        self.get_index_of(&value)
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        self.get_index(value).unwrap().clone()
    }
}

impl LabelValue for String {
    fn visit(&self, v: &mut impl LabelVisitor) {
        v.write_str(self);
    }
}

impl LabelValue for str {
    fn visit(&self, v: &mut impl LabelVisitor) {
        v.write_str(self);
    }
}

impl<T: LabelValue + ?Sized> LabelValue for &T {
    fn visit(&self, v: &mut impl LabelVisitor) {
        T::visit(self, v);
    }
}

#[cfg(feature = "lasso")]
impl<K: lasso::Key, S: BuildHasher> FixedCardinalitySet for lasso::RodeoReader<K, S> {
    fn cardinality(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "lasso")]
impl<K: lasso::Key + Hash, S: BuildHasher + Clone> DynamicLabelSet for lasso::ThreadedRodeo<K, S> {}

#[cfg(feature = "lasso")]
impl<K: lasso::Key, S: BuildHasher> LabelSet for lasso::RodeoReader<K, S> {
    type Value<'a> = &'a str;

    fn encode(&self, value: Self::Value<'_>) -> Option<usize> {
        Some(self.get(value)?.into_usize())
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        self.resolve(&K::try_from_usize(value).unwrap())
    }
}

#[cfg(feature = "lasso")]
impl<K: lasso::Key + Hash, S: BuildHasher + Clone> LabelSet for lasso::ThreadedRodeo<K, S> {
    type Value<'a> = &'a str;

    fn encode(&self, value: Self::Value<'_>) -> Option<usize> {
        Some(self.try_get_or_intern(value).ok()?.into_usize())
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        self.resolve(&K::try_from_usize(value).unwrap())
    }
}

#[cfg(feature = "phf")]
impl LabelSet for phf::OrderedSet<&'static str> {
    type Value<'a> = &'a str;

    fn encode(&self, value: Self::Value<'_>) -> Option<usize> {
        self.get_index(value)
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        self.index(value).unwrap()
    }
}

#[cfg(feature = "phf")]
impl FixedCardinalitySet for phf::OrderedSet<&'static str> {
    fn cardinality(&self) -> usize {
        self.len()
    }
}

/// `ComposedGroup` represents either a combine [`LabelGroup`] or a [`LabelGroupSet`]. See [`LabelGroup::compose_with`]
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct ComposedGroup<A, B>(pub A, pub B);

impl<A: LabelGroupSet, B: LabelGroupSet> LabelGroupSet for ComposedGroup<A, B> {
    type Group<'a> = ComposedGroup<A::Group<'a>, B::Group<'a>>;

    fn cardinality(&self) -> Option<usize> {
        self.0
            .cardinality()
            .and_then(|x| x.checked_mul(self.1.cardinality()?))
    }

    fn encode_dense(&self, values: Self::Unique) -> Option<usize> {
        let mut mul = 1;
        let mut index = 0;

        index += self.1.encode_dense(values.1)? * mul;
        mul *= self.1.cardinality()?;

        index += self.0.encode_dense(values.0)? * mul;
        // mul *= self.1.cardinality()?;

        Some(index)
    }

    fn decode_dense(&self, value: usize) -> Self::Group<'_> {
        let index = value;
        let (index, index1) = (
            index / self.1.cardinality().unwrap(),
            index % self.1.cardinality().unwrap(),
        );
        let b = self.1.decode_dense(index1);
        let (index, index1) = (
            index / self.0.cardinality().unwrap(),
            index % self.0.cardinality().unwrap(),
        );
        let a = self.0.decode_dense(index1);
        debug_assert_eq!(index, 0);
        ComposedGroup(a, b)
    }

    type Unique = ComposedGroup<A::Unique, B::Unique>;

    fn encode(&self, value: Self::Group<'_>) -> Option<Self::Unique> {
        Some(ComposedGroup(
            self.0.encode(value.0)?,
            self.1.encode(value.1)?,
        ))
    }

    fn decode(&self, value: &Self::Unique) -> Self::Group<'_> {
        ComposedGroup(self.0.decode(&value.0), self.1.decode(&value.1))
    }
}

impl<A: LabelGroup, B: LabelGroup> LabelGroup for ComposedGroup<A, B> {
    fn visit_values(&self, v: &mut impl super::LabelSetVisitor) {
        self.0.visit_values(v);
        self.1.visit_values(v);
    }
}

impl<T: LabelGroup> LabelGroup for &T {
    fn visit_values(&self, v: &mut impl super::LabelSetVisitor) {
        T::visit_values(self, v);
    }
}

impl<T: LabelGroupSet + ?Sized> LabelGroupSet for &'static T {
    type Group<'a> = T::Group<'a>;

    fn cardinality(&self) -> Option<usize> {
        T::cardinality(self)
    }

    fn encode_dense(&self, value: Self::Unique) -> Option<usize> {
        T::encode_dense(self, value)
    }

    fn decode_dense(&self, value: usize) -> Self::Group<'_> {
        T::decode_dense(self, value)
    }

    type Unique = T::Unique;

    fn encode(&self, value: Self::Group<'_>) -> Option<Self::Unique> {
        T::encode(self, value)
    }

    fn decode(&self, value: &Self::Unique) -> Self::Group<'_> {
        T::decode(self, value)
    }
}

impl<T: LabelGroupSet + ?Sized> LabelGroupSet for Arc<T> {
    type Group<'a> = T::Group<'a>;

    fn cardinality(&self) -> Option<usize> {
        T::cardinality(self)
    }

    fn encode_dense(&self, value: Self::Unique) -> Option<usize> {
        T::encode_dense(self, value)
    }

    fn decode_dense(&self, value: usize) -> Self::Group<'_> {
        T::decode_dense(self, value)
    }

    type Unique = T::Unique;

    fn encode(&self, value: Self::Group<'_>) -> Option<Self::Unique> {
        T::encode(self, value)
    }

    fn decode(&self, value: &Self::Unique) -> Self::Group<'_> {
        T::decode(self, value)
    }
}

impl<T: FixedCardinalitySet + ?Sized> FixedCardinalitySet for Arc<T> {
    fn cardinality(&self) -> usize {
        T::cardinality(self)
    }
}

impl<T: DynamicLabelSet + ?Sized> DynamicLabelSet for Arc<T> {}

impl<T: LabelSet + ?Sized> LabelSet for Arc<T> {
    type Value<'a> = T::Value<'a>;

    fn encode(&self, value: Self::Value<'_>) -> Option<usize> {
        T::encode(self, value)
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        T::decode(self, value)
    }
}

#[cfg(test)]
mod tests {
    use crate::label::{FixedCardinalitySet, LabelSet};

    #[cfg(feature = "phf")]
    #[test]
    fn phf_ordered_set() {
        let set = phf::phf_ordered_set! {
            "loop",
            "continue",
            "break",
            "fn",
            "extern",
        };

        assert_eq!(set.cardinality(), 5);

        // make sure it's repeatable
        for _ in 0..2 {
            assert_eq!(set.encode("loop"), Some(0));
            assert_eq!(set.decode(0), "loop");

            assert_eq!(set.encode("continue"), Some(1));
            assert_eq!(set.decode(1), "continue");

            assert_eq!(set.encode("break"), Some(2));
            assert_eq!(set.decode(2), "break");

            assert_eq!(set.encode("fn"), Some(3));
            assert_eq!(set.decode(3), "fn");

            assert_eq!(set.encode("extern"), Some(4));
            assert_eq!(set.decode(4), "extern");
        }
    }

    #[cfg(feature = "indexmap")]
    #[test]
    fn indexset() {
        use indexmap::IndexSet;

        let set: IndexSet<&'static str> = ["loop", "continue", "break", "fn", "extern"]
            .into_iter()
            .collect();

        assert_eq!(set.cardinality(), 5);

        // make sure it's repeatable
        for _ in 0..2 {
            assert_eq!(set.encode("loop"), Some(0));
            assert_eq!(set.decode(0), "loop");

            assert_eq!(set.encode("continue"), Some(1));
            assert_eq!(set.decode(1), "continue");

            assert_eq!(set.encode("break"), Some(2));
            assert_eq!(set.decode(2), "break");

            assert_eq!(set.encode("fn"), Some(3));
            assert_eq!(set.decode(3), "fn");

            assert_eq!(set.encode("extern"), Some(4));
            assert_eq!(set.decode(4), "extern");
        }
    }

    #[cfg(feature = "lasso")]
    #[test]
    fn lasso_reader() {
        let set = ["loop", "continue", "break", "fn", "extern"]
            .into_iter()
            .collect::<lasso::Rodeo>()
            .into_reader();

        assert_eq!(set.cardinality(), 5);

        // make sure it's repeatable
        for _ in 0..2 {
            assert_eq!(set.encode("loop"), Some(0));
            assert_eq!(set.decode(0), "loop");

            assert_eq!(set.encode("continue"), Some(1));
            assert_eq!(set.decode(1), "continue");

            assert_eq!(set.encode("break"), Some(2));
            assert_eq!(set.decode(2), "break");

            assert_eq!(set.encode("fn"), Some(3));
            assert_eq!(set.decode(3), "fn");

            assert_eq!(set.encode("extern"), Some(4));
            assert_eq!(set.decode(4), "extern");
        }
    }

    #[cfg(feature = "lasso")]
    #[test]
    fn lasso_dynamic() {
        use lasso::Spur;

        let set = lasso::ThreadedRodeo::<Spur>::new();

        // make sure it's repeatable
        for _ in 0..2 {
            assert_eq!(set.encode("loop"), Some(0));
            assert_eq!(set.decode(0), "loop");

            assert_eq!(set.encode("continue"), Some(1));
            assert_eq!(set.decode(1), "continue");

            assert_eq!(set.encode("break"), Some(2));
            assert_eq!(set.decode(2), "break");

            assert_eq!(set.encode("fn"), Some(3));
            assert_eq!(set.decode(3), "fn");

            assert_eq!(set.encode("extern"), Some(4));
            assert_eq!(set.decode(4), "extern");
        }
    }
}
