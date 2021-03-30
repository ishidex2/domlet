use std::collections::BTreeSet;

use crate::{
    atoms::Id,
    css_parser::{CssParser, CssSelectorAtom, CssSelectorMultiple},
    dom_repr::DomSystem,
};

pub trait QuerySelectorExt {
    fn query_selector(&self, selector: &str) -> BTreeSet<Id>;
}

impl QuerySelectorExt for DomSystem {
    fn query_selector(&self, selector: &str) -> BTreeSet<Id> {
        if let Ok(selector) = CssParser::new(selector).parse_selector() {
            match_selector_against_dom(&selector, self)
        } else {
            BTreeSet::new()
        }
    }
}

// We want to match a query selector (valid css selector) against the DOM
pub fn match_selector_against_dom(
    selector: &CssSelectorMultiple,
    against: &DomSystem,
) -> BTreeSet<Id> {
    let mut multiple_set: BTreeSet<Id> = BTreeSet::new();

    // After connection
    for multiple in &selector.sels {
        let mut composite_subset: BTreeSet<Id> = against.compose_children(against.root()).unwrap().drain(..).collect();
        composite_subset.insert(against.root());
        for composite in &multiple.sels {
            match composite {
                CssSelectorAtom::Class(class_name) => {
                    let elements = against
                        .get_elements_by_class_name(against.root(), class_name)
                        .unwrap();
                    composite_subset = composite_subset
                        .intersection(&elements.iter().cloned().collect())
                        .cloned()
                        .collect();
                }
                CssSelectorAtom::Tag(tag_name) => {
                    let elements = against
                        .get_elements_by_tag_name(against.root(), tag_name)
                        .unwrap();
                    composite_subset = composite_subset
                        .intersection(&elements.iter().cloned().collect())
                        .cloned()
                        .collect();
                }
                CssSelectorAtom::Id(id) => {
                    if let Some(element) = against.get_element_by_id(against.root(), id) {
                        composite_subset = composite_subset
                            .intersection(&[element].iter().cloned().collect())
                            .cloned()
                            .collect();
                        assert_eq!(composite_subset.len(), 1);
                        break;
                    }
                    else {
                        composite_subset.clear();
                        break;
                    }
                }
                _ => {
                    unimplemented!()
                }
            }
        }

        multiple_set = multiple_set.union(&composite_subset).copied().collect();
    }

    multiple_set
}

#[test]
fn query_selector_test() {
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

    <div class="inner"></div>
</root>
    "#;
 
    let sys = DomSystem::from_xml(input).unwrap();
 
    let elements = sys.query_selector("#outter");
    dbg!(&elements);
    assert_eq!(elements.len(), 1);

    let elements = sys.query_selector(".inner.other-class");
    dbg!(&elements);
    assert_eq!(elements.len(), 1);

    let elements = sys.query_selector(".inner");
    dbg!(&elements);
    assert_eq!(elements.len(), 5);

    let elements = sys.query_selector("div");
    dbg!(&elements);
    assert_eq!(elements.len(), 6);
}
