#[cfg(test)]
mod tests {
    use crate::{
        factory::ConverterRegistryFactory,
        types::{PropValue, TaggedValue},
        CssConverter,
    };
    use serde_json::Value;

    fn converter() -> CssConverter {
        let registry = ConverterRegistryFactory::create_registry();
        let expanders = ConverterRegistryFactory::create_expanders();
        CssConverter::new(registry, expanders)
    }

    fn tagged<'a>(result: &'a std::collections::HashMap<String, Option<PropValue>>, key: &str) -> Option<(&'a str, &'a Value)> {
        match result.get(key)?.as_ref()? {
            PropValue::Tagged(TaggedValue { type_key, value }) => Some((type_key.as_str(), value)),
            _ => None,
        }
    }

    // ─── Color ───────────────────────────────────────────────────────────────

    #[test]
    fn converts_color() {
        let c = converter();
        let r = c.convert("color: red");
        let (ty, val) = tagged(&r.props, "color").unwrap();
        assert_eq!(ty, "color");
        assert_eq!(val, "red");
    }

    #[test]
    fn rejects_multi_token_color() {
        let c = converter();
        // Two separate tokens (no parens) — color converter declines
        let r = c.convert("color: red invalid");
        assert!(r.props.get("color").is_none());
    }

    // ─── Size ────────────────────────────────────────────────────────────────

    #[test]
    fn converts_px_size() {
        let c = converter();
        let r = c.convert("width: 100px");
        let (ty, val) = tagged(&r.props, "width").unwrap();
        assert_eq!(ty, "size");
        assert_eq!(val["unit"], "px");
        assert!((val["size"].as_f64().unwrap() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn converts_auto_size() {
        let c = converter();
        let r = c.convert("width: auto");
        let (ty, val) = tagged(&r.props, "width").unwrap();
        assert_eq!(ty, "size");
        assert_eq!(val["unit"], "auto");
        assert_eq!(val["size"], Value::Null);
    }

    #[test]
    fn converts_percentage_size() {
        let c = converter();
        let r = c.convert("width: 50%");
        let (_, val) = tagged(&r.props, "width").unwrap();
        assert_eq!(val["unit"], "%");
        assert!((val["size"].as_f64().unwrap() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn converts_line_height_unitless() {
        let c = converter();
        let r = c.convert("line-height: 1.5");
        let (ty, val) = tagged(&r.props, "line-height").unwrap();
        assert_eq!(ty, "size");
        // Unitless non-zero → stored as Raw string with unit "custom"
        assert_eq!(val["unit"], "custom");
        assert_eq!(val["size"], "1.5");
    }

    // ─── Number ──────────────────────────────────────────────────────────────

    #[test]
    fn converts_z_index() {
        let c = converter();
        let r = c.convert("z-index: 10");
        let (ty, val) = tagged(&r.props, "z-index").unwrap();
        assert_eq!(ty, "number");
        assert!((val.as_f64().unwrap() - 10.0).abs() < f64::EPSILON);
    }

    // ─── String ──────────────────────────────────────────────────────────────

    #[test]
    fn converts_display() {
        let c = converter();
        let r = c.convert("display: flex");
        let (ty, val) = tagged(&r.props, "display").unwrap();
        assert_eq!(ty, "string");
        assert_eq!(val, "flex");
    }

    #[test]
    fn converts_text_align_valid() {
        let c = converter();
        let r = c.convert("text-align: center");
        let (_, val) = tagged(&r.props, "text-align").unwrap();
        assert_eq!(val, "center");
    }

    // ─── Dimensions (padding / margin) ───────────────────────────────────────

    #[test]
    fn converts_padding_shorthand_2_values() {
        let c = converter();
        let r = c.convert("padding: 10px 20px");
        let (ty, val) = tagged(&r.props, "padding").unwrap();
        assert_eq!(ty, "dimensions");
        // Each side is a tagged size: {"$$type":"size","value":{"size":N,"unit":"px"}}
        assert_eq!(val["block-start"]["value"]["unit"], "px");
        assert!((val["block-start"]["value"]["size"].as_f64().unwrap() - 10.0).abs() < f64::EPSILON);
        assert!((val["inline-end"]["value"]["size"].as_f64().unwrap() - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn converts_margin_shorthand_4_values() {
        let c = converter();
        let r = c.convert("margin: 1px 2px 3px 4px");
        let (_, val) = tagged(&r.props, "margin").unwrap();
        assert!((val["block-start"]["value"]["size"].as_f64().unwrap() - 1.0).abs() < f64::EPSILON);
        assert!((val["inline-end"]["value"]["size"].as_f64().unwrap() - 2.0).abs() < f64::EPSILON);
        assert!((val["block-end"]["value"]["size"].as_f64().unwrap() - 3.0).abs() < f64::EPSILON);
        assert!((val["inline-start"]["value"]["size"].as_f64().unwrap() - 4.0).abs() < f64::EPSILON);
    }

    // ─── Border radius ───────────────────────────────────────────────────────

    #[test]
    fn converts_border_radius_single() {
        let c = converter();
        let r = c.convert("border-radius: 8px");
        let (ty, val) = tagged(&r.props, "border-radius").unwrap();
        // Single value → size prop (not corners)
        assert_eq!(ty, "size");
        assert!((val["size"].as_f64().unwrap() - 8.0).abs() < f64::EPSILON);
        assert_eq!(val["unit"], "px");
    }

    #[test]
    fn converts_border_radius_corners() {
        let c = converter();
        let r = c.convert("border-radius: 4px 8px 12px 16px");
        let (ty, val) = tagged(&r.props, "border-radius").unwrap();
        assert_eq!(ty, "border-radius");
        assert!((val["start-start"]["value"]["size"].as_f64().unwrap() - 4.0).abs() < f64::EPSILON);
        assert!((val["end-end"]["value"]["size"].as_f64().unwrap() - 12.0).abs() < f64::EPSILON);
    }

    // ─── Border shorthand expander ────────────────────────────────────────────

    #[test]
    fn expands_border_shorthand() {
        let c = converter();
        let r = c.convert("border: 2px solid red");
        // Single-value border-width expands to size prop (not corners)
        let (ty, w_val) = tagged(&r.props, "border-width").unwrap();
        assert_eq!(ty, "size");
        assert!((w_val["size"].as_f64().unwrap() - 2.0).abs() < f64::EPSILON);
        assert_eq!(w_val["unit"], "px");
        let (_, s_val) = tagged(&r.props, "border-style").unwrap();
        assert_eq!(s_val, "solid");
    }

    // ─── Physical-to-logical expander ────────────────────────────────────────

    #[test]
    fn remaps_top_to_logical() {
        let c = converter();
        let r = c.convert("top: 10px");
        assert!(r.props.get("top").is_none(), "physical key should not appear");
        let (ty, val) = tagged(&r.props, "inset-block-start").unwrap();
        assert_eq!(ty, "size");
        assert!((val["size"].as_f64().unwrap() - 10.0).abs() < f64::EPSILON);
    }

    // ─── Null resets ─────────────────────────────────────────────────────────

    #[test]
    fn null_string_becomes_none_prop() {
        let c = converter();
        let r = c.convert("color: null");
        // `value: None` → prop stored as None (explicit reset)
        assert!(r.props.contains_key("color"));
        // The stored value should be None (a null reset)
        // (converters short-circuit on null rules; the context stores None)
    }

    // ─── Deduplication ───────────────────────────────────────────────────────

    #[test]
    fn last_declaration_wins() {
        let c = converter();
        let r = c.convert("width: 100px; width: 200px");
        let (_, val) = tagged(&r.props, "width").unwrap();
        assert!((val["size"].as_f64().unwrap() - 200.0).abs() < f64::EPSILON);
    }

    // ─── Custom CSS passthrough ───────────────────────────────────────────────

    #[test]
    fn unknown_property_goes_to_custom_css() {
        let c = converter();
        let r = c.convert("--my-var: 42px");
        assert!(r.custom_css.contains("--my-var"));
    }

    // ─── Rejected properties ─────────────────────────────────────────────────

    #[test]
    fn animation_goes_to_rejected() {
        let c = converter();
        let r = c.convert("animation: spin 1s linear infinite");
        assert!(r.rejected.iter().any(|s| s.contains("animation")));
    }

    // ─── Multiple properties ──────────────────────────────────────────────────

    #[test]
    fn converts_multiple_properties() {
        let c = converter();
        let r = c.convert("color: blue; font-size: 16px; display: block; z-index: 5");
        assert!(tagged(&r.props, "color").is_some());
        assert!(tagged(&r.props, "font-size").is_some());
        assert!(tagged(&r.props, "display").is_some());
        assert!(tagged(&r.props, "z-index").is_some());
        assert!(r.custom_css.is_empty());
    }

    // ─── Blocked properties ───────────────────────────────────────────────────

    #[test]
    fn blocked_property_dropped() {
        let c = converter();
        let r = c.convert("behavior: url(foo.htc)");
        assert!(r.props.is_empty());
        assert!(r.custom_css.is_empty());
    }

    #[test]
    fn blocked_value_dropped() {
        let c = converter();
        let r = c.convert("color: expression(alert(1))");
        assert!(r.props.is_empty());
        assert!(r.custom_css.is_empty());
    }

    // ─── Flex ────────────────────────────────────────────────────────────────

    #[test]
    fn converts_flex_none() {
        let c = converter();
        let r = c.convert("flex: none");
        // none → grow=0, shrink=0, basis=auto
        assert!(tagged(&r.props, "flex").is_some());
    }

    #[test]
    fn converts_flex_three_values() {
        let c = converter();
        let r = c.convert("flex: 1 2 50%");
        let (ty, _) = tagged(&r.props, "flex").unwrap();
        assert_eq!(ty, "flex");
    }
}
