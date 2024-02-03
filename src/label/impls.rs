use std::hash::{BuildHasher, Hash};

use super::{
    DynamicLabel, FixedCardinalityDynamicLabel, LabelGroup, LabelGroupSet, LabelValue, LabelVisitor,
};

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

#[derive(Hash, PartialEq, Eq)]
pub struct ComposedGroup<A, B>(A, B);

impl<A: LabelGroupSet, B: LabelGroupSet> LabelGroupSet for ComposedGroup<A, B> {
    type Group<'a> = ComposedGroup<A::Group<'a>, B::Group<'a>>
    where
        Self: 'a;

    fn cardinality(&self) -> Option<usize> {
        self.0
            .cardinality()
            .and_then(|x| x.checked_mul(self.1.cardinality()?))
    }

    fn encode_dense(&self, _value: Self::Unique) -> Option<usize> {
        todo!()
    }

    type Unique = ComposedGroup<A::Unique, B::Unique>;

    fn encode<'a>(&'a self, value: Self::Group<'a>) -> Option<Self::Unique> {
        Some(ComposedGroup(
            self.0.encode(value.0)?,
            self.1.encode(value.1)?,
        ))
    }

    fn decode(&self, value: Self::Unique) -> Self::Group<'_> {
        ComposedGroup(self.0.decode(value.0), self.1.decode(value.1))
    }
}

impl<A: LabelGroup, B: LabelGroup> LabelGroup for ComposedGroup<A, B> {
    fn label_names() -> impl IntoIterator<Item = &'static str> {
        A::label_names().into_iter().chain(B::label_names())
    }

    fn label_values(self, v: &mut impl LabelVisitor) {
        self.0.label_values(v);
        self.1.label_values(v);
    }
}
