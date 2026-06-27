use super::PropertyConverter;
use crate::context::ConversionContext;
use crate::types::*;
use crate::value_parsers::{CssTokenSplitter, SizeValueParser};

const AUTO_BASIS: fn() -> SizeLeaf = || SizeLeaf {
    size: SizeValue::Raw("auto".to_string()),
    unit: UNIT_CUSTOM.to_string(),
};
const ZERO_BASIS: fn() -> SizeLeaf = || SizeLeaf {
    size: SizeValue::Number(0.0),
    unit: UNIT_PX.to_string(),
};

pub struct FlexConverter;

impl PropertyConverter for FlexConverter {
    fn is_supported(&self, rule: &CssRule) -> bool {
        rule.property == "flex"
    }

    fn convert(&self, ctx: &mut ConversionContext, rule: &CssRule) -> bool {
        let Some(value) = &rule.value else {
            ctx.set_prop("flex", None);
            return true;
        };

        let value = value.trim();

        let Some((grow, shrink, basis)) = parse_flex(value) else {
            return false;
        };

        let prop = PropValue::tagged(
            FLEX_KEY,
            serde_json::json!({
                "flexGrow":   {"$$type": NUMBER_KEY, "value": grow},
                "flexShrink": {"$$type": NUMBER_KEY, "value": shrink},
                "flexBasis":  {"$$type": SIZE_KEY,   "value": {"size": basis.size, "unit": basis.unit}},
            }),
        );

        ctx.set_prop("flex", Some(prop));
        true
    }
}

fn parse_flex(value: &str) -> Option<(f64, f64, SizeLeaf)> {
    let lower = value.to_lowercase();

    if lower == "none" {
        return Some((0.0, 1.0, AUTO_BASIS()));
    }
    if lower == "auto" {
        return Some((1.0, 1.0, AUTO_BASIS()));
    }

    let tokens = CssTokenSplitter::split_by_whitespace(value);

    match tokens.len() {
        1 => {
            let grow = tokens[0].parse::<f64>().ok()?;
            Some((grow, 1.0, ZERO_BASIS()))
        }
        2 => {
            let grow = tokens[0].parse::<f64>().ok()?;
            let basis = SizeValueParser::parse(&tokens[1], false)?;
            Some((grow, 1.0, basis))
        }
        3 => {
            let grow   = tokens[0].parse::<f64>().ok()?;
            let shrink = tokens[1].parse::<f64>().ok()?;
            let basis  = parse_basis(&tokens[2])?;
            Some((grow, shrink, basis))
        }
        _ => None,
    }
}

fn parse_basis(token: &str) -> Option<SizeLeaf> {
    if token.eq_ignore_ascii_case("auto") {
        return Some(AUTO_BASIS());
    }
    SizeValueParser::parse(token, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ConversionContext;

    fn run(css: &str) -> bool {
        let mut ctx = ConversionContext::default();
        let rule = CssRule {
            property: "flex".to_string(),
            value: Some(css.to_string()),
            declaration: format!("flex: {}", css),
        };
        FlexConverter.convert(&mut ctx, &rule)
    }

    #[test]
    fn none() { assert!(run("none")); }
    #[test]
    fn auto() { assert!(run("auto")); }
    #[test]
    fn single_grow() { assert!(run("2")); }
    #[test]
    fn grow_and_basis() { assert!(run("1 200px")); }
    #[test]
    fn full_three_token() { assert!(run("1 1 auto")); }
    #[test]
    fn invalid_decline() { assert!(!run("abc")); }
}
