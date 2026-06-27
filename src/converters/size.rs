use super::PropertyConverter;
use crate::context::ConversionContext;
use crate::types::{size_prop, CssRule};
use crate::value_parsers::SizeValueParser;

pub struct SizeConverter {
    property: String,
    allow_unitless: bool,
}

impl SizeConverter {
    pub fn new(property: impl Into<String>, allow_unitless: bool) -> Self {
        Self {
            property: property.into(),
            allow_unitless,
        }
    }
}

impl PropertyConverter for SizeConverter {
    fn is_supported(&self, rule: &CssRule) -> bool {
        rule.property == self.property
    }

    fn convert(&self, ctx: &mut ConversionContext, rule: &CssRule) -> bool {
        let Some(value) = &rule.value else {
            ctx.set_prop(&rule.property, None);
            return true;
        };

        let Some(parsed) = SizeValueParser::parse(value, self.allow_unitless) else {
            return false;
        };

        ctx.set_prop(&rule.property, Some(size_prop(parsed)));
        true
    }
}
