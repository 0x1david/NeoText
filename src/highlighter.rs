// use crate::{cursor::LineCol, Result};
// use crossterm::style::Color;
// use tree_sitter::{Language, Parser, Query, QueryCursor, Tree, TreeCursor};
// use tree_sitter_rust::{language, HIGHLIGHTS_QUERY};

// pub struct Highlighter {
//     parser: Parser,
//     query: Query,
// }
// impl Highlighter {
//     fn new() -> Result<Self> {
//         let lang = &language();
//         let mut parser = Parser::new();
//         parser
//             .set_language(lang)
//             .expect("Couldn't create parser for the given language");
//         let query = Query::new(lang, HIGHLIGHTS_QUERY)
//             .expect("Couldn't create query for the language parser");
//         Ok(Self { parser, query })
//     }
//     pub fn highlight(&mut self, text: &[String]) -> Result<&[HLSpan]> {
//         let text = text.join("\n");
//         let tree = self
//             .parser
//             .parse(text, None)
//             .expect("Parsing should work just fine");

//         let mut highlights = vec![];
//         let mut cursor = QueryCursor::new();
//         let matches = cursor.matches(&self.query, tree.root_node(), text.as_bytes());

//         for m in matches {
//             for capture in m.captures {
//                 let node = capture.node;
//                 let start = node.start_byte();
//                 let end = node.end_byte();
//                 let scope = self.query.capture_names()[capture.index as usize];
//                 let style = self.theme.get_style(scope);

//                 if let Some(style) = style {
//                     colors.push(HLSpan { span.0: start, end, style });
//                 }
//             }
//         }
//         Ok(())
//     }
// }

// pub struct HLSpan {
//     span: (LineCol, LineCol),
//     style: Style,
// }
// impl HLSpan {
//     pub fn contains(&self, pos: LineCol) -> bool {
//         pos >= self.span.0 && pos < self.span.1
//     }
// }

// pub struct Style {
//     pub fg: Option<Color>,
//     pub bg: Option<Color>,
//     pub bold: bool,
//     pub italic: bool,
// }
