/// A validated label name.
///
/// Labels may contain ASCII letters, numbers, as well as underscores. They must match the regex `[a-zA-Z_][a-zA-Z0-9_]*`.
#[repr(transparent)]
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

        // SAFETY: `LabelName` is transparent over `str`. There's no way to do this safely.
        // I could use bytemuck::TransparentWrapper, but the trait enabled users to skip this validation function.
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
