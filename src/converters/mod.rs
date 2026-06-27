pub mod color;
pub mod size;
pub mod number;
pub mod string_conv;
pub mod span;
pub mod dimensions;
pub mod border_radius;
pub mod box_shadow;
pub mod flex;
pub mod transform;
pub mod filter;
pub mod noop;
pub mod rejected;

use crate::context::ConversionContext;
use crate::types::CssRule;

/// The converter trait — mirrors PHP `Property_Converter`.
pub trait PropertyConverter: Send + Sync {
    fn is_supported(&self, rule: &CssRule) -> bool;
    /// Returns `true` if the rule was converted (context was mutated).
    fn convert(&self, ctx: &mut ConversionContext, rule: &CssRule) -> bool;
}
