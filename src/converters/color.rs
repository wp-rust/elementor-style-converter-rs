use super::PropertyConverter;
use crate::context::ConversionContext;
use crate::types::{color_prop, CssRule};
use crate::value_parsers::CssTokenSplitter;

/// Converts a single color value to a `color` PropValue.
/// Declines multi-token values (e.g. `border-color: red green blue`).
pub struct ColorConverter {
    property: String,
}

impl ColorConverter {
    pub fn new(property: impl Into<String>) -> Self {
        Self { property: property.into() }
    }
}

impl PropertyConverter for ColorConverter {
    fn is_supported(&self, rule: &CssRule) -> bool {
        rule.property == self.property
    }

    fn convert(&self, ctx: &mut ConversionContext, rule: &CssRule) -> bool {
        let Some(value) = &rule.value else {
            ctx.set_prop(&rule.property, None);
            return true;
        };

        let color = value.trim();
        // A single color must be one token (paren-aware, so rgb(255 0 0) is fine)
        if CssTokenSplitter::split_by_whitespace(color).len() != 1 {
            return false;
        }

        ctx.set_prop(&rule.property, Some(color_prop(color)));
        true
    }
}
