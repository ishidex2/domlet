/*!
 *
 *      Collection of style properties
 *
 */

use std::convert::TryFrom;

use crate::{css::parser::CssRuleParam, essentials::FourSide};

/**
 *  Generic 4 side modifier
 *  for now temporairly stored as f64, however that will be changed soon to
 *  "UnitValue" to represent contextual units
 */
pub type FourSides = FourSide<f64>;

impl TryFrom<&[CssRuleParam]> for FourSides {
    type Error = ();

    fn try_from(params: &[CssRuleParam]) -> Result<Self, Self::Error> {
        use CssRuleParam::Unit;

        // Pattern matching-fu
        // We are going to check if 4 values, 2 values or 1 value is a unit
        // WARNING: This assumes every unit is a pixel, this will change soon
        match (params.get(0), params.get(1), params.get(2), params.get(3)) {
            (Some(Unit(scalar, _)), None, None, None) => Ok(Self {
                left: *scalar,
                right: *scalar,
                top: *scalar,
                bottom: *scalar,
            }),
            (Some(Unit(vertical, _)), Some(Unit(horizontal, _)), None, None) => Ok(Self {
                left: *horizontal,
                right: *horizontal,
                top: *vertical,
                bottom: *vertical,
            }),
            (
                Some(Unit(top, _)),
                Some(Unit(right, _)),
                Some(Unit(bottom, _)),
                Some(Unit(left, _)),
            ) => Ok(Self {
                left: *left,
                right: *right,
                top: *top,
                bottom: *bottom,
            }),
            // Invalid value
            _ => Err(()),
        }
    }
}

/**
 *  Color in RGBA, way alpha color is stored
 *  might be a subject to reconsideration.
 */
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl TryFrom<&[CssRuleParam]> for Color {
    type Error = ();

    fn try_from(params: &[CssRuleParam]) -> Result<Self, Self::Error> {
        // Check if first parameter is the color
        if let Some(CssRuleParam::Color(red, green, blue, alpha)) = params.get(0) {
            return Ok(Color {
                red: *red,
                green: *green,
                blue: *blue,
                alpha: *alpha,
            });
        }
        Err(())
    }
}

/**
 *  Shared-format style aliases for clarity
 */

// We'll assume it's here
// type Color = Color;
pub type Padding = FourSides;
pub type Margin = FourSides;
pub type BackgroundColor = Color;
