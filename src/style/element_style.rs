/*!
 *
 *      Style descriptor for an element
 *
 */

use std::convert::TryFrom;

use super::styles;
use crate::css::parser::CssRuleset;

/**
 * The representation of element style via a single struct.
 * It has advatage of being quick to access. However as amount of
 * possible style rules grows, this struct may become too big,
 * with unnecessary styles taking up space for no reason.
 *
 * This should be resolved at the point style amount exceeds 1kb,
 * If we consider on average an application consists of 5000 nodes
 * simultaneously on the screen, the overhead will 1kb*5000 = 5 megabytes.
 * While that may seem as not much, we have to consider that this
 * is not the only source of RAM consumption. This is to be thought about.
 */
// This derive macro will assign all elements to None
#[derive(Default)]
pub struct ElementStyle {
    pub padding: Option<styles::Padding>,
    pub margin: Option<styles::Margin>,
    pub background_color: Option<styles::BackgroundColor>,
    pub color: Option<styles::Color>,
}

impl From<CssRuleset> for ElementStyle {
    /**
     * Conversion routine from ruleset to ElementStyle.
     * Currently very repetitive but should be fine because
     * it's very homogenous code anyway.
     */
    fn from(ruleset: CssRuleset) -> Self {
        let mut context = Self::default();

        for rule in ruleset {
            match rule.name.as_str() {
                "padding" => context.padding = TryFrom::try_from(rule.params.as_slice()).ok(),
                "margin" => context.margin = TryFrom::try_from(rule.params.as_slice()).ok(),
                "background-color" => {
                    context.background_color = TryFrom::try_from(rule.params.as_slice()).ok()
                }
                "color" => context.color = TryFrom::try_from(rule.params.as_slice()).ok(),
                _ => {}
            }
        }

        context
    }
}
