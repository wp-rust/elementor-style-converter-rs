use super::{null_rule, ShorthandExpander};
use crate::types::CssRule;

const MAP: &[(&str, &str)] = &[
    ("top",    "inset-block-start"),
    ("right",  "inset-inline-end"),
    ("bottom", "inset-block-end"),
    ("left",   "inset-inline-start"),
];

/// Rewrites `top`/`right`/`bottom`/`left` to their logical equivalents
/// so existing Size converters can handle them.
pub struct PhysicalToLogicalExpander;

impl ShorthandExpander for PhysicalToLogicalExpander {
    fn is_supported(&self, rule: &CssRule) -> bool {
        MAP.iter().any(|(phys, _)| rule.property == *phys)
    }

    fn expand(&self, rule: &CssRule) -> Vec<CssRule> {
        let logical = MAP
            .iter()
            .find(|(phys, _)| rule.property == *phys)
            .map(|(_, log)| *log)
            .unwrap_or(&rule.property);

        if rule.value.is_none() {
            return vec![null_rule(logical)];
        }

        let value = rule.value.clone().unwrap();
        vec![CssRule {
            property: logical.to_string(),
            declaration: format!("{}: {}", logical, value),
            value: Some(value),
        }]
    }
}
