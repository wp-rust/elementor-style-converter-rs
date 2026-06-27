use crate::types::{CssRule, PropValue};
use std::collections::HashMap;

/// Shared mutable state passed through the converter loop.
/// Mirrors PHP `Conversion_Context`.
#[derive(Default)]
pub struct ConversionContext {
    props: HashMap<String, Option<PropValue>>,
    rejected: Vec<String>,
    rules: Vec<CssRule>,
}

impl ConversionContext {
    pub fn new(rules: Vec<CssRule>) -> Self {
        Self {
            rules,
            ..Default::default()
        }
    }

    pub fn set_prop(&mut self, property: &str, value: Option<PropValue>) {
        self.props.insert(property.to_string(), value);
    }

    pub fn get_prop(&self, property: &str) -> Option<&Option<PropValue>> {
        self.props.get(property)
    }

    pub fn has_prop(&self, property: &str) -> bool {
        self.props.contains_key(property)
    }

    pub fn reject(&mut self, declaration: String) {
        self.rejected.push(declaration);
    }

    pub fn into_parts(self) -> (HashMap<String, Option<PropValue>>, Vec<String>) {
        (self.props, self.rejected)
    }

    pub fn rules(&self) -> &[CssRule] {
        &self.rules
    }
}
