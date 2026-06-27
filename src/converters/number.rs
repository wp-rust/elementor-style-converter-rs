use super::PropertyConverter;
use crate::context::ConversionContext;
use crate::types::{number_prop, CssRule};

pub struct NumberConverter {
    property: String,
}

impl NumberConverter {
    pub fn new(property: impl Into<String>) -> Self {
        Self { property: property.into() }
    }
}

impl PropertyConverter for NumberConverter {
    fn is_supported(&self, rule: &CssRule) -> bool {
        rule.property == self.property
    }

    fn convert(&self, ctx: &mut ConversionContext, rule: &CssRule) -> bool {
        let Some(value) = &rule.value else {
            ctx.set_prop(&rule.property, None);
            return true;
        };

        let Ok(n) = value.trim().parse::<f64>() else {
            return false;
        };

        ctx.set_prop(&rule.property, Some(number_prop(n)));
        true
    }
}
