use crate::cursor::{Cursor, LineCol};
use std::{collections::VecDeque, ops::Range};

/// Trait defining the interface for a text buffer
pub trait TextBuffer {
    /// Insert text at the specified position
    fn insert(&mut self, at: &LineCol, text: &str) -> Result<(), BufferError>;

    /// Delete text in the specified range
    fn delete(&mut self, from: &LineCol, to: &LineCol) -> Result<(), BufferError>;

    /// Replace text in the specified range with new text
    fn replace(&mut self, from: &LineCol, to: &LineCol, text: &str) -> Result<(), BufferError>;

    /// Get the text in the specified range
    fn get_text(&self, from: &LineCol, to: &LineCol) -> Result<String, BufferError>;

    /// Get the length of the entire buffer
    fn len(&self) -> usize;

    /// Check if the buffer is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the number of lines in the buffer
    fn line_count(&self) -> usize;

    /// Get the contents of a specific line
    fn line(&self, line_number: usize) -> Result<&str, BufferError>;

    /// Find the next occurrence of a substring
    fn find(&self, query: &str, at: &LineCol) -> Result<LineCol, BufferError>;

    /// Find the previous occurrence of a substring
    fn rfind(&self, query: &str, at: &LineCol) -> Result<LineCol, BufferError>;

    /// Undo the last operation
    fn undo(&mut self) -> Result<(), BufferError>;

    /// Redo the last undone operation
    fn redo(&mut self) -> Result<(), BufferError>;
}

/// Error type for buffer operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferError {
    InvalidPosition,
    InvalidRange,
    InvalidLineNumber,
    OperationFailed,
    InvalidInput,
    PatternNotFound,
    NowhereToGo,
}

/// A stack implementation using a VecDeque as the underlying storage.
pub struct Stack<T> {
    content: VecDeque<T>,
}

impl<T> Stack<T> {
    /// Truncates the stack to a maximum of 1000 elements.
    /// If the stack has more than 1000 elements, it removes the excess from the back.
    fn truncate(&mut self) {
        let len = self.content.len();
        if len > 1000 {
            self.content.truncate(1000)
        }
    }

    /// Removes and returns the top element from the stack.
    /// Returns None if the stack is empty.
    pub fn pop(&mut self) -> Option<T> {
        self.content.pop_front()
    }

    /// Pushes a new element onto the top of the stack.
    /// After pushing, it truncates the stack to maintain a maximum of 1000 elements.
    pub fn push(&mut self, el: T) {
        self.content.push_front(el);
        self.truncate();
    }

    /// Checks if the stack is empty.
    /// Returns true if the stack contains no elements, false otherwise.
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

/// A buffer implementation for storing text as a vector of lines,
/// with undo and redo functionality.
pub struct VecBuffer {
    /// The current state of the text, stored as a vector of lines.
    lines: Vec<String>,
    /// Stack to store past states for undo operations.
    past: Stack<Vec<String>>,
    /// Stack to store future states for redo operations.
    future: Stack<Vec<String>>,
}

impl TextBuffer for VecBuffer {
    /// Performs a redo operation, moving the current state to the next future state if available.
    /// Returns an error if there are no `future` states to redo to.
    fn redo(&mut self) -> Result<(), BufferError> {
        self.future
            .pop()
            .map(|future_state| {
                let current_state = std::mem::replace(&mut self.lines, future_state);
                self.past.push(current_state);
            })
            .map_or_else(|| Err(BufferError::NowhereToGo), |_| Ok(()))
    }

    /// Performs an undo operation, moving the current state to the previous past state if available.
    /// Returns an error if there are no `past` states to undo to.
    fn undo(&mut self) -> Result<(), BufferError> {
        self.past
            .pop()
            .map(|past_state| {
                let current_state = std::mem::replace(&mut self.lines, past_state);
                self.future.push(current_state);
            })
            .map_or_else(|| Err(BufferError::NowhereToGo), |_| Ok(()))
    }

    /// Searches for a query string in the buffer, starting from a given position.
    ///
    /// # Arguments
    ///
    /// * `query` - The string to search for.
    /// * `at` - The position (line and column) to start the search from.
    ///
    /// # Returns
    ///
    /// * `Ok(LineCol)` - The position (line and column) where the query was found.
    /// * `Err(BufferError::PatternNotFound)` - If the query string is not found in the buffer.
    ///
    /// # Behavior
    ///
    /// The search starts at the given position and continues to the end of the buffer.
    /// It searches the remainder of the starting line, then subsequent lines in their entirety.
    /// The search is case-sensitive and returns the position of the first occurrence found.
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer = // ... initialize buffer ...
    /// let result = buffer.find("example", &LineCol(1, 5));
    /// assert_eq!(result, Ok(LineCol(2, 10))); // Found on line 2, column 10
    /// ```
    fn find(&self, query: &str, at: &LineCol) -> Result<LineCol, BufferError> {
        if query.is_empty() {
            return Err(BufferError::InvalidInput)
        }
        let mut current_line = at.0;
        let mut current_col = at.1;

        while current_line < self.lines.len() {
            if let Some(line) = self.lines.get(current_line) {
                if let Some(pos) = line[current_col..].find(query) {
                    return Ok(LineCol(current_line,  current_col + pos));
                }
            }
            current_line += 1;
            current_col = 0;
        }

        Err(BufferError::PatternNotFound)
    }

    /// Searches backwards for a query string in the buffer, starting from a given position.
    ///
    /// # Arguments
    ///
    /// * `query` - The string to search for.
    /// * `at` - The position (line and column) to start the reverse search from.
    ///
    /// # Returns
    ///
    /// * `Ok(LineCol)` - The position (line and column) where the query was found.
    /// * `Err(BufferError::PatternNotFound)` - If the query string is not found in the buffer.
    ///
    /// # Behavior
    ///
    /// The search starts at the given position and continues backwards to the beginning of the buffer.
    /// It first searches the portion of the starting line from the given position to its start,
    /// then searches previous lines in their entirety from end to start.
    /// The search is case-sensitive and returns the position of the last occurrence found
    /// (i.e., the first occurrence when searching backwards).
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer = // ... initialize buffer ...
    /// let result = buffer.rfind("example", &LineCol(2, 15));
    /// assert_eq!(result, Ok(LineCol(1, 5))); // Found on line 1, column 5
    /// ```
    fn rfind(&self, query: &str, at: &LineCol) -> Result<LineCol, BufferError> {
        if query.is_empty() {
            return Err(BufferError::InvalidInput)
        }
        let mut current_line = at.0;
        let mut current_col = at.1;

        loop {
            if let Some(line) = self.lines.get(current_line) {
                if let Some(pos) = line[..current_col].rfind(query) {
                    return Ok(LineCol(current_line,  pos));
                }
            }
            if current_line == 0 {
                break
            }
            current_line -= 1;
            current_col = self.lines[current_line].len();
        }

        Err(BufferError::PatternNotFound)
    }

    fn len(&self) -> usize {
        // Currently length of the entire file seems unnecessary to implement. If I realize it
        // needs to be implemented it might be as a counter at the level of a struct attribute.
        0
    }

    fn line_count(&self) -> usize {
        self.lines.len()
    }
    fn line(&self, line_number: usize) -> Result<&str, BufferError> {
        if line_number > 0 && line_number <= self.line_count() {
            Ok(self.lines.get(line_number).expect("Checks already passed"))
        } else {
            Err(BufferError::InvalidLineNumber)
        }
    }
    fn get_text(&self, from: &LineCol, to: &LineCol) -> Result<String, BufferError> {
        if from.0 > to.0 || (from.0 == to.0 && from.1 > to.1) {
            return Err(BufferError::InvalidRange);
        }

        if from.0 >= self.lines.len() || to.0 >= self.lines.len() {
            return Err(BufferError::InvalidRange);
        }

        let mut result = String::new();

        if from.0 == to.0 {
            result.push_str(&self.lines[from.0][from.1..to.1]);
        } else {
            result.push_str(&self.lines[from.0][from.1..]);
            result.push('\n');

            for line in &self.lines[from.0 + 1..to.0] {
                result.push_str(line);
                result.push('\n');
            }

            result.push_str(&self.lines[to.0][..to.1]);
        }

        Ok(result)
    }
    /// Replaces a range of text in the buffer with new text.
    ///
    /// # Arguments
    ///
    /// * `from` - The starting position (line and column) of the text to be replaced.
    /// * `to` - The ending position (line and column) of the text to be replaced.
    /// * `text` - The new text to insert in place of the replaced range.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the replacement was successful.
    /// * `Err(BufferError)` if an error occurred during the replacement.
    ///
    /// # Behavior
    ///
    /// This function replaces the text between `from` and `to` positions with the provided `text`.
    /// It handles multi-line replacements, including cases where the new text may have more or fewer
    /// lines than the replaced range. Empty lines resulting from the replacement are removed.
    fn replace(&mut self, from: &LineCol, to: &LineCol, text: &str) -> Result<(), BufferError> {
        let mut lines = text.lines();

        let (left, _) = self.lines[from.0].split_at(from.1);
        let mut new_first_line = String::new();

        new_first_line.push_str(left);
        if let Some(l) = lines.next() {
            new_first_line.push_str(l);
        }

        let _ = std::mem::replace(&mut self.lines[from.0], new_first_line);
        if self.lines[from.0].is_empty() {
            self.lines.remove(from.0);
        }

        for l in from.0 + 1..=to.0 {
            if l == to.0 {
                let (_, right) = self.lines[to.0].split_at(to.1);
                let new_last_line = format!("{}{}", lines.next().unwrap_or(""), right);
                if new_last_line.is_empty() {
                    self.lines.remove(l);
                } else {
                    let _ = std::mem::replace(&mut self.lines[to.0], new_last_line);
                }
            } else if let Some(new_line) = lines.next() {
                let _ = std::mem::replace(&mut self.lines[l], new_line.to_string());
            } else {
                self.lines.remove(l);
            }
        }
        let overflow = to.0 + 1;
        lines.for_each(|line| {
            self.lines[overflow] = line.to_string();
        });
        Ok(())
    }
    fn insert(&mut self, at: &LineCol, text: &str) -> Result<(), BufferError> {
        unimplemented!()
    }
    fn delete(&mut self, from: &LineCol, to: &LineCol) -> Result<(), BufferError> {
        unimplemented!()
    }
    fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn new_test_buffer() -> VecBuffer {
        VecBuffer {
            lines: vec![
                "First line".to_string(),
                "Second line".to_string(),
                "Third line".to_string(),
            ],
            past: Stack {
                content: VecDeque::new(),
            },
            future: Stack {
                content: VecDeque::new(),
            },
        }
    }

    #[test]
    fn test_replace_within_single_line() {
        let mut buf = new_test_buffer();
        buf.replace(&LineCol(0, 6), &LineCol(0, 10), "text")
            .unwrap();
        assert_eq!(buf.lines[0], "First text");
    }

    #[test]
    fn test_replace_across_multiple_lines() {
        let mut buf = new_test_buffer();
        buf.replace(&LineCol(0, 6), &LineCol(2, 5), "new\nreplacement\ntext")
            .unwrap();
        assert_eq!(
            buf.lines,
            vec![
                "First new".to_string(),
                "replacement".to_string(),
                "text line".to_string(),
            ]
        );
    }

    #[test]
    fn test_replacing_with_empty_string() {
        let mut buf = new_test_buffer();
        buf.replace(&LineCol(1, 0), &LineCol(1, 11), "").unwrap();
        assert_eq!(
            buf.lines,
            vec!["First line".to_string(), "Third line".to_string(),]
        );
    }

    #[test]
    fn test_replacing_at_line_end() {
        let mut buf = new_test_buffer();
        buf.replace(&LineCol(1, 7), &LineCol(1, 11), "replacement")
            .unwrap();
        assert_eq!(
            buf.lines,
            vec![
                "First line".to_string(),
                "Second replacement".to_string(),
                "Third line".to_string(),
            ]
        );
    }

    #[test]
    fn test_replacing_with_overflowing_lines() {
        let mut buf = new_test_buffer();
        buf.replace(
            &LineCol(0, 6),
            &LineCol(2, 5),
            "new\nreplacement\ntext\nthisalso",
        )
        .unwrap();
        assert_eq!(
            buf.lines,
            vec![
                "First new".to_string(),
                "replacement".to_string(),
                "text".to_string(),
                "thisalso line".to_string()
            ]
        );
    }

    #[test]
    fn test_replacing_at_buffer_end() {
        let mut buf = new_test_buffer();
        buf.replace(&LineCol(2, 5), &LineCol(2, 10), "replacement")
            .unwrap();
        assert_eq!(
            buf.lines,
            vec![
                "First line".to_string(),
                "Second line".to_string(),
                "Third replacement".to_string(),
            ]
        );
    } 

    /// "First line with some text"
    /// "Second line also has text"
    /// "Third line is here too"
    fn new_test_buffer_find() -> VecBuffer {
        VecBuffer {
            lines: vec![
                "First line with some text".to_string(),
                "Second line also has text".to_string(),
                "Third line is here too".to_string(),
            ],
            past: Stack { content: VecDeque::new() },
            future: Stack { content: VecDeque::new() },
        }
    }

    #[test]
    fn test_rfind_basic() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.rfind("line", &LineCol(2, 0)), Ok(LineCol(1, 7)));
    }

    #[test]
    fn test_rfind_not_including_start() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.rfind("line", &LineCol(1, 7)), Ok(LineCol(0, 6)));
    }

    #[test]
    fn test_rfind_across_lines() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.rfind("First", &LineCol(2, 0)), Ok(LineCol(0, 0)));
    }

    #[test]
    fn test_rfind_at_start_of_buffer() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.rfind("First", &LineCol(0, 4)), Err(BufferError::PatternNotFound));
    }

    #[test]
    fn test_rfind_pattern_not_found() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.rfind("nonexistent", &LineCol(2, 0)), Err(BufferError::PatternNotFound));
    }

    #[test]
    fn test_rfind_empty_query() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.rfind("", &LineCol(1, 5)), Err(BufferError::InvalidInput));
    }

    #[test]
    fn test_rfind_at_end_of_buffer() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.rfind("too", &LineCol(2, 22)), Ok(LineCol(2, 19)));
    }
    #[test]
    fn test_find_basic() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.find("line", &LineCol(0, 0)), Ok(LineCol(0, 6)));
    }

    #[test]
    fn test_find_from_middle() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.find("text", &LineCol(0, 10)), Ok(LineCol(0, 21)));
    }

    #[test]
    fn test_find_across_lines() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.find("Second", &LineCol(0, 22)), Ok(LineCol(1, 0)));
    }

    #[test]
    fn test_find_at_start_of_line() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.find("Third", &LineCol(2, 0)), Ok(LineCol(2, 0)));
    }

    #[test]
    fn test_find_at_end_of_line() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.find("text", &LineCol(1, 0)), Ok(LineCol(1, 21)));
    }

    #[test]
    fn test_find_pattern_not_found() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.find("nonexistent", &LineCol(0, 0)), Err(BufferError::PatternNotFound));
    }

    #[test]
    fn test_find_empty_query() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.find("", &LineCol(1, 5)), Err(BufferError::InvalidInput));
    }

    #[test]
    fn test_find_at_end_of_buffer() {
        let buf = new_test_buffer_find();
        let last_line = buf.lines.len() - 1;
        let last_col = buf.lines[last_line].len();
        assert_eq!(buf.find("too", &LineCol(last_line, last_col)), Err(BufferError::PatternNotFound));
    }

    #[test]
    fn test_find_exact_position() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.find("Second", &LineCol(1, 0)), Ok(LineCol(1, 0)));
    }

    #[test]
    fn test_find_multiple_occurrences() {
        let buf = new_test_buffer_find();
        assert_eq!(buf.find("line", &LineCol(0, 7)), Ok(LineCol(1, 7)));
    }

    #[test]
    fn test_find_from_empty_line() {
        let mut buf = new_test_buffer_find();
        buf.lines.insert(1, String::new());
        assert_eq!(buf.find("Third", &LineCol(1, 0)), Ok(LineCol(3, 0)));
    }
}
