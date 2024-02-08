use core::hash::Hash;
use std::sync::Arc;

/// A trait for the label names and values in a label set
pub trait LabelGroupVisitor {
    /// Write a label name and label value to the visitor
    fn write_value(&mut self, name: &super::LabelName, x: &impl super::LabelValue);
}

/// `LabelGroup` represents a group of label-pairs
pub trait LabelGroup {
    /// Writes all the label values into the visitor in the same order as the names
    fn visit_values(&self, v: &mut impl LabelGroupVisitor);

    /// Borrow this label group
    fn by_ref(&self) -> &Self {
        self
    }

    /// Combine this group with another
    fn compose_with<T: LabelGroup>(self, other: T) -> ComposedGroup<Self, T>
    where
        Self: Sized,
    {
        ComposedGroup(self, other)
    }
}

/// `LabelGroupSet` is a helper for [`LabelGroup`]s.
///
/// The `LabelGroup` pairs might need some extra data in order to encode/decode the values into their
/// compressed integer form.
pub trait LabelGroupSet {
    type Group<'a>: LabelGroup;

    /// The number of possible label-pairs the associated group can represent
    fn cardinality(&self) -> Option<usize>;

    /// If the label set is fixed in cardinality, it must return a value here in the range of `0..cardinality`
    fn encode_dense(&self, _value: Self::Unique) -> Option<usize>;
    /// If the label set is fixed in cardinality, a value in the range of `0..cardinality` should decode without panicking.
    fn decode_dense(&self, value: usize) -> Self::Group<'_>;

    /// A type that can uniquely represent all possible labels
    type Unique: Copy + Hash + Eq;

    /// Encode the label groups into the unique compressed representation
    fn encode(&self, value: Self::Group<'_>) -> Option<Self::Unique>;
    /// Decodes the compressed representation into the label values
    fn decode(&self, value: &Self::Unique) -> Self::Group<'_>;
}

/// A [`LabelGroup`] with no label pairs
pub struct NoLabels;

impl LabelGroup for NoLabels {
    fn visit_values(&self, _v: &mut impl LabelGroupVisitor) {}
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
    fn visit_values(&self, v: &mut impl super::LabelGroupVisitor) {
        self.0.visit_values(v);
        self.1.visit_values(v);
    }
}

impl<T: LabelGroup> LabelGroup for &T {
    fn visit_values(&self, v: &mut impl super::LabelGroupVisitor) {
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
