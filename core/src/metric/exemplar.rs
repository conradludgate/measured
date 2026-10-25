//! Extra labels attached to an observation.
use crate::label::{LabelGroup, LabelGroupVisitor, LabelName};
use std::time::SystemTime;

/// An example of an observation, with labels that identify it, such as a trace ID.
///
/// Exemplars are written by the OpenMetrics and protobuf encoders, and ignored by the prometheus text encoder.
/// OpenMetrics limits the labels of an exemplar to 128 characters in total. Exemplars with longer labels
/// are not written by the OpenMetrics encoder.
///
#[derive(Clone)]
pub struct Exemplar {
    labels: Vec<(&'static LabelName, String)>,
    value: f64,
    timestamp: Option<SystemTime>,
}

impl core::fmt::Debug for Exemplar {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Exemplar")
            .field(
                "labels",
                &self
                    .labels
                    .iter()
                    .map(|(name, value)| (name.as_str(), value))
                    .collect::<Vec<_>>(),
            )
            .field("value", &self.value)
            .field("timestamp", &self.timestamp)
            .finish()
    }
}

impl PartialEq for Exemplar {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.timestamp == other.timestamp
            && self.labels.len() == other.labels.len()
            && self
                .labels
                .iter()
                .zip(&other.labels)
                .all(|((a, x), (b, y))| a.as_str() == b.as_str() && x == y)
    }
}

impl Default for Exemplar {
    fn default() -> Self {
        Self::new()
    }
}

impl Exemplar {
    /// An exemplar with no labels.
    ///
    /// When recorded with a metric, such as with [`ExemplarCounter::inc_with_exemplar`], the value is set
    /// to the observed value.
    pub fn new() -> Self {
        Self {
            labels: Vec::new(),
            value: 0.0,
            timestamp: None,
        }
    }

    /// Add a label to this exemplar
    #[must_use]
    pub fn with_label(mut self, name: &'static LabelName, value: impl Into<String>) -> Self {
        self.labels.push((name, value.into()));
        self
    }

    /// Set the value of this exemplar
    #[must_use]
    pub fn with_value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    /// Set when this exemplar was observed
    #[must_use]
    pub fn with_timestamp(mut self, timestamp: SystemTime) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// The labels of this exemplar
    pub fn labels(&self) -> impl LabelGroup + '_ {
        ExemplarLabels(&self.labels)
    }

    /// The number of characters in the label names and values, which OpenMetrics limits to 128.
    pub fn labels_len(&self) -> usize {
        self.labels
            .iter()
            .map(|(name, value)| name.as_str().chars().count() + value.chars().count())
            .sum()
    }

    /// The observed value
    pub fn value(&self) -> f64 {
        self.value
    }

    /// When this exemplar was observed, if set
    pub fn timestamp(&self) -> Option<SystemTime> {
        self.timestamp
    }
}

struct ExemplarLabels<'a>(&'a [(&'static LabelName, String)]);

impl LabelGroup for ExemplarLabels<'_> {
    fn visit_values(&self, v: &mut impl LabelGroupVisitor) {
        for (name, value) in self.0 {
            v.write_value(name, value);
        }
    }
}
