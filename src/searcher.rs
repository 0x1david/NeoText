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
use crate::{notif_bar, get_debug_messages};

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

// impl<F> Pattern for F
// where
//     F: Fn(&str) -> Option<usize>,
// {
//     fn find_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
//         haystack
//             .iter()
//             .enumerate()
//             .find_map(|(line_num, line_content)| {
//                 self(line_content.as_ref()).map(|col| LineCol {
//                     line: line_num,
//                     col,
//                 })
//             })
//     }
// }

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

impl<F> Pattern for F
where
    F: Fn(char) -> bool,
{
    fn find_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        haystack
            .iter()
            .enumerate()
            .find_map(|(line_num, line_content)| {
                line_content
                    .as_ref()
                    .chars()
                    .position(self)
                    .map(|col| LineCol {
                        line: line_num,
                        col,
                    })
            })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_buffer() -> Vec<String> {
        vec![
            "Hello, world!".to_string(),
            "This is a test.".to_string(),
            "12345".to_string(),
            "   Spaces   ".to_string(),
        ]
    }

    #[test]
    fn test_str_pattern() {
        let buffer = create_test_buffer();
        let pattern = "world";
        assert_eq!(pattern.find_pattern(&buffer), Some(LineCol { line: 0, col: 7 }));
    }

    #[test]
    fn test_string_pattern() {
        let buffer = create_test_buffer();
        let pattern = "test".to_string();
        assert_eq!(pattern.find_pattern(&buffer), Some(LineCol { line: 1, col: 10 }));
    }

    #[test]
    fn test_cow_str_pattern() {
        let buffer = create_test_buffer();
        let pattern: Cow<str> = Cow::Borrowed("This");
        assert_eq!(pattern.find_pattern(&buffer), Some(LineCol { line: 1, col: 0 }));
    }

    #[test]
    fn test_char_pattern() {
        let buffer = create_test_buffer();
        let pattern = '5';
        assert_eq!(pattern.find_pattern(&buffer), Some(LineCol { line: 2, col: 4 }));
    }


    #[test]
    fn test_char_predicate_pattern() {
        let buffer = create_test_buffer();
        let pattern = |c: char| c.is_ascii_digit();
        assert_eq!(pattern.find_pattern(&buffer), Some(LineCol { line: 2, col: 0 }));
    }

    #[test]
    fn test_pattern_not_found() {
        let buffer = create_test_buffer();
        let pattern = "nonexistent";
        assert_eq!(pattern.find_pattern(&buffer), None);
    }
    #[test]
    fn test_empty_buffer() {
        let buffer: Vec<String> = Vec::new();
        let pattern = "test";
        assert_eq!(pattern.find_pattern(&buffer), None);
    }

    #[test]
    fn test_pattern_at_start() {
        let buffer = vec!["Start here".to_string()];
        let pattern = "Start";
        assert_eq!(pattern.find_pattern(&buffer), Some(LineCol { line: 0, col: 0 }));
    }

    #[test]
    fn test_pattern_at_end() {
        let buffer = vec!["End here".to_string()];
        let pattern = "here";
        assert_eq!(pattern.find_pattern(&buffer), Some(LineCol { line: 0, col: 4 }));
    }

    #[test]
    fn test_pattern_spanning_lines() {
        let buffer = vec!["First ".to_string(), "line".to_string()];
        let pattern = "First line";
        assert_eq!(pattern.find_pattern(&buffer), None);
    }

    #[test]
    fn test_char_pattern_whitespace() {
        let buffer = vec!["No space".to_string(), " Leading space".to_string()];
        let pattern = ' ';
        assert_eq!(pattern.find_pattern(&buffer), Some(LineCol { line: 0, col: 2 }));
    }

    #[test]
    fn test_char_predicate_uppercase() {
        let buffer = vec!["lowercase".to_string(), "UPPERCASE".to_string()];
        let pattern = |c: char| c.is_ascii_uppercase();
        assert_eq!(pattern.find_pattern(&buffer), Some(LineCol { line: 1, col: 0 }));
    }

    #[test]
    fn test_pattern_with_special_chars() {
        let buffer = vec!["Special: !@#$%^&*()".to_string()];
        let pattern = "$%^&";
        assert_eq!(pattern.find_pattern(&buffer), Some(LineCol { line: 0, col: 12 }));
    }

    #[test]
    fn test_pattern_case_sensitivity() {
        let buffer = vec!["Case Sensitive".to_string()];
        let pattern = "case";
        assert_eq!(pattern.find_pattern(&buffer), None);
    }

    #[test]
    fn test_pattern_overlapping() {
        let buffer = vec!["aaa".to_string()];
        let pattern = "aa";
        assert_eq!(pattern.find_pattern(&buffer), Some(LineCol { line: 0, col: 0 }));
    }
    #[test]
    fn test_sequential_char_predicates() {
        let buffer = vec![
            "First line with some numbers 123".to_string(),
            "Second line without numbers".to_string(),
            "Third line with 456 and more text".to_string(),
            "Fourth line ends with numbers 789".to_string(),
        ];

        let predicate1 = |c: char| c.is_ascii_digit();
        
        let predicate2 = |c: char| c.is_ascii_uppercase();

        let result1 = predicate1.find_pattern(&buffer);
        assert_eq!(result1, Some(LineCol { line: 0, col: 29 }));

        let result2 = predicate2.find_pattern(&buffer[result1.unwrap().line + 1..]);
        assert_eq!(result2, Some(LineCol { line: 0, col: 0 })); // 'S' in "Second"

        let final_result = result2.map(|lc| LineCol { line: lc.line + result1.unwrap().line + 1, col: lc.col });
        assert_eq!(final_result, Some(LineCol { line: 1, col: 0 }));
    }
}
