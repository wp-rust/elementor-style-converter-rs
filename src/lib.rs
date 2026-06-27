//! # elementor-style-converter-rs
//!
//! Converts Elementor atomic widget CSS strings into typed PropValue trees.
//!
//! This is a Rust port of Elementor's PHP `modules/atomic-widgets/css-converter` module.
//!
//! ## Usage
//!
//! ```rust
//! use elementor_style_converter_rs::{CssConverter, ConverterRegistryFactory};
//!
//! let registry = ConverterRegistryFactory::create_registry();
//! let expanders = ConverterRegistryFactory::create_expanders();
//! let converter = CssConverter::new(registry, expanders);
//!
//! let result = converter.convert("color: red; padding: 10px 20px; font-size: 16px");
//! // result.props contains typed PropValues
//! // result.custom_css contains unconverted declarations
//! // result.rejected contains incompatible declarations (e.g. animation)
//! ```

pub mod context;
#[cfg(test)]
mod tests;
pub mod converters;
pub mod css_converter;
pub mod expanders;
pub mod factory;
pub mod registry;
pub mod types;
pub mod value_parsers;

// Top-level re-exports
pub use css_converter::CssConverter;
pub use factory::ConverterRegistryFactory;
pub use types::{ConversionResult, PropValue};
