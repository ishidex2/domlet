
use std::fs::File;
use std::io::BufReader;

use xml::reader::{EventReader, XmlEvent};
use crate::ui::*;

pub fn trim_bs(s: String) -> String
{
    s.replace("\n", "").split(' ').filter(|&e| e != "").collect::<Vec<&str>>().join(" ")
}

pub fn parse_color(s: &str) -> (u8, u8, u8, u8) {
    match s {
        "red" => (255, 0, 0, 255),
        "green" => (0, 255, 0, 255),
        "blue" => (0, 0, 255, 255),
        "lightblue" => (128, 128, 255, 255),
        "white" => (255, 255, 255, 255),
        "black" => (0, 0, 0, 255),
        "yellow" => (255, 255, 0, 255),
        _ => (0, 0, 0, 0)
    }
}

pub fn parse_css(src: &str) -> Option<Vec<(StyleRuleTag, StyleRule)>> {
    let stmts = src.split(";");
    let mut res = vec![];
    for statement in stmts {
        if trim_bs(statement.to_string()) == "" {
            continue;
        }
        let mut s = statement.split(":");
        let rule = match StyleRuleTag::from(&trim_bs(s.next()?.to_string())) {
            Some(x) => x,
            None => continue
        };
        let rdata = trim_bs(s.next()?.to_string());
        let mut data = rdata.split(" ");
        match rule {
            StyleRuleTag::Padding | StyleRuleTag::Margin => {
                let mut dims = vec![];
                while let Some(num) = data.next() {
                    dims.push(num.parse::<i32>().ok()? as f32);
                }
                let (l, t, r, b) = match dims.len() {
                    1 => {
                        (dims[0], dims[0], dims[0], dims[0])
                    }
                    2 => {
                        (dims[0], dims[1], dims[0], dims[1])
                    }
                    4 => {
                        (dims[0], dims[1], dims[2], dims[3])
                    }
                    _ => return None
                };
                res.push((rule, StyleRule::Offset {l, t, r, b}));
            },
            StyleRuleTag::Display => {
                res.push((rule, StyleRule::Display(match data.next()? {
                    "block" => DisplayType::Block,
                    "inline" => DisplayType::Inline,
                    _ => continue
                })));
            },
            StyleRuleTag::Stretch => {
                res.push((rule, StyleRule::Stretch(match data.next()? {
                    "true" | "yes" => StretchType::True,
                    "false" | "no" => StretchType::False,
                    _ => continue
                })));
            },
            StyleRuleTag::Border => {
                res.push((rule, StyleRule::Outline {size: data.next()?.parse::<f32>().ok()?, color: parse_color(data.next()?)}));
            },
            StyleRuleTag::Color | StyleRuleTag::BackgroundColor => {
                res.push((rule, StyleRule::Color {color: parse_color(data.next()?)}));
            },
            _ => {}
        }

    }
    Some(res)
}

pub fn parse_xml(path: &str) -> crate::ui::UiSystem {
    let file = match File::open(path) {
        Ok(x) => x,
        Err(e) => return Item::build().padding(10., 10.)
        .style(StyleRuleTag::Color, StyleRule::Color { color: (255, 0, 0, 255) })
        .style(StyleRuleTag::BackgroundColor, StyleRule::Color { color: (255, 255, 255, 255) })
        .with_children(vec![
            Item::build()
            .padding(10., 10.)
            .with_children(vec![
                Item::build().component(Ui::Text { text: "Whoops! An error:".to_string() })
            ]),
            Item::build().component(Ui::Text { text: e.to_string() }),
            Item::build(),
            Item::build().component(Ui::Text { text: "-----------------------".to_string() }),
            Item::build(),
            Item::build().component(Ui::Text { text: format!("parse_xml(\"{}\")", path) })
        ]).into_ui()
    };
    let file = BufReader::new(file);

    let mut raw_depth = 0;

    let parser = EventReader::new(file);

    let mut stack = vec![Item::build()];

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, ..}) => {
                let mut it = Item::build();
                it = match (name.to_string()).as_str() {
                    "Div" => it.component(Ui::Div),
                    "Button" => it.component(Ui::Button),
                    "Span" => it.component(Ui::Div).style(StyleRuleTag::Display, StyleRule::Display(DisplayType::Inline)),
                    "Raw" => {raw_depth += 1; it.style(StyleRuleTag::Display, StyleRule::Display(DisplayType::Inline))}
                    _ => it.component(Ui::Div)
                };
                for i in attributes {
                    if &(i.name.to_string()) == "style" {
                        let styles = match parse_css(&i.value) { Some(t) => t, None => continue };
                        for style in styles.iter() {
                            it = it.style(style.0, style.1);
                        }
                    }
                }
                stack.push(it);
                let sl = stack.len()-1;
            },
            Ok(XmlEvent::Whitespace(s)) => {
                if raw_depth > 0 {
                    let le = stack.len()-1;
                    stack.get_mut(le).unwrap().children.push(Item::build().component(Ui::Text { text: s }));   
                }
            }
            Ok(XmlEvent::Characters(s)) => {
                let string = if raw_depth > 0 {
                    s
                }
                else {
                    let mut prep = "";
                    let mut app = "";
                    if s.chars().next().unwrap_or('.') == ' ' {
                        prep = " ";
                    }
                    if s.chars().last().unwrap_or('.') == ' ' {
                        app = " ";
                    }
                    String::from(prep)+&trim_bs(s)+app
                };
                let le = stack.len()-1;
                stack.get_mut(le).unwrap().children.push(Item::build().component(Ui::Text { text: string }));
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == "Raw" {
                    raw_depth -= 1;
                }
                let it = stack.pop().unwrap();
                let le = stack.len()-1;
                stack.get_mut(le).unwrap().children.push(it);
            }
            Err(e) => {
                return Item::build().padding(10., 10.)
                .style(StyleRuleTag::Color, StyleRule::Color { color: (255, 0, 0, 255) })
                .style(StyleRuleTag::BackgroundColor, StyleRule::Color { color: (255, 255, 255, 255) })
                .with_children(vec![
                    Item::build()
                    .padding(10., 10.)
                    .with_children(vec![
                        Item::build().component(Ui::Text { text: "Whoops! An error:".to_string() })
                    ]),
                    Item::build().component(Ui::Text { text: e.to_string() }),
                    Item::build(),
                    Item::build().component(Ui::Text { text: "-----------------------".to_string() }),
                    Item::build(),
                    Item::build().component(Ui::Text { text: format!("parse_xml(\"{}\")", path) })
                ]).into_ui();
            }
            _ => {}
        }
    }
    return stack.pop().unwrap().into_ui();
}