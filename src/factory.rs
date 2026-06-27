/// `ConverterRegistryFactory` — mirrors PHP `Converter_Registry_Factory`.
/// Builds a fully-wired `ConverterRegistry` and `ExpanderRegistry` with all known converters.
use crate::converters::{
    border_radius::BorderRadiusConverter,
    box_shadow::BoxShadowConverter,
    color::ColorConverter,
    dimensions::DimensionsConverter,
    filter::FilterConverter,
    flex::FlexConverter,
    noop::NoopConverter,
    number::NumberConverter,
    rejected::RejectedConverter,
    size::SizeConverter,
    span::SpanConverter,
    string_conv::StringConverter,
    transform::TransformConverter,
};
use crate::expanders::{
    border::BorderShorthandExpander,
    outline::OutlineShorthandExpander,
    physical_to_logical::PhysicalToLogicalExpander,
};
use crate::registry::{ConverterRegistry, ExpanderRegistry};

// ─── Property categories ─────────────────────────────────────────────────────

const SIZE_PROPERTIES: &[&str] = &[
    "width", "height", "min-width", "min-height", "max-width", "max-height",
    "inset-block-start", "inset-inline-end", "inset-block-end", "inset-inline-start",
    "scroll-margin-top", "font-size", "letter-spacing", "word-spacing",
    "column-gap", "line-height", "outline-width", "outline-offset",
    "opacity", "gap", "grid-auto-rows", "grid-auto-columns",
];

const UNITLESS_SIZE_PROPERTIES: &[&str] = &["line-height"];

const NUMBER_PROPERTIES: &[&str] = &["z-index", "column-count", "order"];

const COLOR_PROPERTIES: &[&str] = &["color", "border-color", "outline-color"];

const SPAN_PROPERTIES: &[&str] = &["grid-column", "grid-row"];

const STRING_PASSTHROUGH_PROPERTIES: &[&str] =
    &["grid-template-columns", "grid-template-rows"];

const DIMENSIONS_PROPERTIES: &[&str] = &["padding", "margin"];

const FILTER_PROPERTIES: &[(&str, &str)] = &[
    ("filter", "filter"),
    ("backdrop-filter", "backdrop-filter"),
];

const STRING_PROPERTIES: &[(&str, Option<&[&str]>)] = &[
    ("overflow",         Some(&["visible", "hidden", "clip", "scroll", "auto"])),
    ("aspect-ratio",     None),
    ("object-fit",       Some(&["fill", "contain", "cover", "none", "scale-down"])),
    ("position",         Some(&["static", "relative", "absolute", "fixed", "sticky"])),
    ("font-family",      None),
    ("font-weight",      None),
    ("text-align",       Some(&["left", "right", "center", "justify", "start", "end"])),
    ("font-style",       Some(&["normal", "italic", "oblique"])),
    ("text-decoration",  None),
    ("text-transform",   Some(&["none", "capitalize", "uppercase", "lowercase"])),
    ("direction",        Some(&["ltr", "rtl"])),
    ("all",              None),
    ("cursor",           None),
    ("border-style",     Some(&["none", "hidden", "dotted", "dashed", "solid", "double", "groove", "ridge", "inset", "outset"])),
    ("outline-style",    Some(&["none", "auto", "dotted", "dashed", "solid", "double", "groove", "ridge", "inset", "outset"])),
    ("mix-blend-mode",   None),
    ("display",          None),
    ("flex-direction",   Some(&["row", "row-reverse", "column", "column-reverse"])),
    ("flex-wrap",        Some(&["nowrap", "wrap", "wrap-reverse"])),
    ("grid-auto-flow",   None),
    ("justify-content",  None),
    ("justify-items",    None),
    ("align-content",    None),
    ("align-items",      None),
    ("align-self",       None),
    ("content",          None),
    ("appearance",       None),
    ("clip-path",        None),
];

const NOOP_PROPERTIES: &[&str] = &[
    "stroke", "stroke-width", "stroke-opacity", "stroke-dasharray",
    "stroke-dashoffset", "stroke-linecap", "stroke-linejoin", "stroke-miterlimit",
    "background",
];

const REJECTED_PROPERTIES: &[&str] = &[
    "animation", "animation-name", "animation-duration", "animation-timing-function",
    "animation-delay", "animation-iteration-count", "animation-direction",
    "animation-fill-mode", "animation-play-state",
];

// ─── Physical/logical dimension longhands ────────────────────────────────────

const DIMENSIONS_SIDE_SPECS: &[(&str, &str, &str)] = &[
    ("padding-top",          "padding", "block-start"),
    ("padding-right",        "padding", "inline-end"),
    ("padding-bottom",       "padding", "block-end"),
    ("padding-left",         "padding", "inline-start"),
    ("padding-block-start",  "padding", "block-start"),
    ("padding-block-end",    "padding", "block-end"),
    ("padding-inline-start", "padding", "inline-start"),
    ("padding-inline-end",   "padding", "inline-end"),
    ("margin-top",           "margin",  "block-start"),
    ("margin-right",         "margin",  "inline-end"),
    ("margin-bottom",        "margin",  "block-end"),
    ("margin-left",          "margin",  "inline-start"),
    ("margin-block-start",   "margin",  "block-start"),
    ("margin-block-end",     "margin",  "block-end"),
    ("margin-inline-start",  "margin",  "inline-start"),
    ("margin-inline-end",    "margin",  "inline-end"),
];

// ─── Factory ─────────────────────────────────────────────────────────────────

pub struct ConverterRegistryFactory;

impl ConverterRegistryFactory {
    pub fn create_registry() -> ConverterRegistry {
        let mut registry = ConverterRegistry::default();

        // String properties (enum and free-string)
        for (prop, allowed) in STRING_PROPERTIES {
            registry.register(StringConverter::new(
                *prop,
                allowed.map(|vals| vals.iter().map(|s| s.to_string()).collect()),
            ));
        }

        // String passthroughs (no enum constraint)
        for prop in STRING_PASSTHROUGH_PROPERTIES {
            registry.register(StringConverter::passthrough(*prop));
        }

        // Size properties
        for prop in SIZE_PROPERTIES {
            let allow_unitless = UNITLESS_SIZE_PROPERTIES.contains(prop);
            registry.register(SizeConverter::new(*prop, allow_unitless));
        }

        // Number properties
        for prop in NUMBER_PROPERTIES {
            registry.register(NumberConverter::new(*prop));
        }

        // Color properties
        for prop in COLOR_PROPERTIES {
            registry.register(ColorConverter::new(*prop));
        }

        // Span properties (grid placement)
        for prop in SPAN_PROPERTIES {
            registry.register(SpanConverter::new(*prop, None));
        }

        // Box shorthands (padding, margin)
        for prop in DIMENSIONS_PROPERTIES {
            registry.register(DimensionsConverter::dimensions(*prop));
        }

        // Dimension side longhands (padding-top, margin-left, etc.)
        for (prop, _target, _side) in DIMENSIONS_SIDE_SPECS {
            // These contribute to the parent object — simplified: treat as size converters
            // for el2blocks use case (full merge logic requires stateful accumulation)
            registry.register(SizeConverter::new(*prop, false));
        }

        // Border-specific
        registry.register(BorderRadiusConverter::new("border-radius"));
        registry.register(DimensionsConverter::border_width("border-width"));

        // Complex converters
        registry.register(FlexConverter);
        registry.register(TransformConverter);
        registry.register(BoxShadowConverter);

        // Filter properties
        for (prop, type_key) in FILTER_PROPERTIES {
            registry.register(FilterConverter::new(*prop, *type_key));
        }

        // Rejected (incompatible) properties
        for prop in REJECTED_PROPERTIES {
            registry.register(RejectedConverter::new(*prop));
        }

        // Noop fallbacks (known but unconvertible)
        for prop in NOOP_PROPERTIES {
            registry.register(NoopConverter::new(*prop));
        }

        registry
    }

    pub fn create_expanders() -> ExpanderRegistry {
        let mut expanders = ExpanderRegistry::default();
        expanders.register(BorderShorthandExpander::all_sides());
        expanders.register(BorderShorthandExpander::side("top"));
        expanders.register(BorderShorthandExpander::side("right"));
        expanders.register(BorderShorthandExpander::side("bottom"));
        expanders.register(BorderShorthandExpander::side("left"));
        expanders.register(OutlineShorthandExpander);
        expanders.register(PhysicalToLogicalExpander);
        expanders
    }
}
