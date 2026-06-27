use crate::types::SizeLeaf;
use crate::value_parsers::{CssTokenSplitter, SizeValueParser};

/// Result of parsing a CSS box shorthand (padding, margin, border-radius, border-width).
pub enum BoxParseResult {
    /// Single token — emit as a Size (the union's Size member).
    Single(SizeLeaf),
    /// 2–4 tokens expanded via CSS box rule: [top/start-start, right/start-end, bottom/end-end, left/end-start].
    Sides([SizeLeaf; 4]),
}

/// Stateless parser for CSS box shorthands.
/// Paren-aware tokenizer preserves calc() and function values as single tokens.
/// Returns `None` for 0 tokens, >4 tokens, or any token that doesn't parse as a Size.
pub struct BoxShorthandParser;

impl BoxShorthandParser {
    pub fn parse(value: &str) -> Option<BoxParseResult> {
        let tokens = CssTokenSplitter::split_by_whitespace(value.trim());
        let count = tokens.len();

        if count == 0 || count > 4 {
            return None;
        }

        let sizes: Vec<SizeLeaf> = tokens
            .iter()
            .map(|t| SizeValueParser::parse(t, false))
            .collect::<Option<Vec<_>>>()?;

        if count == 1 {
            return Some(BoxParseResult::Single(sizes.into_iter().next().unwrap()));
        }

        Some(BoxParseResult::Sides(Self::expand_box(sizes)))
    }

    /// Expand 2–4 values onto four sides via the CSS box rule.
    fn expand_box(sizes: Vec<SizeLeaf>) -> [SizeLeaf; 4] {
        match sizes.len() {
            2 => [
                sizes[0].clone(),
                sizes[1].clone(),
                sizes[0].clone(),
                sizes[1].clone(),
            ],
            3 => [
                sizes[0].clone(),
                sizes[1].clone(),
                sizes[2].clone(),
                sizes[1].clone(),
            ],
            _ => [
                sizes[0].clone(),
                sizes[1].clone(),
                sizes[2].clone(),
                sizes[3].clone(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_value() {
        let r = BoxShorthandParser::parse("10px").unwrap();
        assert!(matches!(r, BoxParseResult::Single(_)));
    }

    #[test]
    fn two_values_expand() {
        let sides = match BoxShorthandParser::parse("10px 20px").unwrap() {
            BoxParseResult::Sides(s) => s,
            _ => panic!("expected sides"),
        };
        // top=10, right=20, bottom=10, left=20
        assert_eq!(sides[0].unit, "px");
        assert_eq!(sides[2].unit, "px");
    }

    #[test]
    fn four_values() {
        let r = BoxShorthandParser::parse("1px 2px 3px 4px");
        assert!(r.is_some());
    }

    #[test]
    fn five_values_decline() {
        assert!(BoxShorthandParser::parse("1px 2px 3px 4px 5px").is_none());
    }

    #[test]
    fn invalid_token_decline() {
        assert!(BoxShorthandParser::parse("10px abc").is_none());
    }

    #[test]
    fn calc_as_single() {
        let r = BoxShorthandParser::parse("calc(100% / 4)").unwrap();
        assert!(matches!(r, BoxParseResult::Single(_)));
    }
}
