use super::PropertyConverter;
use crate::context::ConversionContext;
use crate::types::{CssRule, PropValue};
use crate::value_parsers::filter_value_parser::FilterValueParser;

pub struct FilterConverter {
    property: String,
    type_key: String,
}

impl FilterConverter {
    pub fn new(property: impl Into<String>, type_key: impl Into<String>) -> Self {
        Self {
            property: property.into(),
            type_key: type_key.into(),
        }
    }
}

impl PropertyConverter for FilterConverter {
    fn is_supported(&self, rule: &CssRule) -> bool {
        rule.property == self.property
    }

    fn convert(&self, ctx: &mut ConversionContext, rule: &CssRule) -> bool {
        let Some(value) = &rule.value else {
            ctx.set_prop(&self.property, None);
            return true;
        };

        let Some(items) = FilterValueParser::parse(value) else {
            return false;
        };

        ctx.set_prop(
            &self.property,
            Some(PropValue::tagged(
                &self.type_key,
                serde_json::Value::Array(items),
            )),
        );
        true
    }
}
