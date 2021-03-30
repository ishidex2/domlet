#[derive(Debug)]
pub enum Ui {
    Button,
    Text { text: String },
    Div,
}

#[derive(Debug)]
pub struct Transform {
    pub pos: crate::atoms::Vec2f
}

use std::collections::hash_map::*;

pub type UiId = usize;

#[derive(Debug)]
pub struct UiElem {
    pub elem: Ui,
    pub styles: HashMap<StyleRuleTag, Style>,
    pub parent: Option<UiId>,
    pub children: Vec<UiId>,
}

pub struct ComputedStyle {
    padding: Vec2f,
    margin: Vec2f,
}

macro_rules! extract {
    ($v:expr, $p:pat => $r:expr) => {
        match $v { $p => $r, _ => unreachable!() }
    };
}


impl UiElem {
    fn get_style(&self, style: StyleRuleTag) -> Style {
        *self.styles.get(&style).unwrap_or(&style.default())
    }

    fn computed_style(&self) -> ComputedStyle {
        let padding = self.get_style(StyleRuleTag::Padding).rule;
        let margin = self.get_style(StyleRuleTag::Margin).rule;
        let border = self.get_style(StyleRuleTag::Border).rule;
        ComputedStyle { padding: extract!(padding, StyleRule::Offset{l, t, r: _, b: _} => Vec2f::new(l, t)),
                        margin:  extract!(margin, StyleRule::Offset{l, t, r: _, b: _} => Vec2f::new(l, t))
                                +extract!(border, StyleRule::Outline{ size, .. } => Vec2f::new(size, size)) }
    }

    // TEMPORARY

    pub fn get_bg(&self) -> (u8, u8, u8, u8) {
        extract!(self.get_style(StyleRuleTag::BackgroundColor).rule, StyleRule::Color { color } => color)
    }

    pub fn get_border(&self) -> (f32, (u8, u8, u8, u8)) {
        extract!(self.get_style(StyleRuleTag::Border).rule, StyleRule::Outline { size, color } => (size, color))
    }

    pub fn get_fg(&self) -> (u8, u8, u8, u8) {
        extract!(self.get_style(StyleRuleTag::Color).rule, StyleRule::Color { color } => color)
    }

    pub fn display(&self) -> DisplayType {
        extract!(self.get_style(StyleRuleTag::Display).rule, StyleRule::Display(d) => d)   
    }

    pub fn stretch(&self) -> StretchType {
        extract!(self.get_style(StyleRuleTag::Stretch).rule, StyleRule::Stretch(d) => d)   
    }
}


#[derive(Debug)]
pub struct Item {
    elem: Ui,
    styles: HashMap<StyleRuleTag, Style>,
    pub children: Vec<Item>,
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum StyleRuleTag {
    Padding,
    Margin,
    Border,
    BackgroundColor,
    Color,
    Display,
    Stretch
}

impl StyleRuleTag {
    pub fn name(self) -> &'static str {
        use StyleRuleTag::*;

        match self {
            Padding => "padding",
            Margin => "margin",
            Border => "border",
            BackgroundColor => "background-color",
            Color => "color",
            Display => "display",
            Stretch => "stretch"
        }
    }

    pub fn from(s: &str) -> Option<StyleRuleTag> {
        use StyleRuleTag::*;

        Some(match s {
            "padding" => Padding,
            "margin" => Margin,
            "border" => Border,
            "background" | "background-color" => BackgroundColor,
            "color" => Color,
            "display" => Display,
            "stretch" => Stretch,
            _ => return None
        })
    }

    pub fn default(self) -> Style {
        use StyleRuleTag::*;

        Style { propagating: matches!(self, Color), rule: match self {
            Padding => StyleRule::Offset { l: 0., t: 0., b: 0., r: 0. },
            Margin => StyleRule::Offset { l: 0., t: 0., b: 0., r: 0. },
            Border => StyleRule::Outline { size: 0.0, color: (0, 0, 0, 0) },
            BackgroundColor => StyleRule::Color { color: (255, 255, 255, 0) },
            Color => StyleRule::Color { color: (0, 0, 0, 255) },
            Display => StyleRule::Display(DisplayType::Block),
            Stretch => StyleRule::Stretch(StretchType::IfBlock)
        }}
    }
}


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DisplayType {
    Inline,
    Block
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StretchType {
    IfBlock,
    True,
    False
}

#[derive(Debug, Clone, Copy)]
pub enum StyleRule {
    Offset {l: f32, t: f32, r: f32, b: f32},
    Outline {size: f32, color: (u8, u8, u8, u8)},
    Color {color: (u8, u8, u8, u8)},
    Display(DisplayType),
    Stretch(StretchType)
}

impl StyleRule {
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub rule: StyleRule,
    pub propagating: bool
}


use crate::atoms::*;
use crate::bucket_array::*;

impl Item {

    pub fn build() -> Self {
        Self { elem: Ui::Div, styles: HashMap::new(), children: vec![] }
    }
    
    pub fn component(mut self, c: Ui) -> Self {
        if matches!(c, Ui::Text { .. }) {
            self.styles.insert(StyleRuleTag::Display, Style { propagating: false, rule: StyleRule::Display(DisplayType::Inline) });
        }
        self.elem = c;
        self
    }

    pub fn style(mut self, tag: StyleRuleTag, style: StyleRule) -> Self {
        let propagating =  matches!(tag, StyleRuleTag::Color);
        self.styles.insert(tag, Style { rule: style, propagating });
        self
    }

    pub fn padding(mut self, x: f32, y: f32) -> Self {
        self.styles.insert(StyleRuleTag::Padding, Style { rule: StyleRule::Offset { l: x, t: y, r: x, b: y }, propagating: false });
        self
    }

    pub fn with_children(mut self, v: Vec<Item>) -> Self {
        self.children = v;
        self
    }

    fn into_elem(mut self, things: &mut BucketArray<UiElem>, parent: Option<UiId>) -> UiId {
        let o = UiElem { elem: self.elem, styles: self.styles, children: vec![], parent: parent};
        let mut children: Vec<_> = self.children.drain(..).map(|e| e.into_elem(things, None)).collect();
        let pid = things.insert(o);
        for child in children.iter() {
            things.get_mut(*child).unwrap().parent = Some(pid);
        }
        things.get_mut(pid).unwrap().children = children.drain(..).collect();
        pid
    }

    pub fn into_ui(self) -> UiSystem {
        let mut arr = BucketArray::new();
        UiSystem { 
            root: self.into_elem(&mut arr, None),
            things: arr
        }
    }
}


pub struct UiSystem {
    pub things: BucketArray<UiElem>,
    pub root: UiId
}

struct LayoutCalculator<'a> {
    sys: &'a UiSystem,
    result: Vec<Frame>
}

pub struct Frame {
    pub rect: Rectf,
    pub zindex: usize,
    pub for_id: UiId,
}

impl<'a> LayoutCalculator<'a> {
    fn new(uisys: &'a UiSystem) -> Self {
        Self { sys: uisys, result: vec![] }
    }

    fn calculate_layout(&mut self) -> Vec<Frame> {
        self.diverge(self.sys.root, Vec2f::new(0., 0.), 0);
        self.resize_children(self.sys.root);
        self.result.drain(..).collect()
    }

    fn resize_children(&mut self, id: UiId) {
        let parent_frame_rect = self.result.iter().find(|e| e.for_id == id).unwrap().rect;
        let thiselem = self.sys.things.get(id).unwrap();

        for i in self.result.iter_mut() {
            if self.sys.things.get(i.for_id).unwrap().parent == Some(id) {
                let elem = self.sys.things.get(i.for_id).unwrap();
                match (elem.stretch(), elem.display()) {
                    (StretchType::IfBlock, DisplayType::Block) | (StretchType::True, DisplayType::Block) => i.rect.size.x = parent_frame_rect.size.x-thiselem.computed_style().padding.x*2.-elem.computed_style().margin.x*2.0,
                    _ => {}
                }
            }
        }

        for i in thiselem.children.iter() {
            self.resize_children(*i);
        }
    }

    fn diverge(&mut self, id: UiId, offset: Vec2f, depth: usize) -> Vec2f {
        let elem = self.sys.things.get(id).unwrap();
        let computed = elem.computed_style();
        let global_offset = offset+computed.margin;
        let mut size = Vec2f::new(0., 0.);
        let mut predecessor = DisplayType::Block;
        let mut min_y = 0.;
        let mut acc_x = 0_f32;
        for i in elem.children.iter() {
            match self.sys.things.get(*i).unwrap().display() {
                DisplayType::Block => {
                    let inner_size = self.diverge(*i, global_offset+computed.padding+Vec2f::new(0., size.y), depth + 1);
                    size.x = size.x.max(inner_size.x);
                    size.y += inner_size.y;
                    min_y = size.y;
                    acc_x = 0.;
                    predecessor = DisplayType::Block;
                },
                DisplayType::Inline => {
                    if predecessor == DisplayType::Block {
                        let inner_size = self.diverge(*i, global_offset+computed.padding+Vec2f::new(0., size.y), depth + 1);
                        size.x = size.x.max(inner_size.x);
                        size.y += inner_size.y;
                        acc_x += inner_size.x;
                    }
                    else {
                        let inner_size = self.diverge(*i, global_offset+computed.padding+Vec2f::new(acc_x, min_y), depth + 1);
                        acc_x += inner_size.x;
                        size.y = size.y.max(inner_size.y);
                        if acc_x > size.x {
                            size.x += inner_size.x;
                        }
                    };
                    predecessor = DisplayType::Inline;
                }
            }
        }
        // Additional size factors
        size = size + match &elem.elem {
            Ui::Text {text} => Vec2f::new(text.len() as f32*8., 16.),
            _ => Vec2f::new(0., 0.)
        };
        // Padding
        size = size + computed.padding*2.;
        self.result.push(Frame { rect: Rect { pos: global_offset, size: size }, zindex: depth, for_id: id });
        return size+computed.margin*2.;
    }
}

impl UiSystem {
    pub fn calculate_layout(&self) -> Vec<Frame> {
        LayoutCalculator::new(self).calculate_layout()
    }

    pub fn propagate_styles_rec(&mut self, id: UiId, mut propagate: HashMap<StyleRuleTag, Style>) {
        for (key, style) in propagate.iter() {
            if matches!(key, StyleRuleTag::BackgroundColor) {
                unreachable!();
            }
            if !self.things.get(id).unwrap().styles.contains_key(&key) {
                self.things.get_mut(id).unwrap().styles.insert(*key, *style);
            }
        }
        let this = self as *mut Self;
        let styles = &mut self.things.get(id).unwrap().styles.iter().filter(|(_, e)| e.propagating).map(|(k, e)| (*k, *e));
        propagate.extend(styles);
        let children = self.things.get(id).unwrap().children.iter().copied();
        for child in children {
            unsafe {
                (*this).propagate_styles_rec(child, propagate.clone());
            }
        }
    }
}