/*!
 *
 *      CSS selector matcher
 *
 */

use crate::{
    css::parser::{CssCompositeSelector, CssSelector, CssSimpleSelector},
    dom::node::Element,
};

/**
 * Check if matches selector
 * TODO: Add test
 */
pub fn matches_selector(el: &Element, selector: &CssSelector) -> bool {
    // Will return true if any selector matches
    selector
        .sels
        .iter()
        .any(|selector| matches_composite_selector(el, selector))
}

/**
 * Check if matches composite selector, selector that matches if multiple combined match
 * TODO: Add test
 */
fn matches_composite_selector(el: &Element, selector: &CssCompositeSelector) -> bool {
    // Will return true if all selectors match
    selector
        .sels
        .iter()
        .all(|selector| matches_simple_selector(el, selector))
}

/**
 * Check if matches simple selector, such as class, wildcard, id, tag, et cetera
 * TODO: Add test
 */
fn matches_simple_selector(element: &Element, selector: &CssSimpleSelector) -> bool {
    // For more convenient unwrapping
    let inner = || -> Option<bool> {
        Some(match selector {
            CssSimpleSelector::Class(class_name) => {
                element.get_class_list().contains(class_name.as_str())
            }
            CssSimpleSelector::Id(id) => element.get_id()? == id,
            CssSimpleSelector::Tag(tag_name) => element.get_tag_name() == tag_name,
        })
    };
    inner().unwrap_or(false)
}
