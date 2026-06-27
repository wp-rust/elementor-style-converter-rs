use super::PropertyConverter;
use crate::context::ConversionContext;
use crate::types::*;
use crate::value_parsers::{BoxShorthandParser, BoxParseResult};
use serde_json::Value;

/// Converts padding/margin shorthands to `dimensions` PropValue,
/// or border-width to `border-width` PropValue.
/// Single value → Size; 2-4 values → logical side object.
pub struct DimensionsConverter {
    property: String,
    type_key: String,
}

impl DimensionsConverter {
    pub fn dimensions(property: impl Into<String>) -> Self {
        Self {
            property: property.into(),
            type_key: DIMENSIONS_KEY.to_string(),
        }
    }

    pub fn border_width(property: impl Into<String>) -> Self {
        Self {
            property: property.into(),
            type_key: BORDER_WIDTH_KEY.to_string(),
        }
    }
}

impl PropertyConverter for DimensionsConverter {
    fn is_supported(&self, rule: &CssRule) -> bool {
        rule.property == self.property
    }

    fn convert(&self, ctx: &mut ConversionContext, rule: &CssRule) -> bool {
        let Some(value) = &rule.value else {
            ctx.set_prop(&rule.property, None);
            return true;
        };

        let Some(parsed) = BoxShorthandParser::parse(value) else {
            return false;
        };

        let prop = match parsed {
            BoxParseResult::Single(s) => size_prop(s),
            BoxParseResult::Sides([top, right, bottom, left]) => {
                let sides = DimensionSides {
                    block_start: size_leaf_to_value(top),
                    inline_end: size_leaf_to_value(right),
                    block_end: size_leaf_to_value(bottom),
                    inline_start: size_leaf_to_value(left),
                };
                if self.type_key == BORDER_WIDTH_KEY {
                    border_width_prop(sides)
                } else {
                    dimensions_prop(sides)
                }
            }
        };

        ctx.set_prop(&self.property, Some(prop));
        true
    }
}

pub fn size_leaf_to_value(leaf: SizeLeaf) -> Value {
    serde_json::json!({
        "$$type": SIZE_KEY,
        "value": {"size": leaf.size, "unit": leaf.unit}
    })
}
