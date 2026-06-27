use super::{null_rule, ShorthandExpander};
use crate::types::CssRule;
use crate::value_parsers::{CssTokenSplitter, SizeValueParser};

const WIDTH_KEYWORDS: &[&str] = &["thin", "medium", "thick"];

/// Expands `border` / `border-{side}` shorthands into width / style / color longhands.
pub struct BorderShorthandExpander {
    property: String,
    longhands: BorderLonghands,
    style_keywords: Vec<String>,
}

pub struct BorderLonghands {
    pub width: String,
    pub style: String,
    pub color: String,
}

impl BorderShorthandExpander {
    /// All-sides `border` shorthand.
    pub fn all_sides() -> Self {
        Self {
            property: "border".to_string(),
            longhands: BorderLonghands {
                width: "border-width".to_string(),
                style: "border-style".to_string(),
                color: "border-color".to_string(),
            },
            style_keywords: default_border_style_keywords(),
        }
    }

    /// Per-side shorthand, e.g. `border-top`.
    pub fn side(side: &str) -> Self {
        Self {
            property: format!("border-{}", side),
            longhands: BorderLonghands {
                width: format!("border-{}-width", side),
                style: format!("border-{}-style", side),
                color: format!("border-{}-color", side),
            },
            style_keywords: default_border_style_keywords(),
        }
    }
}

fn default_border_style_keywords() -> Vec<String> {
    vec![
        "none", "hidden", "dotted", "dashed", "solid", "double",
        "groove", "ridge", "inset", "outset",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}

impl ShorthandExpander for BorderShorthandExpander {
    fn is_supported(&self, rule: &CssRule) -> bool {
        rule.property == self.property
    }

    fn expand(&self, rule: &CssRule) -> Vec<CssRule> {
        if rule.value.is_none() {
            return vec![
                null_rule(&self.longhands.width),
                null_rule(&self.longhands.style),
                null_rule(&self.longhands.color),
            ];
        }

        let value = rule.value.as_deref().unwrap_or("").trim();
        let tokens = CssTokenSplitter::split_by_whitespace(value);

        if tokens.is_empty() {
            return vec![];
        }

        let mut width: Option<String> = None;
        let mut style: Option<String> = None;
        let mut color: Option<String> = None;

        for token in &tokens {
            let lower = token.to_lowercase();

            if self.style_keywords.contains(&lower) {
                if style.is_some() { return vec![]; }
                style = Some(token.clone());
            } else if WIDTH_KEYWORDS.contains(&lower.as_str())
                || SizeValueParser::parse(token, false).is_some()
            {
                if width.is_some() { return vec![]; }
                width = Some(token.clone());
            } else {
                if color.is_some() { return vec![]; }
                color = Some(token.clone());
            }
        }

        let mut rules = Vec::new();

        if let Some(w) = width {
            let prop = self.longhands.width.clone();
            rules.push(CssRule {
                declaration: format!("{}: {}", prop, w),
                property: prop,
                value: Some(w),
            });
        }
        if let Some(s) = style {
            let prop = self.longhands.style.clone();
            rules.push(CssRule {
                declaration: format!("{}: {}", prop, s),
                property: prop,
                value: Some(s),
            });
        }
        if let Some(c) = color {
            let prop = self.longhands.color.clone();
            rules.push(CssRule {
                declaration: format!("{}: {}", prop, c),
                property: prop,
                value: Some(c),
            });
        }

        rules
    }
}
