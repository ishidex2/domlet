/**
 *
 *      Get elements matching to a selector
 *
 */
use crate::{
    css::parser::CssSelector,
    dom::node::{Element, Node},
    tree::BasicTree,
};

pub fn get_matching_elements<'a, Tree>(
    selector: &CssSelector,
    tree: &'a mut Tree,
) -> Vec<&'a mut Element>
where
    Tree: BasicTree<Node>
{
    let mut matching = vec![];
    if let Some(element) = tree.get_node().as_element_mut() {
        if super::matcher::matches_selector(element, selector) {
            matching.push(element)
        }
    }

    let child_count = tree.child_count();
    for index in 0..child_count {
        let child = tree.get_child_mut(index).unwrap();
        matching.append(&mut get_matching_elements(selector, child));
    }

    matching
}
