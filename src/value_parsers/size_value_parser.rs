use crate::types::{SizeLeaf, SizeValue, ALL_SUPPORTED_UNITS, DEFAULT_UNIT, UNIT_AUTO, UNIT_CUSTOM};
use regex::Regex;
use std::sync::OnceLock;

static NUMBER_WITH_UNIT: OnceLock<Regex> = OnceLock::new();
static DYNAMIC_FUNC: OnceLock<Regex> = OnceLock::new();

fn number_with_unit() -> &'static Regex {
    NUMBER_WITH_UNIT.get_or_init(|| Regex::new(r"(?i)^(-?\d*\.?\d+)([a-z%]*)$").unwrap())
}

fn dynamic_func() -> &'static Regex {
    DYNAMIC_FUNC.get_or_init(|| Regex::new(r"(?i)(?:calc|clamp|min|max|var|env)\(").unwrap())
}

/// Stateless parser: raw CSS length-ish token → `SizeLeaf`, or `None` to decline.
///
/// Rules (matching PHP `Size_Value_Parser::parse`):
/// - `"auto"`                          → `{ size: Null, unit: "auto" }`
/// - `calc()/clamp()/min()/max()/var()/env()` → `{ size: Raw("<raw>"), unit: "custom" }`
/// - `"<number><unit>"` (unit in `ALL_SUPPORTED_UNITS`) → `{ size: Number(n), unit }`
/// - unitless `"0"`                    → `{ size: Number(0), unit: "px" }`
/// - unitless non-zero, `allow_unitless=true` → `{ size: Raw("<raw>"), unit: "custom" }`
/// - anything else                     → `None`
pub struct SizeValueParser;

impl SizeValueParser {
    pub fn parse(value: &str, allow_unitless: bool) -> Option<SizeLeaf> {
        let value = value.trim();

        if value.is_empty() {
            return None;
        }

        if value.eq_ignore_ascii_case(UNIT_AUTO) {
            return Some(SizeLeaf {
                size: SizeValue::Null,
                unit: UNIT_AUTO.to_string(),
            });
        }

        if dynamic_func().is_match(value) {
            return Some(SizeLeaf {
                size: SizeValue::Raw(value.to_string()),
                unit: UNIT_CUSTOM.to_string(),
            });
        }

        Self::parse_number_with_unit(value, allow_unitless)
    }

    fn parse_number_with_unit(value: &str, allow_unitless: bool) -> Option<SizeLeaf> {
        let caps = number_with_unit().captures(value)?;
        let num_str = &caps[1];
        let unit_str = caps[2].to_lowercase();

        let num: f64 = num_str.parse().ok()?;

        if unit_str.is_empty() {
            if num == 0.0 {
                return Some(SizeLeaf {
                    size: SizeValue::Number(0.0),
                    unit: DEFAULT_UNIT.to_string(),
                });
            }
            if allow_unitless {
                return Some(SizeLeaf {
                    size: SizeValue::Raw(num_str.to_string()),
                    unit: UNIT_CUSTOM.to_string(),
                });
            }
            return None;
        }

        if !ALL_SUPPORTED_UNITS.contains(&unit_str.as_str()) {
            return None;
        }

        Some(SizeLeaf {
            size: SizeValue::Number(num),
            unit: unit_str,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_px() {
        let r = SizeValueParser::parse("10px", false).unwrap();
        assert_eq!(r.unit, "px");
        assert_eq!(r.size, SizeValue::Number(10.0));
    }

    #[test]
    fn parses_percent() {
        let r = SizeValueParser::parse("50%", false).unwrap();
        assert_eq!(r.unit, "%");
    }

    #[test]
    fn parses_auto() {
        let r = SizeValueParser::parse("auto", false).unwrap();
        assert_eq!(r.unit, "auto");
        assert_eq!(r.size, SizeValue::Null);
    }

    #[test]
    fn parses_zero_no_unit() {
        let r = SizeValueParser::parse("0", false).unwrap();
        assert_eq!(r.unit, "px");
        assert_eq!(r.size, SizeValue::Number(0.0));
    }

    #[test]
    fn declines_unitless_nonzero_without_flag() {
        assert!(SizeValueParser::parse("1.5", false).is_none());
    }

    #[test]
    fn accepts_unitless_nonzero_with_flag() {
        let r = SizeValueParser::parse("1.5", true).unwrap();
        assert_eq!(r.unit, "custom");
    }

    #[test]
    fn parses_calc() {
        let r = SizeValueParser::parse("calc(100% - 10px)", false).unwrap();
        assert_eq!(r.unit, "custom");
        assert_eq!(r.size, SizeValue::Raw("calc(100% - 10px)".to_string()));
    }

    #[test]
    fn declines_unknown_unit() {
        assert!(SizeValueParser::parse("10xyz", false).is_none());
    }

    #[test]
    fn parses_negative() {
        let r = SizeValueParser::parse("-5px", false).unwrap();
        assert_eq!(r.size, SizeValue::Number(-5.0));
    }

    #[test]
    fn parses_em_rem() {
        assert!(SizeValueParser::parse("1.5em", false).is_some());
        assert!(SizeValueParser::parse("2rem", false).is_some());
    }

    #[test]
    fn parses_deg() {
        let r = SizeValueParser::parse("45deg", false).unwrap();
        assert_eq!(r.unit, "deg");
    }
}
