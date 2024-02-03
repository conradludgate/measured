use std::hash::{BuildHasher, Hash};

use super::{DynamicLabel, FixedCardinalityDynamicLabel, LabelValue, LabelVisitor};

impl<T: LabelValue + Hash + Eq + Clone, S: BuildHasher> FixedCardinalityDynamicLabel
    for indexmap::IndexSet<T, S>
{
    type Value<'a> = T where Self: 'a;

    fn cardinality(&self) -> usize {
        self.len()
    }

    fn encode<'a>(&'a self, value: Self::Value<'a>) -> Option<usize> {
        self.get_index_of(&value)
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        self.get_index(value).unwrap().clone()
    }
}

impl LabelValue for String {
    fn visit(&self, v: &mut impl LabelVisitor) {
        v.write_str(self)
    }
}

impl LabelValue for str {
    fn visit(&self, v: &mut impl LabelVisitor) {
        v.write_str(self)
    }
}

impl<T: LabelValue + ?Sized> LabelValue for &T {
    fn visit(&self, v: &mut impl LabelVisitor) {
        T::visit(self, v)
    }
}

impl<K: lasso::Key, S: BuildHasher> FixedCardinalityDynamicLabel for lasso::RodeoReader<K, S> {
    type Value<'a> = &'a str where Self: 'a;

    fn cardinality(&self) -> usize {
        self.len()
    }

    fn encode<'a>(&'a self, value: Self::Value<'a>) -> Option<usize> {
        Some(self.get(value)?.into_usize())
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        self.resolve(&K::try_from_usize(value).unwrap())
    }
}

impl<K: lasso::Key + Hash, S: BuildHasher + Clone> DynamicLabel for lasso::ThreadedRodeo<K, S> {
    type Value<'a> = &'a str where Self: 'a;

    fn encode<'a>(&'a self, value: Self::Value<'a>) -> Option<usize> {
        Some(self.try_get_or_intern(value).ok()?.into_usize())
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        self.resolve(&K::try_from_usize(value).unwrap())
    }
}
