/// The canonical PropValue type used throughout the converter.
/// Elementor's prop system uses tagged objects: `{"$$type": "...", "value": ...}`.
/// A `null` value means "reset this prop to its CSS initial".
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A typed Elementor PropValue.
/// Most values are `{"$$type": "...", "value": ...}`.
/// Null resets the prop to CSS initial.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropValue {
    Null,
    Tagged(TaggedValue),
    Raw(Value),
}

impl PropValue {
    pub fn null() -> Self {
        PropValue::Null
    }

    pub fn tagged(type_key: impl Into<String>, value: Value) -> Self {
        PropValue::Tagged(TaggedValue {
            type_key: type_key.into(),
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaggedValue {
    #[serde(rename = "$$type")]
    pub type_key: String,
    pub value: Value,
}

// ─── Size ────────────────────────────────────────────────────────────────────

/// `{"$$type": "size", "value": {"size": <number|string|null>, "unit": <string>}}`
pub fn size_prop(size: SizeLeaf) -> PropValue {
    PropValue::tagged(
        "size",
        serde_json::json!({
            "size": size.size,
            "unit": size.unit,
        }),
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct SizeLeaf {
    /// numeric value, or a raw string (e.g. "calc(100% - 10px)", "auto")
    pub size: SizeValue,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SizeValue {
    Null,
    Number(f64),
    Raw(String),
}

impl From<SizeLeaf> for Value {
    fn from(s: SizeLeaf) -> Value {
        serde_json::json!({
            "size": s.size,
            "unit": s.unit,
        })
    }
}

pub const SIZE_KEY: &str = "size";
pub const UNIT_PX: &str = "px";
pub const UNIT_AUTO: &str = "auto";
pub const UNIT_CUSTOM: &str = "custom";
pub const DEFAULT_UNIT: &str = UNIT_PX;

pub const ALL_SUPPORTED_UNITS: &[&str] = &[
    "px", "em", "rem", "%", "vw", "vh", "vmin", "vmax", "dvw", "dvh", "svw", "svh", "lvw", "lvh",
    "fr", "deg", "rad", "grad", "turn", "s", "ms", "ch", "ex", "lh", "rlh", "cqi", "cqb", "cqw",
    "cqh", "pt", "cm", "mm", "in", "pc",
];

// ─── Color ───────────────────────────────────────────────────────────────────

pub const COLOR_KEY: &str = "color";

/// `{"$$type": "color", "value": "<css-color-string>"}`
pub fn color_prop(value: impl Into<String>) -> PropValue {
    PropValue::tagged(COLOR_KEY, Value::String(value.into()))
}

// ─── String ──────────────────────────────────────────────────────────────────

pub const STRING_KEY: &str = "string";

/// `{"$$type": "string", "value": "<string>"}`
pub fn string_prop(value: impl Into<String>) -> PropValue {
    PropValue::tagged(STRING_KEY, Value::String(value.into()))
}

// ─── Number ──────────────────────────────────────────────────────────────────

pub const NUMBER_KEY: &str = "number";

/// `{"$$type": "number", "value": <f64>}`
pub fn number_prop(value: f64) -> PropValue {
    PropValue::tagged(NUMBER_KEY, serde_json::json!(value))
}

// ─── Dimensions (padding / margin / border-width sides) ──────────────────────

pub const DIMENSIONS_KEY: &str = "dimensions";
pub const BORDER_WIDTH_KEY: &str = "border-width";

/// `{"$$type": "dimensions", "value": {"block-start": <size>, ...}}`
pub fn dimensions_prop(sides: DimensionSides) -> PropValue {
    PropValue::tagged(
        DIMENSIONS_KEY,
        serde_json::json!({
            "block-start": sides.block_start,
            "inline-end": sides.inline_end,
            "block-end": sides.block_end,
            "inline-start": sides.inline_start,
        }),
    )
}

pub fn border_width_prop(sides: DimensionSides) -> PropValue {
    PropValue::tagged(
        BORDER_WIDTH_KEY,
        serde_json::json!({
            "block-start": sides.block_start,
            "inline-end": sides.inline_end,
            "block-end": sides.block_end,
            "inline-start": sides.inline_start,
        }),
    )
}

pub struct DimensionSides {
    pub block_start: Value,
    pub inline_end: Value,
    pub block_end: Value,
    pub inline_start: Value,
}

// ─── Border Radius ───────────────────────────────────────────────────────────

pub const BORDER_RADIUS_KEY: &str = "border-radius";

/// `{"$$type": "border-radius", "value": {"start-start": <size>, ...}}`
pub fn border_radius_prop(corners: BorderRadiusCorners) -> PropValue {
    PropValue::tagged(
        BORDER_RADIUS_KEY,
        serde_json::json!({
            "start-start": corners.start_start,
            "start-end": corners.start_end,
            "end-end": corners.end_end,
            "end-start": corners.end_start,
        }),
    )
}

pub struct BorderRadiusCorners {
    pub start_start: Value,
    pub start_end: Value,
    pub end_end: Value,
    pub end_start: Value,
}

// ─── Box Shadow ──────────────────────────────────────────────────────────────

pub const BOX_SHADOW_KEY: &str = "box-shadow";
pub const SHADOW_KEY: &str = "shadow";

/// `{"$$type": "box-shadow", "value": [<shadow>, ...]}`
pub fn box_shadow_prop(shadows: Vec<Value>) -> PropValue {
    PropValue::tagged(BOX_SHADOW_KEY, Value::Array(shadows))
}

pub fn shadow_value(fields: ShadowFields) -> Value {
    let mut map = serde_json::json!({
        "$$type": SHADOW_KEY,
        "value": {
            "hOffset": fields.h_offset,
            "vOffset": fields.v_offset,
            "blur": fields.blur,
            "spread": fields.spread,
            "color": fields.color,
        }
    });
    if let Some(pos) = fields.position {
        map["value"]["position"] = serde_json::json!(pos);
    }
    map
}

pub struct ShadowFields {
    pub h_offset: Value,
    pub v_offset: Value,
    pub blur: Value,
    pub spread: Value,
    pub color: Value,
    pub position: Option<Value>,
}

// ─── Flex ────────────────────────────────────────────────────────────────────

pub const FLEX_KEY: &str = "flex";

pub fn flex_prop(grow: f64, shrink: f64, basis: SizeLeaf) -> PropValue {
    PropValue::tagged(
        FLEX_KEY,
        serde_json::json!({
            "$$type": FLEX_KEY,
            "value": {
                "flexGrow": {"$$type": NUMBER_KEY, "value": grow},
                "flexShrink": {"$$type": NUMBER_KEY, "value": shrink},
                "flexBasis": {"$$type": SIZE_KEY, "value": Value::from(basis)},
            }
        }),
    )
}

// ─── Transform ───────────────────────────────────────────────────────────────

pub const TRANSFORM_KEY: &str = "transform";
pub const TRANSFORM_MOVE_KEY: &str = "transform-move";
pub const TRANSFORM_SCALE_KEY: &str = "transform-scale";
pub const TRANSFORM_ROTATE_KEY: &str = "transform-rotate";
pub const TRANSFORM_FUNCTIONS_KEY: &str = "transform-functions";

// ─── Filter ──────────────────────────────────────────────────────────────────

pub const FILTER_KEY: &str = "filter";
pub const BACKDROP_FILTER_KEY: &str = "backdrop-filter";
pub const CSS_FILTER_FUNC_KEY: &str = "css-filter-func";

// ─── Span (grid placement) ───────────────────────────────────────────────────

pub const SPAN_KEY: &str = "span";

pub fn span_prop(value: impl Into<String>) -> PropValue {
    PropValue::tagged(SPAN_KEY, Value::String(value.into()))
}

// ─── Conversion result ───────────────────────────────────────────────────────

/// The output of `CssConverter::convert()`.
#[derive(Debug, Default)]
pub struct ConversionResult {
    /// Typed props that were successfully converted.
    /// `None` values represent explicit null-resets (set the prop to its CSS initial).
    pub props: HashMap<String, Option<PropValue>>,
    /// CSS declarations that had no converter and were left as-is.
    pub custom_css: String,
    /// CSS declarations that are structurally incompatible (e.g. `animation`).
    pub rejected: Vec<String>,
}

/// A single parsed CSS declaration.
#[derive(Debug, Clone)]
pub struct CssRule {
    pub property: String,
    pub value: Option<String>, // None = explicit `null` reset
    pub declaration: String,
}
