
/// THIS DOCUMENT IS A STUB!!

use std::{collections::HashMap, fs::File};
use std::io::BufReader;
use crate::{atoms::Id, css_parser::Css, dom_repr::DomComponent};
use crate::dom_repr::DomSystem;
use xml::reader::{EventReader, XmlEvent};
use crate::{css_matcher::match_selector_against_dom, ui::*};
use crate::css_gen::generate_from;



fn make_error(message: String, path: &str, pathcss: &str) -> UiSystem {
    return Item::build().padding(10., 10.)
        .style(StyleRuleTag::Color, StyleRule::Color { color: (255, 0, 0, 255) })
        .style(StyleRuleTag::BackgroundColor, StyleRule::Color { color: (255, 255, 255, 255) })
        .with_children(vec![
            Item::build()
                .padding(10., 10.)
                .with_children(vec![
                    Item::build().component(Ui::Text { text: "Whoops! An error:".to_string() })
                ]),
            Item::build().component(Ui::Text { text: message }),
            Item::build(),
            Item::build().component(Ui::Text { text: "-----------------------".to_string() }),
            Item::build(),
            Item::build().component(Ui::Text { text: format!("parse_xml(\"{}\", \"{}\")", path, pathcss) })
        ]).into_ui()
}

fn build_recursively(system: &DomSystem, root: Id, styles: &HashMap<Id, Vec<usize>>, css: &Css) -> Item {
    let components = system.firstlevel_components(root).unwrap();
    let element = system.get_element(root).unwrap();
    let mut item = match element.tag.as_str() {
        "Span" => Item::build().style(StyleRuleTag::Display, StyleRule::Display(DisplayType::Inline)),
        _ => Item::build(),
    };

    let mut children = vec![];
    let mut is_first = true;
    for component_id in components {
        let component = system.get_component(*component_id).unwrap();
        if let Some(style_list) = styles.get(&root) {
            for block_id in style_list {
                let block = &css.blocks[*block_id];
                let mut genertated = generate_from(&block.rules);
                for style in genertated.drain(..) {
                    item = item.style(style.0, style.1);
                }
            }
        }
        match component {
            DomComponent::Element(el) => {
                children.push(build_recursively(system, *component_id, styles, css));
            },
            DomComponent::Text(t) => {
                let prepend = if t.text.chars().next() == Some(' ') && !is_first { " " } else { "" };
                let append = if t.text.chars().last() == Some(' ') { " " } else { "" };

                let target = format!("{}{}{}", prepend, crate::util::remove_trailing_spaces(t.text.as_str()), append);
                children.push(Item::build().component(Ui::Text { text: target }));
            }
        }
        is_first = false;
    }

    item = item.with_children(children);
    item
}

pub fn parse_xml(path: &str) -> crate::ui::UiSystem {
    let style_file = std::fs::read_to_string("./style.css").unwrap();
    let mut stylesheet = match crate::css_parser::CssParser::new(style_file.as_str()).parse() {
        Ok(x) => x,
        Err(e) => return make_error(format!("{:?}", e), path, "./style.css")
    };

    let file = match std::fs::read_to_string(path) {
        Ok(x) => x,
        Err(e) => return make_error(e.to_string(), path, "./style.css")
    };

    // Create a DOM system
    let mut system = match DomSystem::from_xml(&file) {
        Ok(x) => x,
        Err(e) => return make_error(e.to_string(), path, "./style.css")
    };

    let mut styles_for_elements: HashMap<Id, Vec<usize>> = HashMap::new();

    for (index, block) in stylesheet.blocks.iter().enumerate() {
        let matching = match_selector_against_dom(&block.selector, &system);
        for i in matching {
            if styles_for_elements.contains_key(&i) {
                styles_for_elements.get_mut(&i).unwrap().push(index)
            }
            else {
                styles_for_elements.insert(i, vec![index]);
            }
        }
    }

    let items = build_recursively(&system, system.root(), &styles_for_elements, &stylesheet);
    items.into_ui()
}
