use super::PropertyConverter;
use crate::context::ConversionContext;
use crate::types::CssRule;

/// Claims a property and unconditionally rejects it — goes to the `rejected` bucket,
/// not `custom_css`. Used for structurally incompatible properties like `animation`.
pub struct RejectedConverter {
    property: String,
}

impl RejectedConverter {
    pub fn new(property: impl Into<String>) -> Self {
        Self { property: property.into() }
    }
}

impl PropertyConverter for RejectedConverter {
    fn is_supported(&self, rule: &CssRule) -> bool {
        rule.property == self.property
    }

    fn convert(&self, ctx: &mut ConversionContext, rule: &CssRule) -> bool {
        ctx.reject(format!("{};", rule.declaration));
        true
    }
}
