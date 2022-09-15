/*!
*
*       DOM Element & Node
*
*/

use std::collections::{BTreeSet, HashMap};

use crate::{style::element_style::ElementStyle, util::first_until_whitespace};

/**
 * Attributes of an element
 */
pub type ElementAttributes = HashMap<String, String>;

pub struct Element {
    pub(super) styles: ElementStyle,
    pub(super) tag_name: String,
    pub(super) attributes: ElementAttributes,
}

impl Element {
    pub fn get_tag_name(&self) -> &str {
        self.tag_name.as_str()
    }

    /**
     * Retrieves id of the element
     */
    pub fn get_id(&self) -> Option<&str> {
        Some(first_until_whitespace(self.attributes.get("id")?))
    }

    /**
     * Retrieves class list of element,
     * For now it recalculates the class list every time we want to know it
     * That will be changed soon
     */
    pub fn get_class_list(&self) -> BTreeSet<&str> {
        match self.attributes.get("class") {
            Some(list) => list.split(' ').collect(),
            None => BTreeSet::new(),
        }
    }

    pub fn new(tag_name: String, attributes: ElementAttributes) -> Self {
        Self {
            tag_name,
            attributes,
            styles: Default::default(),
        }
    }
}

/**
 * Node represents either the text of document or the element that might contain other
 * Text or other nodes.
 */
pub enum Node {
    Text(String),
    Element(Element),
}

impl Node {
    /**
     * Conversion routine that will transform into `&str` if it is
     * a node of type `Text`
     */
    pub fn as_text(&self) -> Option<&str> {
        if let Node::Text(text) = self {
            return Some(text.as_str());
        }
        None
    }

    /**
     * Conversion routine that will transform into `&Element` if it is
     * a node of type `Element`
     */
    pub fn as_element(&self) -> Option<&Element> {
        if let Node::Element(el) = self {
            return Some(&el);
        }
        None
    }

    /**
     * Conversion routine that will transform into `&mut Element` if it is
     * a node of type `Element`
     */
    pub fn as_element_mut(&mut self) -> Option<&mut Element> {
        if let Node::Element(el) = self {
            return Some(&mut el);
        }
        None
    }

}
