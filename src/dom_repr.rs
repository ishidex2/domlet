use crate::bucket_array::{self, BucketArray};
use crate::util::first_until_whitespace;
use crate::util::split_by_whitespace_trimmed;
use crate::Id;
use std::collections::{hash_map::HashMap, BTreeSet};
use xml::reader::XmlEvent;
use xml::EventReader;

#[derive(Debug)]
pub struct Element {
    pub tag: String,
    attrs: HashMap<String, String>,
    pub class_list: BTreeSet<String>,
    id: Option<String>,
    pub children: Vec<Id>,
    parent: Option<Id>,
}

#[derive(Debug)]
pub struct Text {
    pub text: String,
}

#[derive(Debug)]
pub enum DomComponent {
    Text(Text),
    Element(Element),
}

struct XmlToDom<'a> {
    reader: EventReader<&'a [u8]>,
}

impl<'a> XmlToDom<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            reader: EventReader::new(bytes),
        }
    }

    pub fn parse(&mut self) -> Result<DomSystem, xml::reader::Error> {
        let mut sys = DomSystem {
            elements: BucketArray::new(),
            root: 0,
        };
        let res = self.parse_elements(&mut sys, None)?;
        self.post_process_attributes(&mut sys);
        assert_eq!(res.len(), 1);
        sys.root = *res.get(0).unwrap();
        Ok(sys)
    }

    fn post_process_attributes(&mut self, sys: &mut DomSystem) {
        for (_, element) in sys.elements.iter_mut() {
            // We only want to do that for elements, not text
            if let DomComponent::Element(el) = element {
                // Here we will need to assign class list to our elements
                let classes = el.attrs.remove("class").unwrap_or("".to_string());
                let id = el.attrs.remove("id").unwrap_or("".to_string());
                el.class_list = split_by_whitespace_trimmed(&classes).drain(..).collect();
                let id_value = first_until_whitespace(&id);
                // If id is "" there's no id
                el.id = if id_value == "" { None } else { Some(id_value) }
            }
        }
    }

    fn parse_element_open(
        &mut self,
        name: String,
        attributes: &[xml::attribute::OwnedAttribute],
        sys: &mut DomSystem,
        parent: Option<Id>,
    ) -> Result<Id, xml::reader::Error> {
        let el = Element {
            tag: name,
            attrs: attributes
                .iter()
                .map(|a| (a.name.to_string(), a.value.to_string()))
                .collect::<HashMap<String, String>>(),

            // We well assign those later
            class_list: BTreeSet::new(),
            id: None,

            children: vec![],
            parent,
        };

        // This is the id for current element to identify the chilren
        let current_id = sys.elements.insert(DomComponent::Element(el));
        let children = self.parse_elements(sys, Some(current_id))?;
        // We need to get the element back since it's moved
        let el = sys.elements.get_mut(current_id).unwrap();
        // We need to unwrap it now
        match el {
            DomComponent::Element(el) => el.children = children,
            _ => unreachable!(),
        }
        Ok(current_id)
    }

    fn parse_elements(
        &mut self,
        sys: &mut DomSystem,
        parent: Option<Id>,
    ) -> Result<Vec<Id>, xml::reader::Error> {
        let mut ids = Vec::new();
        loop {
            match self.reader.next() {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    ids.push(self.parse_element_open(
                        name.to_string(),
                        attributes.as_slice(),
                        sys,
                        parent,
                    )?);
                }
                Ok(XmlEvent::EndElement { .. }) | Ok(XmlEvent::EndDocument) => return Ok(ids),
                Ok(XmlEvent::Characters(text)) => {
                    let text = Text { text };
                    let id = sys.elements.insert(DomComponent::Text(text));
                    ids.push(id);
                }
                Err(e) => return Err(e),
                _ => {}
            }
        }
    }
}
#[derive(Debug)]
pub struct DomSystem {
    elements: BucketArray<DomComponent>,
    root: Id,
}

impl DomSystem {
    pub fn from_xml(xml: &str) -> Result<Self, xml::reader::Error> {
        let bytes = xml.as_bytes();
        XmlToDom::new(bytes).parse()
    }

    pub fn root(&self) -> Id {
        return self.root;
    }

    /// Warning: Returns text included
    pub fn firstlevel_components(&self, origin: Id) -> Option<&Vec<Id>> {
        let root = self.elements.get(origin)?;

        match root {
            DomComponent::Element(e) => Some({ &e.children }),
            _ => None,
        }
    }

    /// Returns None if origin element is a text or doesn't exist
    /// NOTE: elements after "original" origin are filtered
    pub fn compose_children(&self, origin: Id) -> Option<Vec<Id>> {
        let root = self.elements.get(origin)?;
        match root {
            DomComponent::Element(e) => Some({
                // A union of elements and corresponding children elements
                let mut children: Vec<_> = e
                    .children
                    .iter()
                    .filter(|child| {
                        matches!(
                            self.elements.get(**child).unwrap(),
                            DomComponent::Element(_)
                        )
                    })
                    .copied()
                    .collect();
                children.append(
                    &mut e
                        .children
                        .iter()
                        .map(|id| self.compose_children(*id))
                        .filter(|e| e.is_some())
                        .flatten()
                        .flatten()
                        .collect::<Vec<_>>(),
                );
                children
            }),
            _ => None,
        }
    }

    /// WARNING: Includes text
    pub fn get_component(&self, origin: Id) -> Option<&DomComponent> {
        self.elements.get(origin)
    }

    /// WARNING: Includes text
    pub fn get_component_mut(&mut self, origin: Id) -> Option<&mut DomComponent> {
        self.elements.get_mut(origin)
    }

    pub fn get_element(&self, origin: Id) -> Option<&Element> {
        return self
            .elements
            .get(origin)
            .map(|e| {
                if let DomComponent::Element(el) = e {
                    Some(el)
                } else {
                    None
                }
            })
            .flatten();
    }

    pub fn get_element_mut(&mut self, origin: Id) -> Option<&mut Element> {
        return self
            .elements
            .get_mut(origin)
            .map(|e| {
                if let DomComponent::Element(el) = e {
                    Some(el)
                } else {
                    None
                }
            })
            .flatten();
    }

    /// Returns `None` if there's no such origin element, or the origin element doesn't exist  
    pub fn get_elements_by_class_name(&self, origin: Id, class_name: &str) -> Option<Vec<Id>> {
        let children = self.compose_children(origin);
        Some(
            children?
                .iter()
                .chain([origin].iter())
                .copied()
                .filter(|id| match self.elements.get(*id).unwrap() {
                    DomComponent::Element(element) => {
                        return element
                            .class_list
                            .iter()
                            .find(|e| e.as_str() == class_name)
                            .is_some();
                    }
                    _ => unreachable!(),
                })
                .collect(),
        )
    }

    pub fn get_elements_by_tag_name(&self, origin: Id, tagname: &str) -> Option<Vec<Id>> {
        Some(
            self.compose_children(origin)?
                .iter()
                .chain([origin].iter())
                .copied()
                .filter(|id| match self.elements.get(*id).unwrap() {
                    DomComponent::Element(element) => return element.tag.as_str() == tagname,
                    _ => unreachable!(),
                })
                .collect(),
        )
    }

    pub fn get_element_by_id(&self, origin: Id, identifier: &str) -> Option<Id> {
        self.compose_children(origin)?
            .iter()
            .chain([origin].iter())
            .copied()
            .find(|id| match self.elements.get(*id).unwrap() {
                DomComponent::Element(element) => {
                    return element
                        .id
                        .as_ref()
                        .map(|e| e.as_str() == identifier)
                        .unwrap_or(false);
                }
                _ => unreachable!(),
            })
    }
}

#[test]
fn test_dom_parser() {
    let input = r#"<?xml version="1.0" encoding="utf-8"?><div param="a">test<span></span>another text</div>"#;
    let sys = DomSystem::from_xml(input).unwrap();
    let el = sys.elements.get(sys.root).unwrap();
    match el {
        DomComponent::Element(el) => {
            assert_eq!(el.tag, "div");
            assert_eq!(el.attrs.get("param").unwrap(), "a");
            dbg!(&el.children);
            match sys.elements.get(el.children[0]).unwrap() {
                DomComponent::Text(t) => assert_eq!(t.text, "test"),
                e => panic!(format!("{:?}", e)),
            }
            match sys.elements.get(el.children[1]).unwrap() {
                DomComponent::Element(t) => assert_eq!(t.tag, "span"),
                e => panic!(format!("{:?}", e)),
            }
            match sys.elements.get(el.children[2]).unwrap() {
                DomComponent::Text(t) => assert_eq!(t.text, "another text"),
                e => panic!(format!("{:?}", e)),
            }
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_dom_parser_selector() {
    let input = r#"<?xml version="1.0" encoding="utf-8"?>
<root>
    <div id="outter">
        <div class="inner">
        </div>
        <div class="inner other-class">
            <div class="inner">
            </div>
        </div>
        <div class="inner">
        </div>
    </div>
    rstrstin
    rst
    <div class="inner"></div>
</root>
    "#;

    let sys = DomSystem::from_xml(input).unwrap();

    let el = sys.get_elements_by_tag_name(sys.root, "div").unwrap();
    assert_eq!(el.len(), 6);

    let el = sys.get_elements_by_tag_name(sys.root, "root").unwrap();
    assert_eq!(el.len(), 0);

    let el = sys.get_elements_by_class_name(sys.root, "inner").unwrap();
    assert_eq!(el.len(), 5);

    let outter = sys.get_element_by_id(sys.root, "outter").unwrap();
    let el = sys.get_elements_by_class_name(outter, "inner").unwrap();
    assert_eq!(el.len(), 4);
}
