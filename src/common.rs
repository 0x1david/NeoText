use std::{
    collections::VecDeque,
    sync::{Mutex, OnceLock},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}

static DEBUG_MESSAGES: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

impl std::ops::Sub for LineCol {
    type Output = LineCol;
    fn sub(self, rhs: Self) -> Self::Output {
        LineCol {
            line: self.line.saturating_sub(rhs.line),
            col: self.col.saturating_sub(rhs.col),
        }
    }
}

impl std::ops::Add for LineCol {
    type Output = LineCol;
    fn add(self, rhs: Self) -> Self::Output {
        LineCol {
            line: self.line + rhs.line,
            col: self.col + rhs.col,
        }
    }
}

impl std::fmt::Display for LineCol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

impl PartialOrd for LineCol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.line.cmp(&other.line) {
            std::cmp::Ordering::Equal => self.col.cmp(&other.col).into(),
            otherwise => Some(otherwise),
        }
    }
}

/// Retrieves or initializes the global debug message queue.
///
/// Returns a static reference to a `Mutex<VecDeque<String>>` which stores
/// debug messages used by the `bar_dbg!` macro. Initializes the queue
/// on first call.
pub fn get_debug_messages() -> &'static Mutex<VecDeque<String>> {
    DEBUG_MESSAGES.get_or_init(|| Mutex::new(VecDeque::new()))
}

/// A versatile debugging macro that logs expressions and their values to an info bar,
/// similar to the standard `dbg!` macro, with additional flexibility.
///
/// This macro captures the file name and line number where it's invoked,
/// evaluates the given expression(s), formats a debug message, and adds it
/// to a global debug message queue. It can either return the value of the expression
/// or not, depending on whether the element list is ended with semicolon or not.
///
/// # Features
/// - Logs the file name and line number of the macro invocation
/// - Logs the expression as a string and its evaluated value
/// - Can handle multiple expressions
/// - Optionally returns the value of the expression, allowing inline use
/// - Maintains a queue of the last 10 debug messages
/// - Behavior changes based on the presence or absence of a trailing semicolon
///
/// # Usage
/// ```
/// let x = notif_bar!(5 + 3);  // Logs and returns 8
/// notif_bar!(5 + 3;)  // Logs without returning
/// let (a, b) = notif_bar!(1, "two");  // Logs and returns (1, "two")
/// notif_bar!(1, "two";)  // Logs multiple values without returning
/// ```
///
/// # Notes
/// - The expression(s) must implement the `Debug` trait for proper formatting
/// - If the debug message queue exceeds 10 messages, the oldest message is removed
/// - The presence or absence of a trailing semicolon determines whether the macro returns a value
///
/// # Panics
/// This macro will not panic, but it may fail silently if it cannot acquire
/// the lock on the debug message queue.the debug message queue.
#[macro_export]
macro_rules! notif_bar {
    // Version that returns the value (no semicolon)
    ($val:expr) => {{
        let file = file!();
        let line = line!();
        let val = $val;
        let message = format!("[{}:{}] {} = {:?}", file, line, stringify!($val), &val);
        if let Ok(mut messages) = $crate::get_debug_messages().lock() {
            messages.push_back(message);
            if messages.len() > 10 {
                messages.pop_front();
            }
        }
        val
    }};

    // Version that doesn't return the value (with semicolon)
    ($val:expr;) => {{
        let file = file!();
        let line = line!();
        let message = format!("[{}:{}] {} = {:?}", file, line, stringify!($val), &$val);
        if let Ok(mut messages) = get_debug_messages().lock() {
            messages.push_back(message);
            if messages.len() > 10 {
                messages.pop_front();
            }
        }
    }};

    // Multiple arguments version (no semicolon)
    ($($val:expr),+ $(,)?) => {
        ($(notif_bar!($val)),+,)
    };

    // Multiple arguments version (with semicolon)
    ($($val:expr),+ $(,)?;) => {
        $(notif_bar!($val;))+
    };
}

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
// So essentially what I need is for each type to implement a function that takes a &[impl AsRef<str>]
// and returns a LineCol

use std::borrow::Cow;

pub trait Pattern {
    /// The caller has two main responsibilities:
    ///     1. Preprocessing the haystack in such a way that only the part to be searched is
    ///        provided
    ///     2. Adding the column number that we were starting from if the match is found on the
    ///        first line of the search (if returned linecol.line equals the cursor position)
    ///
    /// Thus find and rfind will require to be split at the cursor
    fn find_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol>;
    fn rfind_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol>;
}

impl Pattern for &str {
    fn find_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        haystack
            .iter()
            .enumerate()
            .find_map(|(line_num, line_content)| {
                line_content.as_ref().find(self).map(|col| LineCol {
                    line: line_num,
                    col,
                })
            })
    }
    fn rfind_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        haystack
            .iter()
            .enumerate()
            .rev()
            .find_map(|(line_num, line_content)| {
                line_content.as_ref().rfind(self).map(|col| LineCol {
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
    fn rfind_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        self.as_str().rfind_pattern(haystack)
    }
}

impl Pattern for Cow<'_, str> {
    fn find_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        self.as_ref().find_pattern(haystack)
    }
    fn rfind_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        self.as_ref().rfind_pattern(haystack)
    }
}

impl Pattern for char {
    fn find_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        haystack
            .iter()
            .enumerate()
            .find_map(|(line_num, line_content)| {
                line_content.as_ref().find(*self).map(|col| LineCol {
                    line: line_num,
                    col,
                })
            })
    }
    fn rfind_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        haystack
            .iter()
            .enumerate()
            .rev()
            .find_map(|(line_num, line_content)| {
                line_content.as_ref().rfind(*self).map(|col| LineCol {
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
    fn rfind_pattern(&self, haystack: &[impl AsRef<str>]) -> Option<LineCol> {
        haystack
            .iter()
            .enumerate()
            .rev()
            .find_map(|(line_num, line_content)| {
                line_content
                    .as_ref()
                    .chars()
                    .rev()
                    .position(self)
                    .map(|rcol| LineCol {
                        line: line_num,
                        col: line_content.as_ref().len() - rcol,
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
        assert_eq!(
            pattern.find_pattern(&buffer),
            Some(LineCol { line: 0, col: 7 })
        );
    }

    #[test]
    fn test_string_pattern() {
        let buffer = create_test_buffer();
        let pattern = "test".to_string();
        assert_eq!(
            pattern.find_pattern(&buffer),
            Some(LineCol { line: 1, col: 10 })
        );
    }

    #[test]
    fn test_cow_str_pattern() {
        let buffer = create_test_buffer();
        let pattern: Cow<str> = Cow::Borrowed("This");
        assert_eq!(
            pattern.find_pattern(&buffer),
            Some(LineCol { line: 1, col: 0 })
        );
    }

    #[test]
    fn test_char_pattern() {
        let buffer = create_test_buffer();
        let pattern = '5';
        assert_eq!(
            pattern.find_pattern(&buffer),
            Some(LineCol { line: 2, col: 4 })
        );
    }

    #[test]
    fn test_char_predicate_pattern() {
        let buffer = create_test_buffer();
        let pattern = |c: char| c.is_ascii_digit();
        assert_eq!(
            pattern.find_pattern(&buffer),
            Some(LineCol { line: 2, col: 0 })
        );
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
        assert_eq!(
            pattern.find_pattern(&buffer),
            Some(LineCol { line: 0, col: 0 })
        );
    }

    #[test]
    fn test_pattern_at_end() {
        let buffer = vec!["End here".to_string()];
        let pattern = "here";
        assert_eq!(
            pattern.find_pattern(&buffer),
            Some(LineCol { line: 0, col: 4 })
        );
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
        assert_eq!(
            pattern.find_pattern(&buffer),
            Some(LineCol { line: 0, col: 2 })
        );
    }

    #[test]
    fn test_char_predicate_uppercase() {
        let buffer = vec!["lowercase".to_string(), "UPPERCASE".to_string()];
        let pattern = |c: char| c.is_ascii_uppercase();
        assert_eq!(
            pattern.find_pattern(&buffer),
            Some(LineCol { line: 1, col: 0 })
        );
    }

    #[test]
    fn test_pattern_with_special_chars() {
        let buffer = vec!["Special: !@#$%^&*()".to_string()];
        let pattern = "$%^&";
        assert_eq!(
            pattern.find_pattern(&buffer),
            Some(LineCol { line: 0, col: 12 })
        );
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
        assert_eq!(
            pattern.find_pattern(&buffer),
            Some(LineCol { line: 0, col: 0 })
        );
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

        let final_result = result2.map(|lc| LineCol {
            line: lc.line + result1.unwrap().line + 1,
            col: lc.col,
        });
        assert_eq!(final_result, Some(LineCol { line: 1, col: 0 }));
    }
}
