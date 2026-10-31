/// A validated label name.
///
/// Labels may contain ASCII letters, numbers, as well as underscores. They must match the regex `[a-zA-Z_][a-zA-Z0-9_]*`.
#[repr(transparent)]
// Invariant: the inner str satisfies the label-name grammar documented above.
pub struct LabelName(str);

impl LabelName {
    /// Get the label name as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validate the string is a valid label
    ///
    /// # Panics
    /// This function will panic if the string does not conform to the prometheus label name requirements
    pub const fn from_str(value: &str) -> &Self {
        assert_label_name(value);

        // SAFETY: Creating the shared reference requires a live, aligned pointee
        // with valid contents for the returned lifetime. repr(transparent) gives
        // LabelName str's layout/alignment; the unsized pointer cast preserves
        // the data pointer and byte-length metadata. The input &str supplies
        // initialized UTF-8 and a shared borrow; the result's elided lifetime is
        // tied to that borrow. Validation above establishes the semantic name invariant.
        // No mutation, ownership transfer or extension of the byte extent occurs.
        unsafe { &*(value as *const str as *const LabelName) }
    }
}

const fn assert_label_name(name: &str) {
    assert!(!name.is_empty(), "string should not be empty");

    let mut i = 0;
    while i < name.len() {
        match name.as_bytes()[i] {
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'_' => {}
            _ => panic!("string should only contain [a-zA-Z0-9_]"),
        }
        i += 1;
    }

    assert!(
        !name.as_bytes()[0].is_ascii_digit(),
        "string should not start with a digit"
    );
}
