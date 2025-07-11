use core::marker::PhantomData;

use crate::LabelGroup;

use super::LabelGroupSet;

/// `StaticLabelSet` is a [`LabelSet`] for a [`FixedCardinalityLabel`]
pub struct StaticLabelSet<T>(PhantomData<T>);

impl<T> Default for StaticLabelSet<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> StaticLabelSet<T> {
    /// Create a new `StaticLabelSet`
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: FixedCardinalityLabel> FixedCardinalitySet for StaticLabelSet<T> {
    fn cardinality(&self) -> usize {
        T::cardinality()
    }
}

impl<T: FixedCardinalityLabel> LabelSet for StaticLabelSet<T> {
    type Value<'a> = T;

    fn dynamic_cardinality(&self) -> Option<usize> {
        Some(T::cardinality())
    }

    fn encode(&self, value: Self::Value<'_>) -> Option<usize> {
        Some(value.encode())
    }

    fn decode(&self, value: usize) -> Self::Value<'_> {
        T::decode(value)
    }
}

impl<T: FixedCardinalityLabel + LabelGroup> LabelGroupSet for StaticLabelSet<T> {
    type Group<'a> = T;

    fn cardinality(&self) -> Option<usize> {
        LabelSet::dynamic_cardinality(self)
    }

    fn encode_dense(&self, value: Self::Unique) -> Option<usize> {
        Some(value)
    }

    fn decode_dense(&self, value: usize) -> Self::Group<'_> {
        LabelSet::decode(self, value)
    }

    type Unique = usize;

    fn encode(&self, value: Self::Group<'_>) -> Option<Self::Unique> {
        LabelSet::encode(self, value)
    }

    fn decode(&self, value: &Self::Unique) -> Self::Group<'_> {
        LabelSet::decode(self, *value)
    }
}

/// A [`LabelVisitor`] that is useful for testing purposes
#[derive(Default, Debug)]
pub struct LabelTestVisitor;

impl LabelVisitor for LabelTestVisitor {
    type Output = String;
    fn write_int(self, x: i64) -> String {
        self.write_str(itoa::Buffer::new().format(x))
    }

    fn write_float(self, x: f64) -> String {
        if x.is_infinite() {
            if x.is_sign_positive() {
                self.write_str("+Inf")
            } else {
                self.write_str("-Inf")
            }
        } else if x.is_nan() {
            self.write_str("NaN")
        } else {
            self.write_str(ryu::Buffer::new().format(x))
        }
    }

    fn write_str(self, x: &str) -> String {
        x.to_owned()
    }
}

/// A trait for visiting the value of a label
pub trait LabelVisitor {
    /// Output of this visitor
    type Output;

    /// Write an integer value to this visitor
    fn write_int(self, x: i64) -> Self::Output;
    /// Write a floating point value to this visitor
    fn write_float(self, x: f64) -> Self::Output;
    /// Write a string value to this visitor
    fn write_str(self, x: &str) -> Self::Output;
}

/// A type that contains a label value
pub trait LabelValue {
    /// Visit this value
    fn visit<V: LabelVisitor>(&self, v: V) -> V::Output;
}

/// `FixedCardinalityLabel` represents a label value with a value<-> integer encoding known at compile time.
///
/// This is usually implemented by enums with the [`FixedCardinalityLabel`](macro@crate::FixedCardinalityLabel) derive macro
pub trait FixedCardinalityLabel: LabelValue + Copy {
    /// The number of possible label values
    fn cardinality() -> usize;

    /// Encode the label value into an integer
    fn encode(&self) -> usize;

    /// Decode the integer into the associated label value.
    ///
    /// If the integer is outside the range of this set, the behaviour is not defined.
    /// It would most likely panic.
    fn decode(value: usize) -> Self;
}

/// `FixedCardinalitySet` is an immutable [`LabelSet`] that has a known fixed size.
///
/// An example of a dynamic label that has a fixed capacity is an API path with parameters removed
/// * `/api/v1/users`
/// * `/api/v1/users/:id`
/// * `/api/v1/products`
/// * `/api/v1/products/:id`
/// * `/api/v1/products/:id/owner`
/// * `/api/v1/products/:id/purchase`
///
/// These values can be awkward to set up as an enum for a compile time metric, but might be easier to build
/// as a runtime quantity.
///
/// Additionally, sometimes the set of label values can only be known based on some startup configuration, but never changes.
pub trait FixedCardinalitySet: LabelSet {
    /// The maximum number of possible label values
    ///
    /// # Details
    /// This number must never change due to some interior mutation, eg with an atomic or a mutex.
    fn cardinality(&self) -> usize {
        LabelSet::dynamic_cardinality(self).unwrap()
    }
}

/// `DynamicLabelSet` is a mutable [`LabelSet`] that has an unknown maximum size.
///
/// This is not recommended to be used, but provided for completeness sake.
/// [Prometheus recommends against high-cardinality metrics](https://grafana.com/blog/2022/02/15/what-are-cardinality-spikes-and-why-do-they-matter/)
/// but there might be cases where you still want to use this
///
/// 1. Compatibility with your existing setup
/// 2. Not exporting to prometheus
/// 3. You know there wont be many labels but you just don't know what they are
pub trait DynamicLabelSet: LabelSet {
    #[doc(hidden)]
    fn __private_check_dynamic() {}
}

/// `LabelSet` defines a way to take a label value, eg a `&str`, and encode it into a compressed integer for more efficient encoding.
///
/// How this encoding is done is up to the application, but several ways are provided.
/// * [`lasso::RodeoReader`] is an immutable label set that stores a `&str` into a larger `String` allocation. It has a `HashMap` to quickly find the index of the string
/// * [`lasso::ThreadedRodeo`] is a mutable label set that works similarly to the `RodeoReader`.
/// * [`indexmap::IndexSet`] is an immutable `HashSet` that stores an associated index position of the inserted elements.
pub trait LabelSet {
    /// The label value this set can encode
    type Value<'a>: LabelValue;

    /// The maximum number of possible label values
    fn dynamic_cardinality(&self) -> Option<usize>;

    /// Encode the label value into an integer. Returns `None` if the value is not in the set
    fn encode(&self, value: Self::Value<'_>) -> Option<usize>;

    /// Decode the integer into the associated label value.
    ///
    /// If the integer is outside the range of this set, the behaviour is not defined.
    /// It would most likely panic.
    fn decode(&self, value: usize) -> Self::Value<'_>;
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]

    use measured_derive::MetricGroup;

    use crate::CounterVec;

    use super::StaticLabelSet;

    #[derive(Clone, Copy, PartialEq, Debug, measured_derive::FixedCardinalityLabel)]
    #[label(crate = crate)]
    #[label(singleton = "kind")]
    enum ErrorKind {
        User,
        Internal,
        Network,
    }

    #[derive(MetricGroup, Default)]
    #[metric(crate = crate)]
    struct Metrics {
        errors: CounterVec<StaticLabelSet<ErrorKind>>,
    }
}
