use super::PropertyConverter;
use crate::context::ConversionContext;
use crate::types::CssRule;

/// Claims a property but always declines → routes to `custom_css`.
/// Used for properties that have no converter yet (Phase 1 stubs).
pub struct NoopConverter {
    property: String,
}

impl NoopConverter {
    pub fn new(property: impl Into<String>) -> Self {
        Self { property: property.into() }
    }
}

impl PropertyConverter for NoopConverter {
    fn is_supported(&self, rule: &CssRule) -> bool {
        rule.property == self.property
    }

    fn convert(&self, _ctx: &mut ConversionContext, _rule: &CssRule) -> bool {
        false
    }
}
