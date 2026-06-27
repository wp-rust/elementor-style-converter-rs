use super::PropertyConverter;
use crate::context::ConversionContext;
use crate::types::{string_prop, CssRule};

/// Converts a CSS property to a `string` PropValue.
/// When `allowed_values` is `Some`, acts as an enum: declines values not in the list.
/// When `None`, accepts any non-empty string (free-string / passthrough).
pub struct StringConverter {
    property: String,
    allowed_values: Option<Vec<String>>,
}

impl StringConverter {
    pub fn new(property: impl Into<String>, allowed_values: Option<Vec<String>>) -> Self {
        Self {
            property: property.into(),
            allowed_values,
        }
    }

    pub fn passthrough(property: impl Into<String>) -> Self {
        Self::new(property, None)
    }

    pub fn enumerated(property: impl Into<String>, values: Vec<&str>) -> Self {
        Self::new(
            property,
            Some(values.into_iter().map(|s| s.to_string()).collect()),
        )
    }
}

impl PropertyConverter for StringConverter {
    fn is_supported(&self, rule: &CssRule) -> bool {
        rule.property == self.property
    }

    fn convert(&self, ctx: &mut ConversionContext, rule: &CssRule) -> bool {
        let Some(value) = &rule.value else {
            ctx.set_prop(&rule.property, None);
            return true;
        };

        let v = value.trim();
        if v.is_empty() {
            return false;
        }

        if let Some(ref allowed) = self.allowed_values {
            if !allowed.iter().any(|a| a == v) {
                return false;
            }
        }

        ctx.set_prop(&rule.property, Some(string_prop(v)));
        true
    }
}
