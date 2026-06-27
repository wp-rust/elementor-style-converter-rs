use super::{null_rule, ShorthandExpander};
use crate::types::CssRule;
use crate::value_parsers::{CssTokenSplitter, SizeValueParser};

const STYLE_KEYWORDS: &[&str] = &[
    "none", "auto", "dotted", "dashed", "solid", "double",
    "groove", "ridge", "inset", "outset",
];
const WIDTH_KEYWORDS: &[&str] = &["thin", "medium", "thick"];

pub struct OutlineShorthandExpander;

impl ShorthandExpander for OutlineShorthandExpander {
    fn is_supported(&self, rule: &CssRule) -> bool {
        rule.property == "outline"
    }

    fn expand(&self, rule: &CssRule) -> Vec<CssRule> {
        if rule.value.is_none() {
            return vec![
                null_rule("outline-width"),
                null_rule("outline-style"),
                null_rule("outline-color"),
                null_rule("outline-offset"),
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

            if STYLE_KEYWORDS.contains(&lower.as_str()) {
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
            rules.push(CssRule { property: "outline-width".to_string(), value: Some(w.clone()), declaration: format!("outline-width: {}", w) });
        }
        if let Some(s) = style {
            rules.push(CssRule { property: "outline-style".to_string(), value: Some(s.clone()), declaration: format!("outline-style: {}", s) });
        }
        if let Some(c) = color {
            rules.push(CssRule { property: "outline-color".to_string(), value: Some(c.clone()), declaration: format!("outline-color: {}", c) });
        }

        rules
    }
}
