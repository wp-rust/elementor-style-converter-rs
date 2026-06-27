pub mod border;
pub mod outline;
pub mod physical_to_logical;

use crate::types::CssRule;

/// Pre-processing pass: rewrite a shorthand into longhand declarations.
/// Mirrors PHP `Shorthand_Expander`.
pub trait ShorthandExpander: Send + Sync {
    fn is_supported(&self, rule: &CssRule) -> bool;
    /// Returns expanded rules, or an empty Vec to decline (shorthand kept as-is → custom_css).
    fn expand(&self, rule: &CssRule) -> Vec<CssRule>;
}

/// Base helper: builds a null-reset rule.
pub fn null_rule(property: &str) -> CssRule {
    CssRule {
        property: property.to_string(),
        value: None,
        declaration: format!("{}: ", property),
    }
}
