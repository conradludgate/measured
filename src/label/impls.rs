use std::hash::{BuildHasher, Hash};

use super::{FixedCardinalityDynamicLabel, LabelValue, LabelVisitor};


impl<T: LabelValue + Hash + Eq + Clone, S: BuildHasher> FixedCardinalityDynamicLabel
    for indexmap::IndexSet<T, S>
{
    type Value<'a> = T where Self: 'a;

    fn cardinality(&self) -> usize {
        self.len()
    }

    fn encode<'a>(&'a self, value: Self::Value<'a>) -> usize {
        self.get_index_of(&value).unwrap()
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

    fn encode<'a>(&'a self, value: Self::Value<'a>) -> usize {
        self.get(value).unwrap().into_usize()
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        self.resolve(&K::try_from_usize(value).unwrap())
    }
}
