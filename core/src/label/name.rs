use crate::metric::name::assert_metric_name;

#[repr(transparent)]
pub struct LabelName(str);

impl LabelName {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub const fn from_static(value: &'static str) -> &'static Self {
        assert_metric_name(value);

        // SAFETY: `LabelName` is transparent over `str`. There's no way to do this safely.
        // I could use bytemuck::TransparentWrapper, but the trait enabled users to skip this validation function.
        unsafe { &*(value as *const str as *const LabelName) }
    }
}
