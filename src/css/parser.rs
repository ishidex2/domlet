/*!
 *
 *      Parser for CSS files
 *
 */

use std::fmt::Debug;

#[derive(Debug)]
pub enum CssSimpleSelector {
    Id(String),
    Class(String),
    Tag(String),
}

#[derive(Debug)]
pub struct CssCompositeSelector {
    pub sels: Vec<CssSimpleSelector>,
}

#[derive(Debug)]
pub struct CssVariantSelector {
    pub sels: Vec<CssCompositeSelector>,
}

pub type CssSelector = CssVariantSelector;

impl std::fmt::Display for CssSimpleSelector {
    fn fmt<'a>(&self, fmt: &mut std::fmt::Formatter<'a>) -> std::fmt::Result {
        use CssSimpleSelector::*;
        match self {
            Class(s) => write!(fmt, ".{}", s),
            Id(s) => write!(fmt, "#{}", s),
            Tag(s) => write!(fmt, "{}", s.clone()),
        }
    }
}

impl CssSimpleSelector {
    pub fn to_string(&self) -> String {
        return format!("{}", self);
    }
}

#[derive(Debug)]
pub enum CssUnit {
    Px,
}

#[derive(Debug)]
pub enum CssRuleParam {
    Color(u8, u8, u8, u8),
    UnknownIdent(String),
    Unit(f64, CssUnit),
}

impl CssRuleParam {
    pub fn into_color(&self) -> Option<(u8, u8, u8, u8)> {
        if let CssRuleParam::Color(r, g, b, a) = self {
            return Some((*r, *g, *b, *a));
        }
        None
    }

    pub fn into_ident(&self) -> Option<&str> {
        if let CssRuleParam::UnknownIdent(str) = self {
            return Some(str.as_str());
        }
        None
    }

    pub fn into_px(&self) -> Option<f64> {
        if let CssRuleParam::Unit(f, _) = self {
            return Some(*f);
        }
        None
    }
}

#[derive(Debug)]
pub struct CssRule {
    pub name: String,
    pub params: Vec<CssRuleParam>,
}

pub type CssRuleset = Vec<CssRule>;

#[derive(Debug)]
pub struct CssBlock {
    pub selector: CssVariantSelector,
    pub rules: CssRuleset,
}

#[derive(Debug)]
pub struct Css {
    pub blocks: Vec<CssBlock>,
}

pub enum CssErrorKind {
    Eof,
    InvalidSymbolInIdent,
    ExpectedCharacter(char),
    UnknownUnit(String),
    UnexpectedIdent(String),
    InvalidRuleParameter,
    InvalidNumber,
    HexColorLengthMayNotBe(usize),
}

impl Debug for CssErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CssErrorKind::*;

        write!(
            f,
            "{}",
            match self {
                Eof => "Unexpected end of file".to_string(),
                InvalidSymbolInIdent => "Invalid symbol in identifier".to_string(),
                ExpectedCharacter(ch) => format!("Expected character '{}'", ch),
                UnknownUnit(s) => format!("Unknown unit '{}'", s),
                UnexpectedIdent(s) => format!("Unexpected identifier '{}'", s),
                HexColorLengthMayNotBe(count) =>
                    format!("Hexadecimal color may not be of length {}", count),
                InvalidRuleParameter => "Invalid rule parameter".to_string(),
                InvalidNumber => "InvalidNumber".to_string(),
            }
        )
    }
}
pub struct CssError {
    pub line: usize,
    pub col: usize,
    pub kind: CssErrorKind,
}

impl Debug for CssError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CSS error: {:?} at {}:{}",
            self.kind,
            self.line + 1,
            self.col + 1
        )
    }
}
pub struct CssParser<'a> {
    chars: std::str::Chars<'a>,
    current: char,
    line: usize,
    col: usize,
    eof: bool,
}

impl<'a> CssParser<'a> {
    fn err(&self, kind: CssErrorKind) -> CssError {
        CssError {
            line: self.line,
            col: self.col,
            kind,
        }
    }

    // Check if value is a formatting value
    fn is_ignored(&self) -> bool {
        let peek = self.current;
        peek == ' ' || peek == '\n' || peek == '\t'
    }

    // This function is needed to remove unnecessary spaces, and called
    // Before each parser to make sure that there's no trailing spaces
    fn update_lines(&mut self) {
        self.col += 1;
        if self.current == '\n' {
            self.line += 1;
            self.col = 0;
        }
    }

    fn no_rubbish(&mut self) {
        // Remove all ignored characters
        while self.is_ignored() {
            self.skip();
        }

        if self.peek() == '/' {
            self.skip();
            if self.peek() == '*' {
                loop {
                    self.skip();
                    if self.peek() == '*' {
                        self.skip();
                        if self.peek() == '/' {
                            self.skip();
                            self.no_rubbish();
                            break;
                        }
                    }
                }
            }
        }
    }

    fn peek(&mut self) -> char {
        return self.current;
    }

    fn skip(&mut self) {
        if self.chars.as_str() == "" {
            // This character will indicate end of file since it's unused anyway
            self.current = '\0';
            self.eof = true;
            return;
        }
        self.current = self.chars.next().unwrap();
        self.update_lines();
    }

    fn next(&mut self) -> Result<char, CssError> {
        let previous = self.current;

        // We don't want to consume anything if there's end of file
        if self.eof {
            return Err(self.err(CssErrorKind::Eof));
        }

        self.skip();

        Ok(previous)
    }

    fn parse_ident(&mut self) -> Result<String, CssError> {
        let mut result = "".to_string();

        // Miscellaneous characters
        const IDENT_VALID: [char; 2] = ['_', '-'];

        // Check for alphabetic characters and miscellaneous
        while self.peek().is_alphabetic() || IDENT_VALID.contains(&self.peek()) {
            result.push(self.next()?);
        }

        // If there were no alphabetic characters the identifier is invalid
        if result.len() < 1 {
            return Err(self.err(CssErrorKind::InvalidSymbolInIdent));
        }

        // We may include numbers after that
        while self.peek().is_alphanumeric() || IDENT_VALID.contains(&self.peek()) {
            result.push(self.next()?);
        }

        self.no_rubbish();
        Ok(result)
    }

    fn parse_selector_atomic(&mut self) -> Result<CssSimpleSelector, CssError> {
        // Parse different selectors, we can't backtrack so we will have to manually skip the prefix symbols
        let selector = match self.peek() {
            '.' => {
                self.next()?;
                CssSimpleSelector::Class(self.parse_ident()?)
            }
            '#' => {
                self.next()?;
                CssSimpleSelector::Id(self.parse_ident()?)
            }
            _ => CssSimpleSelector::Tag(self.parse_ident()?),
        };

        Ok(selector)
    }

    fn parse_composite_selector(&mut self) -> Result<CssCompositeSelector, CssError> {
        let mut selectors = vec![self.parse_selector_atomic()?];
        while self.peek() == '.' || self.peek() == '#' {
            selectors.push(self.parse_selector_atomic()?);
        }
        Ok(CssCompositeSelector { sels: selectors })
    }

    fn parse_multiple_selector(&mut self) -> Result<CssVariantSelector, CssError> {
        let mut selectors = vec![self.parse_composite_selector()?];
        while self.peek() == ',' {
            self.skip_char(',')?;

            selectors.push(self.parse_composite_selector()?);
        }
        Ok(CssVariantSelector { sels: selectors })
    }

    pub fn parse_selector(&mut self) -> Result<CssVariantSelector, CssError> {
        self.parse_multiple_selector()
    }

    fn skip_char(&mut self, ch: char) -> Result<(), CssError> {
        if self.peek() == ch {
            self.next()?;
            self.no_rubbish();
            Ok(())
        } else {
            Err(self.err(CssErrorKind::ExpectedCharacter(ch)))
        }
    }

    fn parse_number(&mut self) -> Result<f64, CssError> {
        let mut res = "".to_string();

        if self.peek() == '-' {
            self.next()?;
            res.push('-');
        }

        while self.peek().is_digit(10) {
            res.push(self.next()?);
        }

        if self.peek() == '.' {
            self.next()?;
            res.push('.');
        }

        while self.peek().is_digit(10) {
            res.push(self.next()?);
        }

        if !(res == "." || res == "") {
            self.no_rubbish();
            return Ok(res.parse().unwrap());
        }
        return Err(self.err(CssErrorKind::InvalidNumber));
    }

    fn parse_unit_value(&mut self) -> Result<CssRuleParam, CssError> {
        let number = self.parse_number()?;
        let unit = self.parse_ident()?;

        if unit == "px" {
            return Ok(CssRuleParam::Unit(number, CssUnit::Px));
        }
        Err(self.err(CssErrorKind::UnknownUnit(unit)))
    }

    fn parse_ident_rule(&mut self) -> Result<CssRuleParam, CssError> {
        let ident = self.parse_ident()?;

        // For now we will always assume it's a color
        Ok(match ident.as_str() {
            "red" => CssRuleParam::Color(255, 0, 0, 255),
            "green" => CssRuleParam::Color(0, 255, 0, 255),
            "blue" => CssRuleParam::Color(0, 0, 255, 255),
            "lightblue" => CssRuleParam::Color(128, 128, 255, 255),
            "white" => CssRuleParam::Color(255, 255, 255, 255),
            "black" => CssRuleParam::Color(0, 0, 0, 255),
            "yellow" => CssRuleParam::Color(255, 255, 0, 255),
            _ => CssRuleParam::UnknownIdent(ident),
        })
    }

    fn parse_hex_color(&mut self) -> Result<CssRuleParam, CssError> {
        self.skip_char('#')?;
        let mut hex = "".to_string();
        while self.peek().is_digit(16) {
            hex.push(self.next()?);
        }
        self.no_rubbish();

        fn hex_to_u8(c: char) -> Option<u8> {
            Some(match c {
                '0' => 0,
                '1' => 1,
                '2' => 2,
                '3' => 3,
                '4' => 4,
                '5' => 5,
                '6' => 6,
                '7' => 7,
                '8' => 8,
                '9' => 9,
                'a' | 'A' => 0xA,
                'b' | 'B' => 0xB,
                'c' | 'C' => 0xC,
                'd' | 'D' => 0xD,
                'e' | 'E' => 0xE,
                'f' | 'F' => 0xF,
                _ => return None,
            })
        }

        match hex.len() {
            3 => {
                let mut iter = hex.chars();
                let r = hex_to_u8(iter.next().unwrap()).unwrap();
                let g = hex_to_u8(iter.next().unwrap()).unwrap();
                let b = hex_to_u8(iter.next().unwrap()).unwrap();
                return Ok(CssRuleParam::Color(r << 4 | r, g << 4 | g, b << 4 | b, 255));
            }
            4 => {
                let mut iter = hex.chars();
                let r = hex_to_u8(iter.next().unwrap()).unwrap();
                let g = hex_to_u8(iter.next().unwrap()).unwrap();
                let b = hex_to_u8(iter.next().unwrap()).unwrap();
                let a = hex_to_u8(iter.next().unwrap()).unwrap();
                dbg!(a << 4 | a);
                return Ok(CssRuleParam::Color(
                    r << 4 | r,
                    g << 4 | g,
                    b << 4 | b,
                    a << 4 | a,
                ));
            }
            6 => {
                let mut iter = hex.chars();
                let rh = hex_to_u8(iter.next().unwrap()).unwrap();
                let rl = hex_to_u8(iter.next().unwrap()).unwrap();
                let gh = hex_to_u8(iter.next().unwrap()).unwrap();
                let gl = hex_to_u8(iter.next().unwrap()).unwrap();
                let bh = hex_to_u8(iter.next().unwrap()).unwrap();
                let bl = hex_to_u8(iter.next().unwrap()).unwrap();
                return Ok(CssRuleParam::Color(
                    rh << 4 | rl,
                    gh << 4 | gl,
                    bh << 4 | bl,
                    255,
                ));
            }
            8 => {
                let mut iter = hex.chars();
                let rh = hex_to_u8(iter.next().unwrap()).unwrap();
                let rl = hex_to_u8(iter.next().unwrap()).unwrap();
                let gh = hex_to_u8(iter.next().unwrap()).unwrap();
                let gl = hex_to_u8(iter.next().unwrap()).unwrap();
                let bh = hex_to_u8(iter.next().unwrap()).unwrap();
                let bl = hex_to_u8(iter.next().unwrap()).unwrap();
                let ah = hex_to_u8(iter.next().unwrap()).unwrap();
                let al = hex_to_u8(iter.next().unwrap()).unwrap();
                return Ok(CssRuleParam::Color(
                    rh << 4 | rl,
                    gh << 4 | gl,
                    bh << 4 | bl,
                    ah << 4 | al,
                ));
            }
            _ => return Err(self.err(CssErrorKind::HexColorLengthMayNotBe(hex.len()))),
        }
    }
    // This function will redirect to smaller parsers
    fn parse_value(&mut self) -> Result<CssRuleParam, CssError> {
        if self.peek().is_digit(10) || self.peek() == '-' {
            self.parse_unit_value()
        } else if self.peek().is_alphabetic() {
            self.parse_ident_rule()
        } else if self.peek() == '#' {
            self.parse_hex_color()
        } else {
            Err(self.err(CssErrorKind::InvalidRuleParameter))
        }
    }

    fn parse_rule(&mut self) -> Result<CssRule, CssError> {
        let mut rule = CssRule {
            name: self.parse_ident()?,
            params: vec![],
        };

        self.skip_char(':')?;

        while self.peek() != ';' {
            rule.params.push(self.parse_value()?);
        }

        self.skip_char(';')?;

        Ok(rule)
    }

    pub fn new(source: &'a str) -> Self {
        let chars = source.chars();
        let mut this = Self {
            current: ' ',
            chars: chars,
            line: 0,
            col: 0,
            eof: false,
        };
        // Before that, we need to remove trailing spaces
        this.no_rubbish();
        this
    }

    pub fn parse(&mut self) -> Result<Css, CssError> {
        let mut result = Css { blocks: vec![] };
        while !self.eof {
            let selector = self.parse_multiple_selector()?;
            // This is needed as we encounter spaces and other ignored characters
            self.skip_char('{')?;
            let mut rules = vec![];
            while self.peek() != '}' {
                rules.push(self.parse_rule()?);
            }
            self.skip_char('}')?;
            result.blocks.push(CssBlock { rules, selector });
        }
        Ok(result)
    }
}

#[test]
fn test_integrity() {
    let mut parser = CssParser::new(
        "Div {} .test { color: red; } #ident { border: 2px; padding: 2px 3px 10px 1px; }",
    );
    parser.parse().unwrap();
}

#[test]
fn test_ignored() {
    let mut parser = CssParser::new("   .test");
    parser.no_rubbish();
    // Because we consume next character and store it, the dot is ignored
    assert_eq!(parser.chars.as_str(), "test");
}

// #[test]
fn test_rule_keyword_param() {
    let mut parser = CssParser::new("rsnte");
    let _rule = parser.parse_ident_rule().unwrap_err();
}

#[test]
fn test_rule_parsing() {
    let mut parser = CssParser::new("background-color: red; border: 2px black;");
    let rule = parser.parse_rule().unwrap();
    assert_eq!(rule.name.as_str(), "background-color");
    let param = &rule.params[0];
    assert!(matches!(param, CssRuleParam::Color(255, 0, 0, 255)));

    let rule = parser.parse_rule().unwrap();
    assert_eq!(rule.name.as_str(), "border");
    let param = &rule.params[0];
    assert!(matches!(param, CssRuleParam::Unit(.., CssUnit::Px)));
    let param = &rule.params[1];
    assert!(matches!(param, CssRuleParam::Color(0, 0, 0, 255)));
}

#[test]
// Test selectors for all types of them
fn test_selector_parsing() {
    let mut parser = CssParser::new("   .test");
    let selector = parser.parse_selector_atomic().unwrap();
    assert_eq!(selector.to_string().as_str(), ".test");
    let mut parser = CssParser::new(" \n #test");
    let selector = parser.parse_selector_atomic().unwrap();
    assert_eq!(selector.to_string().as_str(), "#test");

    let mut parser = CssParser::new("\ttest");
    let selector = parser.parse_selector_atomic().unwrap();
    assert_eq!(selector.to_string().as_str(), "test");
}

#[test]
// Test identifier parser
fn test_identifier_parsing() {
    let mut parser = CssParser::new("valid_1dentifier--");
    let ident = parser.parse_ident().unwrap();
    assert_eq!(ident, "valid_1dentifier--");

    // With unknown characters
    let mut parser = CssParser::new("-valid_1dentifier--$$$#$532485092385");
    let ident = parser.parse_ident().unwrap();
    assert_eq!(ident, "-valid_1dentifier--");

    // With ignored characters
    let mut parser = CssParser::new("_valid_1dentifier-- this_is_something-else");
    let ident = parser.parse_ident().unwrap();
    assert_eq!(ident, "_valid_1dentifier--");

    // Invalid
    let mut parser = CssParser::new("1valid_1dentifier-- this_is_something-else");
    let error = parser.parse_ident().unwrap_err();
    assert!(matches!(error.kind, CssErrorKind::InvalidSymbolInIdent));
}
