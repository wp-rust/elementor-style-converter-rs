use crate::types::*;
use crate::value_parsers::SizeValueParser;
use serde_json::Value;

/// Stateless parser for CSS `filter` / `backdrop-filter` values.
/// Returns an ordered list of `css-filter-func` PropValues, or `None` to decline.
/// All-or-nothing: one unsupported function declines the entire value.
pub struct FilterValueParser;

const SUPPORTED_FUNCTIONS: &[(&str, &str)] = &[
    ("blur", "blur"),
    ("brightness", "intensity"),
    ("contrast", "intensity"),
    ("saturate", "intensity"),
    ("grayscale", "color-tone"),
    ("invert", "color-tone"),
    ("sepia", "color-tone"),
    ("hue-rotate", "hue-rotate"),
    ("drop-shadow", "drop-shadow"),
];

impl FilterValueParser {
    pub fn parse(value: &str) -> Option<Vec<Value>> {
        let functions = Self::split_functions(value.trim())?;

        if functions.is_empty() {
            return None;
        }

        functions
            .into_iter()
            .map(|(name, args)| Self::parse_function(&name, &args))
            .collect()
    }

    fn split_functions(value: &str) -> Option<Vec<(String, String)>> {
        let mut functions = Vec::new();
        let chars: Vec<char> = value.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            // skip whitespace
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            if i >= len {
                break;
            }

            // read function name (alpha and hyphens)
            let name_start = i;
            while i < len && (chars[i].is_alphabetic() || chars[i] == '-') {
                i += 1;
            }
            let name: String = chars[name_start..i].iter().collect();

            // skip whitespace
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }

            if name.is_empty() || i >= len || chars[i] != '(' {
                return None;
            }

            // find matching close paren
            let args_start = i + 1;
            let mut depth = 0i32;
            while i < len {
                if chars[i] == '(' {
                    depth += 1;
                } else if chars[i] == ')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                i += 1;
            }

            if depth != 0 || i >= len {
                return None;
            }

            let args: String = chars[args_start..i].iter().collect();
            let args = args.trim().to_string();
            functions.push((name.to_lowercase(), args));
            i += 1; // skip closing ')'
        }

        if functions.is_empty() {
            None
        } else {
            Some(functions)
        }
    }

    fn parse_function(name: &str, args: &str) -> Option<Value> {
        let group = SUPPORTED_FUNCTIONS
            .iter()
            .find(|(func, _)| *func == name)
            .map(|(_, g)| *g)?;

        let parsed_args = if group == "drop-shadow" {
            Self::parse_drop_shadow(args)?
        } else {
            Self::parse_single_size(group, args)?
        };

        Some(serde_json::json!({
            "$$type": CSS_FILTER_FUNC_KEY,
            "value": {
                "func": {"$$type": STRING_KEY, "value": name},
                "args": parsed_args,
            }
        }))
    }

    fn parse_single_size(group: &str, args: &str) -> Option<Value> {
        // Intensity/color-tone values like brightness(1.5) are unitless fractions — allow.
        let allow_unitless = matches!(group, "intensity" | "color-tone" | "hue-rotate");
        let size = SizeValueParser::parse(args, allow_unitless)?;
        let size_val: Value = serde_json::json!({
            "$$type": SIZE_KEY,
            "value": {"size": size.size, "unit": size.unit}
        });

        let type_key = match group {
            "blur" => "blur",
            "intensity" => "intensity",
            "color-tone" => "color-tone",
            "hue-rotate" => "hue-rotate",
            _ => return None,
        };

        Some(serde_json::json!({
            "$$type": type_key,
            "value": {"size": size_val}
        }))
    }

    fn parse_drop_shadow(args: &str) -> Option<Value> {
        // Split on top-level whitespace (same as CssTokenSplitter but inline)
        let tokens = Self::split_top_level(args);

        if tokens.is_empty() {
            return None;
        }

        let mut sizes = Vec::new();
        let mut color: Option<String> = None;

        for token in &tokens {
            if let Some(s) = SizeValueParser::parse(token, false) {
                sizes.push(s);
            } else {
                if color.is_some() {
                    return None; // two colors — decline
                }
                color = Some(token.clone());
            }
        }

        if sizes.len() < 2 || sizes.len() > 3 {
            return None;
        }

        let blur = sizes.get(2).cloned().unwrap_or(crate::types::SizeLeaf {
            size: SizeValue::Number(10.0),
            unit: UNIT_PX.to_string(),
        });

        let default_color = "rgba(0, 0, 0, 1)".to_string();
        let color_str = color.unwrap_or(default_color);

        Some(serde_json::json!({
            "$$type": "drop-shadow",
            "value": {
                "xAxis": {"$$type": SIZE_KEY, "value": {"size": sizes[0].size, "unit": sizes[0].unit}},
                "yAxis": {"$$type": SIZE_KEY, "value": {"size": sizes[1].size, "unit": sizes[1].unit}},
                "blur":  {"$$type": SIZE_KEY, "value": {"size": blur.size, "unit": blur.unit}},
                "color": {"$$type": COLOR_KEY, "value": color_str},
            }
        }))
    }

    fn split_top_level(value: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut depth = 0i32;

        for ch in value.chars() {
            match ch {
                '(' => {
                    depth += 1;
                    current.push(ch);
                }
                ')' => {
                    depth = (depth - 1).max(0);
                    current.push(ch);
                }
                c if c.is_whitespace() && depth == 0 => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blur() {
        let r = FilterValueParser::parse("blur(5px)").unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0]["value"]["func"]["value"], "blur");
    }

    #[test]
    fn multiple_functions() {
        let r = FilterValueParser::parse("blur(2px) brightness(1.5)").unwrap();
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn drop_shadow() {
        let r = FilterValueParser::parse("drop-shadow(2px 3px 4px red)").unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0]["value"]["func"]["value"], "drop-shadow");
    }

    #[test]
    fn unknown_function_decline() {
        assert!(FilterValueParser::parse("blur(5px) spin(45deg)").is_none());
    }
}
