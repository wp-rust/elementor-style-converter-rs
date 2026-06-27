use super::PropertyConverter;
use crate::context::ConversionContext;
use crate::types::{span_prop, CssRule};
use regex::Regex;

pub struct SpanConverter {
    property: String,
    pattern: Option<Regex>,
}

impl SpanConverter {
    pub fn new(property: impl Into<String>, pattern: Option<&str>) -> Self {
        Self {
            property: property.into(),
            pattern: pattern.map(|p| Regex::new(p).expect("invalid span pattern")),
        }
    }
}

impl PropertyConverter for SpanConverter {
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

        if let Some(ref pat) = self.pattern {
            if !pat.is_match(v) {
                return false;
            }
        }

        ctx.set_prop(&rule.property, Some(span_prop(v)));
        true
    }
}
