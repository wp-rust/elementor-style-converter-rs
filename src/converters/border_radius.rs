use super::PropertyConverter;
use crate::context::ConversionContext;
use crate::types::*;
use crate::value_parsers::{BoxShorthandParser, BoxParseResult};
use crate::converters::dimensions::size_leaf_to_value;

pub struct BorderRadiusConverter {
    property: String,
}

impl BorderRadiusConverter {
    pub fn new(property: impl Into<String>) -> Self {
        Self { property: property.into() }
    }
}

impl PropertyConverter for BorderRadiusConverter {
    fn is_supported(&self, rule: &CssRule) -> bool {
        rule.property == self.property
    }

    fn convert(&self, ctx: &mut ConversionContext, rule: &CssRule) -> bool {
        let Some(value) = &rule.value else {
            ctx.set_prop(&rule.property, None);
            return true;
        };

        // The box parser's paren-awareness handles calc() division but rejects "/" separator
        // (which is the elliptical border-radius form) — it just fails to parse as a size leaf.
        let Some(parsed) = BoxShorthandParser::parse(value) else {
            return false;
        };

        let prop = match parsed {
            BoxParseResult::Single(s) => size_prop(s),
            BoxParseResult::Sides([tl, tr, br, bl]) => {
                let corners = BorderRadiusCorners {
                    start_start: size_leaf_to_value(tl),
                    start_end: size_leaf_to_value(tr),
                    end_end: size_leaf_to_value(br),
                    end_start: size_leaf_to_value(bl),
                };
                border_radius_prop(corners)
            }
        };

        ctx.set_prop(&self.property, Some(prop));
        true
    }
}
