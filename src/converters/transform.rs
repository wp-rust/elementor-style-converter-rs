use super::PropertyConverter;
use crate::context::ConversionContext;
use crate::types::*;
use crate::value_parsers::{CssTokenSplitter, SizeValueParser};
use serde_json::Value;

const MOVE_UNITS: &[&str] = &["%", "px", "em", "rem", "vw", UNIT_CUSTOM];
const ROTATE_UNITS: &[&str] = &["deg", "rad", "grad", "turn", UNIT_CUSTOM];
const ZERO_MOVE: fn() -> SizeLeaf = || SizeLeaf { size: SizeValue::Number(0.0), unit: "px".to_string() };
const ZERO_ROTATE: fn() -> SizeLeaf = || SizeLeaf { size: SizeValue::Number(0.0), unit: "deg".to_string() };

pub struct TransformConverter;

impl PropertyConverter for TransformConverter {
    fn is_supported(&self, rule: &CssRule) -> bool {
        rule.property == "transform"
    }

    fn convert(&self, ctx: &mut ConversionContext, rule: &CssRule) -> bool {
        let Some(value) = &rule.value else {
            ctx.set_prop("transform", None);
            return true;
        };

        let Some(functions) = parse_functions(value.trim()) else {
            return false;
        };

        // Merge into existing transform prop if present
        let existing_fields = ctx
            .get_prop("transform")
            .and_then(|p| p.as_ref())
            .and_then(|p| {
                if let PropValue::Tagged(tv) = p {
                    tv.value.as_object().cloned()
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let mut fields = existing_fields;
        fields.insert(
            "transform-functions".to_string(),
            serde_json::json!({
                "$$type": TRANSFORM_FUNCTIONS_KEY,
                "value": functions
            }),
        );

        ctx.set_prop(
            "transform",
            Some(PropValue::tagged(TRANSFORM_KEY, Value::Object(fields))),
        );
        true
    }
}

fn parse_functions(value: &str) -> Option<Vec<Value>> {
    if value.is_empty() || value.eq_ignore_ascii_case("none") {
        return Some(vec![]);
    }

    let raw = split_functions(value)?;
    raw.into_iter().map(|s| parse_fn(&s)).collect()
}

fn split_functions(value: &str) -> Option<Vec<String>> {
    let mut functions = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;
    let chars: Vec<char> = value.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                current.push(ch);
                functions.push(current.trim().to_string());
                current.clear();
                i += 1;
                while i < len && chars[i] == ' ' {
                    i += 1;
                }
                continue;
            }
        }
        current.push(ch);
        i += 1;
    }

    if !current.trim().is_empty() || depth != 0 {
        return None;
    }

    Some(functions)
}

fn parse_fn(callback: &str) -> Option<Value> {
    let paren = callback.find('(')?;
    let name = callback[..paren].trim().to_lowercase();
    let inner = &callback[paren + 1..callback.len() - 1]; // strip parens
    let args: Vec<String> = CssTokenSplitter::split_by_comma(inner)
        .into_iter()
        .map(|s| s.trim().to_string())
        .collect();

    match name.as_str() {
        "translate"  => parse_translate(&args),
        "translatex" => parse_translate_axis(&args, 'x'),
        "translatey" => parse_translate_axis(&args, 'y'),
        "translatez" => parse_translate_axis(&args, 'z'),
        "translate3d" => parse_translate_3d(&args),
        "scale"      => parse_scale(&args),
        "scalex"     => parse_scale_axis(&args, 'x'),
        "scaley"     => parse_scale_axis(&args, 'y'),
        "scalez"     => parse_scale_axis(&args, 'z'),
        "scale3d"    => parse_scale_3d(&args),
        "rotate" | "rotatez" => parse_rotate_axis(&args, 'z'),
        "rotatex"    => parse_rotate_axis(&args, 'x'),
        "rotatey"    => parse_rotate_axis(&args, 'y'),
        _            => None,
    }
}

fn move_size(token: &str) -> Option<SizeLeaf> {
    let s = SizeValueParser::parse(token, false)?;
    if MOVE_UNITS.contains(&s.unit.as_str()) { Some(s) } else { None }
}

fn rotate_size(token: &str) -> Option<SizeLeaf> {
    let s = SizeValueParser::parse(token, false)?;
    if ROTATE_UNITS.contains(&s.unit.as_str()) { Some(s) } else { None }
}

fn scale_num(token: &str) -> Option<f64> {
    token.parse::<f64>().ok()
}

fn size_val(s: SizeLeaf) -> Value {
    serde_json::json!({"$$type": SIZE_KEY, "value": {"size": s.size, "unit": s.unit}})
}
fn num_val(n: f64) -> Value { serde_json::json!({"$$type": NUMBER_KEY, "value": n}) }

fn move_item(x: SizeLeaf, y: SizeLeaf, z: SizeLeaf) -> Value {
    serde_json::json!({
        "$$type": TRANSFORM_MOVE_KEY,
        "value": {"x": size_val(x), "y": size_val(y), "z": size_val(z)}
    })
}
fn scale_item(x: f64, y: f64, z: f64) -> Value {
    serde_json::json!({
        "$$type": TRANSFORM_SCALE_KEY,
        "value": {"x": num_val(x), "y": num_val(y), "z": num_val(z)}
    })
}
fn rotate_item(x: SizeLeaf, y: SizeLeaf, z: SizeLeaf) -> Value {
    serde_json::json!({
        "$$type": TRANSFORM_ROTATE_KEY,
        "value": {"x": size_val(x), "y": size_val(y), "z": size_val(z)}
    })
}

fn parse_translate(args: &[String]) -> Option<Value> {
    match args.len() {
        1 => Some(move_item(move_size(&args[0])?, ZERO_MOVE(), ZERO_MOVE())),
        2 => Some(move_item(move_size(&args[0])?, move_size(&args[1])?, ZERO_MOVE())),
        _ => None,
    }
}
fn parse_translate_axis(args: &[String], axis: char) -> Option<Value> {
    if args.len() != 1 { return None; }
    let v = move_size(&args[0])?;
    Some(match axis {
        'x' => move_item(v, ZERO_MOVE(), ZERO_MOVE()),
        'y' => move_item(ZERO_MOVE(), v, ZERO_MOVE()),
        _   => move_item(ZERO_MOVE(), ZERO_MOVE(), v),
    })
}
fn parse_translate_3d(args: &[String]) -> Option<Value> {
    if args.len() != 3 { return None; }
    Some(move_item(move_size(&args[0])?, move_size(&args[1])?, move_size(&args[2])?))
}
fn parse_scale(args: &[String]) -> Option<Value> {
    match args.len() {
        1 => { let n = scale_num(&args[0])?; Some(scale_item(n, n, 1.0)) }
        2 => Some(scale_item(scale_num(&args[0])?, scale_num(&args[1])?, 1.0)),
        _ => None,
    }
}
fn parse_scale_axis(args: &[String], axis: char) -> Option<Value> {
    if args.len() != 1 { return None; }
    let n = scale_num(&args[0])?;
    Some(match axis {
        'x' => scale_item(n, 1.0, 1.0),
        'y' => scale_item(1.0, n, 1.0),
        _   => scale_item(1.0, 1.0, n),
    })
}
fn parse_scale_3d(args: &[String]) -> Option<Value> {
    if args.len() != 3 { return None; }
    Some(scale_item(scale_num(&args[0])?, scale_num(&args[1])?, scale_num(&args[2])?))
}
fn parse_rotate_axis(args: &[String], axis: char) -> Option<Value> {
    if args.len() != 1 { return None; }
    let v = rotate_size(&args[0])?;
    Some(match axis {
        'x' => rotate_item(v, ZERO_ROTATE(), ZERO_ROTATE()),
        'y' => rotate_item(ZERO_ROTATE(), v, ZERO_ROTATE()),
        _   => rotate_item(ZERO_ROTATE(), ZERO_ROTATE(), v),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ConversionContext;

    fn run(css: &str) -> bool {
        let mut ctx = ConversionContext::default();
        let rule = CssRule {
            property: "transform".to_string(),
            value: Some(css.to_string()),
            declaration: format!("transform: {}", css),
        };
        TransformConverter.convert(&mut ctx, &rule)
    }

    #[test]
    fn translate() { assert!(run("translate(10px, 20px)")); }
    #[test]
    fn rotate() { assert!(run("rotate(45deg)")); }
    #[test]
    fn scale() { assert!(run("scale(1.5)")); }
    #[test]
    fn chained() { assert!(run("translateX(10px) rotate(45deg)")); }
    #[test]
    fn none() { assert!(run("none")); }
    #[test]
    fn unknown_fn_decline() { assert!(!run("skew(10deg)")); }
}
