use super::PropertyConverter;
use crate::context::ConversionContext;
use crate::types::*;
use crate::value_parsers::{CssTokenSplitter, SizeValueParser};
use serde_json::Value;

const INSET: &str = "inset";
const DEFAULT_COLOR: &str = "currentColor";

pub struct BoxShadowConverter;

impl PropertyConverter for BoxShadowConverter {
    fn is_supported(&self, rule: &CssRule) -> bool {
        rule.property == "box-shadow"
    }

    fn convert(&self, ctx: &mut ConversionContext, rule: &CssRule) -> bool {
        let Some(value) = &rule.value else {
            ctx.set_prop("box-shadow", None);
            return true;
        };

        let value = value.trim();
        if value.is_empty() {
            return false;
        }

        if value.eq_ignore_ascii_case("none") {
            ctx.set_prop("box-shadow", Some(box_shadow_prop(vec![])));
            return true;
        }

        let layers = CssTokenSplitter::split_by_comma(value);
        let mut shadows = Vec::new();

        for layer in layers {
            let Some(shadow) = parse_layer(layer.trim()) else {
                return false;
            };
            shadows.push(shadow);
        }

        ctx.set_prop("box-shadow", Some(box_shadow_prop(shadows)));
        true
    }
}

fn parse_layer(layer: &str) -> Option<Value> {
    let tokens = CssTokenSplitter::split_by_whitespace(layer);
    if tokens.is_empty() {
        return None;
    }

    let mut lengths = Vec::new();
    let mut color: Option<String> = None;
    let mut is_inset = false;

    for token in &tokens {
        if token.eq_ignore_ascii_case(INSET) {
            if is_inset { return None; } // duplicate inset
            is_inset = true;
            continue;
        }

        if let Some(size) = SizeValueParser::parse(token, false) {
            lengths.push(size);
            continue;
        }

        if color.is_some() {
            return None; // two color tokens
        }
        color = Some(token.clone());
    }

    if lengths.len() < 2 || lengths.len() > 4 {
        return None;
    }

    let zero = SizeLeaf { size: SizeValue::Number(0.0), unit: UNIT_PX.to_string() };
    let blur   = lengths.get(2).cloned().unwrap_or_else(|| zero.clone());
    let spread = lengths.get(3).cloned().unwrap_or_else(|| zero.clone());

    let color_val = color.unwrap_or_else(|| DEFAULT_COLOR.to_string());

    let mut fields = ShadowFields {
        h_offset: size_to_val(&lengths[0]),
        v_offset: size_to_val(&lengths[1]),
        blur:     size_to_val(&blur),
        spread:   size_to_val(&spread),
        color:    serde_json::json!({"$$type": COLOR_KEY, "value": color_val}),
        position: None,
    };

    if is_inset {
        fields.position = Some(serde_json::json!({"$$type": STRING_KEY, "value": INSET}));
    }

    Some(shadow_value(fields))
}

fn size_to_val(s: &SizeLeaf) -> Value {
    serde_json::json!({
        "$$type": SIZE_KEY,
        "value": {"size": s.size, "unit": s.unit}
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ConversionContext;

    fn convert(css: &str) -> Option<ConversionContext> {
        let mut ctx = ConversionContext::default();
        let rule = CssRule {
            property: "box-shadow".to_string(),
            value: Some(css.to_string()),
            declaration: format!("box-shadow: {}", css),
        };
        let conv = BoxShadowConverter;
        if conv.convert(&mut ctx, &rule) {
            Some(ctx)
        } else {
            None
        }
    }

    #[test]
    fn simple_shadow() {
        assert!(convert("2px 4px 6px rgba(0,0,0,0.3)").is_some());
    }

    #[test]
    fn inset() {
        assert!(convert("inset 1px 2px 3px red").is_some());
    }

    #[test]
    fn none_clears() {
        let ctx = convert("none").unwrap();
        let v = ctx.get_prop("box-shadow").unwrap();
        // should be box-shadow with empty array
        assert!(v.is_some());
    }

    #[test]
    fn only_two_lengths() {
        assert!(convert("1px 2px").is_some());
    }

    #[test]
    fn one_length_decline() {
        assert!(convert("1px").is_none());
    }
}
