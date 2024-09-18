use crate::{
    theme::{self, Theme},
    Result,
};
use crossterm::style::Color;
use rangemap::RangeMap;
use tree_sitter::{Parser, Query, QueryCursor};
use tree_sitter_rust::{language, HIGHLIGHTS_QUERY};

pub struct Highlighter {
    parser: Parser,
    query: Query,
    pub theme: Box<dyn Theme>,
    tree: Option<tree_sitter::Tree>,
}
impl Highlighter {
    pub fn new(text: impl AsRef<[u8]>) -> Result<Self> {
        let lang = &language();
        let mut parser = Parser::new();
        parser
            .set_language(lang)
            .expect("Couldn't create parser for the given language");
        let query = Query::new(lang, HIGHLIGHTS_QUERY)
            .expect("Couldn't create query for the language parser");

        Ok(Self {
            query,
            theme: Box::new(theme::Monokai {}),
            tree: parser.parse(text, None),
            parser,
        })
    }
    pub fn parse(&mut self, t: &[u8]) {
        let tree = self.parser.parse(t, self.tree.as_ref());
        self.tree = tree;
    }
    pub fn highlight(&mut self, text: &[u8]) -> Result<RangeMap<usize, Style>> {
        let mut cursor = QueryCursor::new();
        let tree = self.tree.as_ref().expect("Parsing preceds highlighting");

        let matches = cursor.matches(&self.query, tree.root_node(), text);
        let mut style_map = RangeMap::new();

        for match_ in matches {
            for capture in match_.captures {
                let node = capture.node;
                let range = node.byte_range();
                let scope = self.query.capture_names()[capture.index as usize];
                let style = self.theme.from_str(scope);

                style_map.insert(range, Style::new(style, Color::Reset, false, false));
            }
        }
        Ok(style_map)
    }
}

/// Style with span location
pub struct StyleSpan {
    span: (usize, usize),
    style: Style,
}
impl StyleSpan {
    pub fn contains(&self, pos: usize) -> bool {
        pos >= self.span.0 && pos < self.span.1
    }

    pub fn new(from: usize, to: usize, fg: Color, bg: Color, bold: bool, italic: bool) -> Self {
        Self {
            span: (from, to),
            style: Style::new(fg, bg, bold, italic),
        }
    }
}

/// Contains style information
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
            bold: false,
            italic: false,
        }
    }
}

impl Style {
    pub fn new(fg: Color, bg: Color, bold: bool, italic: bool) -> Self {
        Self {
            fg,
            bg,
            bold,
            italic,
        }
    }
}
