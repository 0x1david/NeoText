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

use crate::cursor::LineCol;

pub trait Pattern {
    /// The caller has two main responsibilities:
    ///     1. Preprocessing the haystack in such a way that only the part to be searched is
    ///        provided
    ///     2. Adding the column number that we were starting from if the match is found on the
    ///        first line of the search (if returned linecol.line equals the cursor position)
    ///
    /// Thus find and rfind will require to be split at the cursor
    fn find(self, haystack: &[impl AsRef<str>]) -> Option<LineCol>;
}

impl<T> Pattern for T
where
    T: AsRef<str>,
{
    fn find(self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        haystack
            .iter()
            .enumerate()
            .find_map(|(line_num, line_content)| {
                line_content
                    .as_ref()
                    .find(self.as_ref())
                    .map(|col| LineCol {
                        line: line_num,
                        col,
                    })
            })
    }
}


pub struct FnSearcher<F>( pub F );

impl<F> Pattern for FnSearcher<F>
where
    F: Fn(&str) -> Option<usize>,
{
    fn find(self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        haystack
            .iter()
            .enumerate()
            .find_map(|(line_num, line_content)| {
                self.0(line_content.as_ref()).map(|col| LineCol {
                    line: line_num,
                    col
                })
            })
    }
}
