// I have functions find and rfind
// I wan't them to be able to take either:
// STRING_TYPES -> Which are the pattern that we are looking for in the return type.
// CHAR -> Return the first occurence of the character in the searched file.
// CLOSURE -> e.g FIND NEXT SPACE AND AFTER THAT FIND FIRST NONSPACE AND RETURN THE LOCATION OF IT
// || {
//      let pos = vec.find_space();
//      let nonspace = vec.find_non_space();
//      return LineCol nonspace;
// }
//
// So essentially what I need is for each type to implement a function that takes a Vec<AsRef<str>>
// and returns a LineCol

use std::borrow::Cow;

use crate::cursor::LineCol;

pub trait Pattern {
    /// The caller has two main responsibilities:
    ///     1. Preprocessing the haystack in such a way that only the part to be searched is
    ///        provided
    ///     2. Adding the column number that we were starting from if the match is found on the
    ///        first line of the search (if returned linecol.line equals the cursor position)
    ///
    /// Thus find and rfind will require to be split at the cursor
    fn find_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol>;
}

impl Pattern for &str
{
    fn find_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        haystack
            .iter()
            .enumerate()
            .find_map(|(line_num, line_content)| {
                line_content
                    .as_ref()
                    .find(self)
                    .map(|col| LineCol {
                        line: line_num,
                        col,
                    })
            })
    }
}

impl<F> Pattern for F
where
    F: Fn(&str) -> Option<usize>,
{
    fn find_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        haystack
            .iter()
            .enumerate()
            .find_map(|(line_num, line_content)| {
                self(line_content.as_ref()).map(|col| LineCol {
                    line: line_num,
                    col,
                })
            })
    }
}

impl Pattern for String {
    fn find_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        self.as_str().find_pattern(haystack)
    }
}

impl Pattern for Cow<'_, str> {
    fn find_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        self.as_ref().find_pattern(haystack)
    }
}

impl Pattern for char {
    fn find_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        haystack
            .iter()
            .enumerate()
            .find_map(|(line_num, line_content)| {
                line_content
                    .as_ref()
                    .find(*self)
                    .map(|col| LineCol {
                        line: line_num,
                        col,
                    })
            })
    }
}
