use alloc::sync::Arc;
use core::hash::Hash;

use super::{DynamicLabelSet, FixedCardinalitySet, LabelSet, LabelValue, LabelVisitor};

#[cfg(feature = "indexmap")]
impl<T: LabelValue + Hash + Eq + Clone, S: core::hash::BuildHasher> FixedCardinalitySet
    for indexmap::IndexSet<T, S>
{
    fn cardinality(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "indexmap")]
impl<T: LabelValue + Hash + Eq + Clone, S: core::hash::BuildHasher> LabelSet
    for indexmap::IndexSet<T, S>
{
    type Value<'a> = T;

    fn encode(&self, value: Self::Value<'_>) -> Option<usize> {
        self.get_index_of(&value)
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        self.get_index(value).unwrap().clone()
    }
}

impl LabelValue for String {
    fn visit<V: LabelVisitor>(&self, v: V) -> V::Output {
        v.write_str(self)
    }
}

impl LabelValue for str {
    fn visit<V: LabelVisitor>(&self, v: V) -> V::Output {
        v.write_str(self)
    }
}

impl<T: LabelValue + ?Sized> LabelValue for &T {
    fn visit<V: LabelVisitor>(&self, v: V) -> V::Output {
        T::visit(self, v)
    }
}

#[cfg(feature = "lasso")]
impl<K: lasso::Key, S: core::hash::BuildHasher> FixedCardinalitySet for lasso::RodeoReader<K, S> {
    fn cardinality(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "lasso")]
impl<K: lasso::Key + Hash, S: core::hash::BuildHasher + Clone> DynamicLabelSet
    for lasso::ThreadedRodeo<K, S>
{
}

#[cfg(feature = "lasso")]
impl<K: lasso::Key, S: core::hash::BuildHasher> LabelSet for lasso::RodeoReader<K, S> {
    type Value<'a> = &'a str;

    fn encode(&self, value: Self::Value<'_>) -> Option<usize> {
        Some(self.get(value)?.into_usize())
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        self.resolve(&K::try_from_usize(value).unwrap())
    }
}

#[cfg(feature = "lasso")]
impl<K: lasso::Key + Hash, S: core::hash::BuildHasher + Clone> LabelSet
    for lasso::ThreadedRodeo<K, S>
{
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
