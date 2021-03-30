use crate::css_parser::*;

use crate::ui::{StyleRule, StyleRuleTag, DisplayType, StretchType};

enum ExpectingValue {
    Unit,
    Color,
}

struct Generator<'a>  {
    vals: &'a [ExpectingValue],
    callback: fn (units: &[CssRuleParam]) -> Vec<(StyleRuleTag, StyleRule)>
}

impl<'a> Generator<'a> {
    fn new(vals: &'a [ExpectingValue], callback: fn (units: &[CssRuleParam]) -> Vec<(StyleRuleTag, StyleRule)>) -> Self {
        Generator { vals, callback }
    }
}

fn params_into_offset(params: &[CssRuleParam]) -> Option<StyleRule> {
    match params.len() {
        1 => {
            let p = params.get(0)?.into_px()?;
            Some(StyleRule::Offset { l: p, t: p, r: p, b: p })
        }
        2 => {
            let h = params.get(0)?.into_px()?;
            let v = params.get(1)?.into_px()?;
            Some(StyleRule::Offset { l: h, t: v, r: h, b: v })
        }
        4 => {
            let l = params.get(0)?.into_px()?;
            let t = params.get(1)?.into_px()?;
            let r = params.get(2)?.into_px()?;
            let b = params.get(3)?.into_px()?;
            Some(StyleRule::Offset { l, t, r, b })
        }
        _ => None
    }
}

fn generate_rule(rule: &CssRule) -> Option<(StyleRuleTag, StyleRule)> {
    let n = StyleRuleTag::from(rule.name.as_str())?;
    Some((n, match n {
        StyleRuleTag::BackgroundColor | StyleRuleTag::Color => {
            StyleRule::Color { color: rule.params.get(0)?.into_color()? }
        },
        StyleRuleTag::Padding | StyleRuleTag::Margin => {
            params_into_offset(&rule.params)?
        },
        StyleRuleTag::Border => {
            StyleRule::Outline { size: rule.params.get(0)?.into_px()?, color: rule.params.get(1)?.into_color()?, }
        },
        StyleRuleTag::Display => {
            StyleRule::Display(match rule.params.get(0)?.into_ident()? {
                "block" => DisplayType::Block,
                "inline" => DisplayType::Inline,
                _ => None?
            })
        },
        StyleRuleTag::Stretch => {
            StyleRule::Stretch(match rule.params.get(0)?.into_ident()? {
                "yes" | "true" => StretchType::True,
                "false" | "no" => StretchType::False,
                _ => None?
            })
        },
        // _ => return None
    }))
}

pub fn generate_from(rules: &[CssRule]) -> Vec<(StyleRuleTag, StyleRule)> {
    let mut styles = vec![];
    for rule in rules {
        styles.push(match generate_rule(rule) {
            Some(x) => x,
            _ => continue
        });
    }
    styles
}
