use alloc::sync::Arc;

use super::{DynamicLabelSet, FixedCardinalitySet, LabelSet, LabelValue, LabelVisitor};

#[cfg(feature = "indexmap")]
impl<T: LabelValue + core::hash::Hash + Eq + Clone, S: core::hash::BuildHasher> FixedCardinalitySet
    for indexmap::IndexSet<T, S>
{
    fn cardinality(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "indexmap")]
impl<T: LabelValue + core::hash::Hash + Eq + Clone, S: core::hash::BuildHasher> LabelSet for indexmap::IndexSet<T, S> {
    type Value<'a> = T;

    fn dynamic_cardinality(&self) -> Option<usize> {
        Some(self.len())
    }

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

impl LabelValue for std::borrow::Cow<'_, str> {
    fn visit<V: LabelVisitor>(&self, v: V) -> V::Output {
        v.write_str(self)
    }
}

impl LabelValue for f64 {
    fn visit<V: LabelVisitor>(&self, v: V) -> V::Output {
        v.write_float(*self)
    }
}

impl LabelValue for bool {
    fn visit<V: LabelVisitor>(&self, v: V) -> V::Output {
        v.write_str(if *self { "true" } else { "false" })
    }
}

macro_rules! int_label_value {
    ($($t:ty),*) => {$(
        impl LabelValue for $t {
            fn visit<V: LabelVisitor>(&self, v: V) -> V::Output {
                v.write_int(i64::from(*self))
            }
        }
    )*};
}
int_label_value!(i8, i16, i32, i64, u8, u16, u32);

macro_rules! large_int_label_value {
    ($($t:ty),*) => {$(
        impl LabelValue for $t {
            fn visit<V: LabelVisitor>(&self, v: V) -> V::Output {
                match i64::try_from(*self) {
                    Ok(x) => v.write_int(x),
                    Err(_) => v.write_str((*self).format_into(&mut core::fmt::NumBuffer::new())),
                }
            }
        }
    )*};
}
large_int_label_value!(u64, usize, isize);

/// A range of integer label values, eg worker IDs `0..num_workers`.
impl LabelSet for core::ops::Range<usize> {
    type Value<'a> = usize;

    fn dynamic_cardinality(&self) -> Option<usize> {
        Some(self.len())
    }

    fn encode(&self, value: usize) -> Option<usize> {
        self.contains(&value).then(|| value - self.start)
    }

    fn decode(&self, value: usize) -> usize {
        self.start + value
    }
}

impl FixedCardinalitySet for core::ops::Range<usize> {}

#[cfg(feature = "paracord")]
impl<S: core::hash::BuildHasher> DynamicLabelSet for paracord::ParaCord<S> {}

#[cfg(feature = "paracord")]
impl<S: core::hash::BuildHasher> LabelSet for paracord::ParaCord<S> {
    type Value<'a> = &'a str;

    fn dynamic_cardinality(&self) -> Option<usize> {
        Some(self.len())
    }

    fn encode(&self, value: Self::Value<'_>) -> Option<usize> {
        Some(self.get_or_intern(value).into_repr() as usize)
    }

    fn find(&self, value: Self::Value<'_>) -> Option<usize> {
        Some(self.get(value)?.into_repr() as usize)
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        self.resolve(
            u32::try_from(value)
                .ok()
                .and_then(paracord::Key::try_from_repr)
                .unwrap(),
        )
    }
}

#[cfg(feature = "lasso")]
impl<K: lasso::Key, S: core::hash::BuildHasher> FixedCardinalitySet for lasso::RodeoReader<K, S> {}

#[cfg(feature = "lasso")]
impl<K: lasso::Key + core::hash::Hash, S: core::hash::BuildHasher + Clone> DynamicLabelSet
    for lasso::ThreadedRodeo<K, S>
{
}

#[cfg(feature = "lasso")]
impl<K: lasso::Key, S: core::hash::BuildHasher> LabelSet for lasso::RodeoReader<K, S> {
    type Value<'a> = &'a str;

    fn dynamic_cardinality(&self) -> Option<usize> {
        Some(self.len())
    }

    fn encode(&self, value: Self::Value<'_>) -> Option<usize> {
        Some(self.get(value)?.into_usize())
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        self.resolve(&K::try_from_usize(value).unwrap())
    }
}

#[cfg(feature = "lasso")]
impl<K: lasso::Key + core::hash::Hash, S: core::hash::BuildHasher + Clone> LabelSet for lasso::ThreadedRodeo<K, S> {
    type Value<'a> = &'a str;

    fn dynamic_cardinality(&self) -> Option<usize> {
        None
    }

    fn encode(&self, value: Self::Value<'_>) -> Option<usize> {
        Some(self.try_get_or_intern(value).ok()?.into_usize())
    }

    fn find(&self, value: Self::Value<'_>) -> Option<usize> {
        Some(self.get(value)?.into_usize())
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        self.resolve(&K::try_from_usize(value).unwrap())
    }
}

#[cfg(feature = "phf")]
impl LabelSet for phf::OrderedSet<&'static str> {
    type Value<'a> = &'a str;

    fn dynamic_cardinality(&self) -> Option<usize> {
        Some(self.len())
    }

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
        <T as FixedCardinalitySet>::cardinality(self)
    }
}

impl<T: DynamicLabelSet + ?Sized> DynamicLabelSet for Arc<T> {}

impl<T: LabelSet + ?Sized> LabelSet for Arc<T> {
    type Value<'a> = T::Value<'a>;

    fn dynamic_cardinality(&self) -> Option<usize> {
        T::dynamic_cardinality(self)
    }

    fn encode(&self, value: Self::Value<'_>) -> Option<usize> {
        T::encode(self, value)
    }

    fn find(&self, value: Self::Value<'_>) -> Option<usize> {
        T::find(self, value)
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        T::decode(self, value)
    }
}

#[cfg(all(
    test,
    any(
        feature = "phf",
        feature = "indexmap",
        feature = "lasso",
        feature = "paracord"
    )
))]
mod tests {
    #[cfg(any(feature = "phf", feature = "indexmap", feature = "lasso"))]
    use crate::label::FixedCardinalitySet;
    use crate::label::LabelSet;
    const VALUES: [&str; 5] = ["loop", "continue", "break", "fn", "extern"];

    macro_rules! check_values {
        ($set:expr) => {{
            let set = &$set;
            for (index, value) in VALUES.into_iter().enumerate() {
                assert_eq!(set.encode(value), Some(index), "encode {value}");
                assert_eq!(set.decode(index), value, "decode {index}");
                assert_eq!(set.find(value), Some(index), "find {value}");
            }
        }};
    }

    #[cfg(feature = "phf")]
    #[test]
    fn phf_ordered_set() {
        let set = phf::phf_ordered_set! { "loop", "continue", "break", "fn", "extern" };
        check_values!(set);
        assert_eq!(set.cardinality(), 5);
        assert_eq!(set.encode("missing"), None);
        assert_eq!(set.find("missing"), None);
    }

    #[cfg(feature = "indexmap")]
    #[test]
    fn indexset() {
        let set: indexmap::IndexSet<&str> = VALUES.into_iter().collect();
        check_values!(set);
        assert_eq!(set.cardinality(), 5);
        assert_eq!(set.encode("missing"), None);
        assert_eq!(set.find("missing"), None);
    }

    #[cfg(feature = "lasso")]
    #[test]
    fn lasso_reader() {
        let set = VALUES.into_iter().collect::<lasso::Rodeo>().into_reader();
        check_values!(set);
        assert_eq!(set.cardinality(), 5);
        assert_eq!(set.encode("missing"), None);
        assert_eq!(set.find("missing"), None);
    }

    #[cfg(feature = "lasso")]
    #[test]
    fn lasso_dynamic() {
        let set = lasso::ThreadedRodeo::<lasso::Spur>::new();
        assert_eq!(set.find("missing"), None);
        assert!(set.is_empty());
        check_values!(set);
        check_values!(set); // Existing values must retain their IDs without new entries.
        assert_eq!(set.len(), 5);
        assert_eq!(set.find("missing"), None);
        assert_eq!(set.len(), 5);
    }

    #[cfg(feature = "paracord")]
    #[test]
    fn paracord() {
        let set = paracord::ParaCord::default();
        assert_eq!(set.find("missing"), None);
        assert!(set.is_empty());
        check_values!(set);
        check_values!(set);
        assert_eq!(set.len(), 5);
        assert_eq!(set.find("missing"), None);
        assert_eq!(set.len(), 5);
    }
}
