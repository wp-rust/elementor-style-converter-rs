use crate::context::ConversionContext;
use crate::registry::{ConverterRegistry, ExpanderRegistry};
use crate::types::{ConversionResult, CssRule, PropValue};

const BLOCKED_PROPERTIES: &[&str] = &["behavior", "-moz-binding"];
const BLOCKED_VALUE_NEEDLES: &[&str] = &["expression(", "javascript:"];

/// Main CSS→PropValue converter. Mirrors PHP `Css_Converter`.
///
/// Pipeline:
/// 1. Parse CSS string into `CssRule` list (split on `;`, then on first `:`)
/// 2. Expand shorthands via `ExpanderRegistry`
/// 3. Deduplicate (last declaration for a property wins)
/// 4. Run each rule through `ConverterRegistry` (first converter that claims it wins)
/// 5. Leftover rules go to `custom_css`
pub struct CssConverter {
    registry: ConverterRegistry,
    expanders: ExpanderRegistry,
}

impl CssConverter {
    pub fn new(registry: ConverterRegistry, expanders: ExpanderRegistry) -> Self {
        Self { registry, expanders }
    }

    pub fn convert(&self, css: &str) -> ConversionResult {
        let parsed = self.parse(css);
        let expanded = self.expand_shorthands(parsed);
        let rules = self.dedupe(expanded);

        let mut ctx = ConversionContext::new(rules.clone());
        let mut leftover: Vec<String> = Vec::new();

        for rule in &rules {
            if !self.try_convert(&mut ctx, rule) {
                leftover.push(format!("{};", rule.declaration));
            }
        }

        let (props, rejected) = ctx.into_parts();

        ConversionResult {
            props,
            custom_css: leftover.join(" "),
            rejected,
        }
    }

    fn try_convert(&self, ctx: &mut ConversionContext, rule: &CssRule) -> bool {
        for converter in self.registry.all() {
            if !converter.is_supported(rule) {
                continue;
            }
            if converter.convert(ctx, rule) {
                return true;
            }
        }
        false
    }

    fn expand_shorthands(&self, rules: Vec<CssRule>) -> Vec<CssRule> {
        let mut expanded = Vec::new();
        for rule in rules {
            let mut handled = false;
            for expander in self.expanders.all() {
                if expander.is_supported(&rule) {
                    let result = expander.expand(&rule);
                    if !result.is_empty() {
                        expanded.extend(result);
                    } else {
                        expanded.push(rule.clone());
                    }
                    handled = true;
                    break;
                }
            }
            if !handled {
                expanded.push(rule);
            }
        }
        expanded
    }

    /// Last-writer-wins deduplication: keep only the last rule for each property.
    fn dedupe(&self, rules: Vec<CssRule>) -> Vec<CssRule> {
        let mut last_index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for (i, rule) in rules.iter().enumerate() {
            last_index.insert(rule.property.clone(), i);
        }
        rules
            .into_iter()
            .enumerate()
            .filter(|(i, rule)| last_index.get(&rule.property) == Some(i))
            .map(|(_, rule)| rule)
            .collect()
    }

    /// Parse a CSS string into rules. Splits on `;`, then on first `:`.
    fn parse(&self, css: &str) -> Vec<CssRule> {
        let mut rules = Vec::new();

        for declaration in css.split(';') {
            let declaration = declaration.trim();
            if declaration.is_empty() {
                continue;
            }

            let Some(sep) = declaration.find(':') else {
                continue;
            };

            let property = declaration[..sep].trim().to_lowercase();
            let raw_value = declaration[sep + 1..].trim().to_string();

            if property.is_empty() || raw_value.is_empty() {
                continue;
            }

            if self.is_blocked(&property, &raw_value) {
                continue;
            }

            let value = if raw_value == "null" {
                None
            } else {
                Some(raw_value)
            };

            rules.push(CssRule {
                property,
                value,
                declaration: declaration.to_string(),
            });
        }

        rules
    }

    fn is_blocked(&self, property: &str, value: &str) -> bool {
        if BLOCKED_PROPERTIES.contains(&property) {
            return true;
        }
        let lower = value.to_lowercase();
        BLOCKED_VALUE_NEEDLES.iter().any(|needle| lower.contains(needle))
    }
}

// ─── ConversionResult adjustment ─────────────────────────────────────────────

impl ConversionResult {
    /// Flatten `HashMap<String, Option<PropValue>>` to `HashMap<String, PropValue>`.
    /// `None` entries represent explicit null-resets and are kept as `PropValue::Null`.
    pub fn flat_props(&self) -> std::collections::HashMap<&str, &PropValue> {
        self.props
            .iter()
            .filter_map(|(k, v)| v.as_ref().map(|pv| (k.as_str(), pv)))
            .collect()
    }
}
