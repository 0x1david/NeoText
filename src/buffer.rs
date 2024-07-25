use crate::cursor::{Cursor, LineCol};
use std::{collections::VecDeque, ops::Range};

/// Trait defining the interface for a text buffer
pub trait TextBuffer {
    fn insert_newline(&mut self, at: LineCol) -> LineCol;
    /// Insert a single symbol at specified position
    fn insert(&mut self, at: LineCol, insertable: char) -> Result<LineCol, BufferError>;

    /// Insert text at the specified position
    fn insert_text(
        &mut self,
        at: LineCol,
        text: String,
        newline: bool,
    ) -> Result<LineCol, BufferError>;

    /// Delete text in the specified range
    fn delete_selection(&mut self, from: LineCol, to: LineCol) -> Result<LineCol, BufferError>;

    /// Delete the symbol at the specified position
    fn delete(&mut self, at: LineCol) -> Result<LineCol, BufferError>;

    /// Replace text in the specified range with new text
    fn replace(&mut self, from: LineCol, to: LineCol, text: &str) -> Result<(), BufferError>;

    /// Get the text in the specified range
    fn get_text(&self, from: LineCol, to: LineCol) -> Result<String, BufferError>;

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
    fn find(&self, query: &str, at: LineCol) -> Result<LineCol, BufferError>;

    /// Find the previous occurrence of a substring
    fn rfind(&self, query: &str, at: LineCol) -> Result<LineCol, BufferError>;

    /// Undo the last operation
    fn undo(&mut self, at: LineCol) -> Result<LineCol, BufferError>;

    /// Redo the last undone operation
    fn redo(&mut self, at: LineCol) -> Result<LineCol, BufferError>;

    /// Redo the last undone operation
    fn get_entire_text(&self) -> &Vec<String>;

    /// Redo the last undone operation
    fn max_line(&self) -> usize;
    /// Redo the last undone operation
    fn max_col(&self, at: LineCol) -> usize;
}

/// Error type for buffer operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferError {
    InvalidPosition,
    InvalidRange,
    InvalidLineNumber,
    InvalidInput,
    PatternNotFound,
    NowhereToGo,
    ImATeacup,
}

/// A stack implementation using a VecDeque as the underlying storage.
#[derive(Debug, Default)]
pub struct Stack {
    content: VecDeque<StateCapsule>,
}

impl Stack {
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
    pub fn pop(&mut self) -> Option<StateCapsule> {
        self.content.pop_front()
    }

    /// Pushes a new element onto the top of the stack.
    /// After pushing, it truncates the stack to maintain a maximum of 1000 elements.
    pub fn push(&mut self, el: StateCapsule) {
        self.content.push_front(el);
        self.truncate();
    }

    /// Checks if the stack is empty.
    /// Returns true if the stack contains no elements, false otherwise.
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

/// Stores content and cursor location at a point in time of the editing process.
#[derive(Debug, Default)]
pub struct StateCapsule {
    content: Vec<String>,
    loc: LineCol,
}

/// A buffer implementation for storing text as a vector of lines,
/// with undo and redo functionality. Highly inefficient, both tim complexity wise and implementation wise. Simply a placeholder for testing.
#[derive(Debug)]
pub struct VecBuffer {
    /// The current state of the text, stored as a vector of lines.
    lines: Vec<String>,
    /// Stack to store past states for undo operations.
    past: Stack,
    /// Stack to store future states for redo operations.
    future: Stack,
}

impl Default for VecBuffer {
    fn default() -> Self {
        Self {
            lines: vec!["".to_string()],
            past: Stack::default(),
            future: Stack::default(),
        }
    }
}

impl TextBuffer for VecBuffer {
    fn max_col(&self, at: LineCol) -> usize {
        self.lines[at.line].len()
    }
    fn max_line(&self) -> usize {
        self.lines.len()
    }
    fn insert_newline(&mut self, mut at: LineCol) -> LineCol {
        self.lines.insert(at.line + 1, Default::default());
        at.line += 1;
        at.col = 0;
        at
    }
    fn insert(&mut self, mut at: LineCol, ch: char) -> Result<LineCol, BufferError> {
        if at.line > self.lines.len() || at.col > self.lines[at.line].len() {
            return Err(BufferError::InvalidPosition);
        }
        self.lines[at.line].insert(at.col, ch);
        at.col += 1;
        Ok(at)
    }
    /// Performs a redo operation, moving the current state to the next future state if available.
    /// Returns an error if there are no `future` states to redo to.
    fn redo(&mut self, at: LineCol) -> Result<LineCol, BufferError> {
        self.future
            .pop()
            .map(|future_state| {
                let current_state = std::mem::replace(&mut self.lines, future_state.content);
                self.past.push(StateCapsule {
                    content: current_state,
                    loc: at,
                });
                future_state.loc
            })
            .map_or_else(|| Err(BufferError::NowhereToGo), Ok)
    }

    /// Performs an undo operation, moving the current state to the previous past state if available.
    /// Returns an error if there are no `past` states to undo to.
    fn undo(&mut self, at: LineCol) -> Result<LineCol, BufferError> {
        self.past
            .pop()
            .map(|past_state| {
                let current_state = std::mem::replace(&mut self.lines, past_state.content);
                self.future.push(StateCapsule {
                    content: current_state,
                    loc: at,
                });
                past_state.loc
            })
            .map_or_else(|| Err(BufferError::NowhereToGo), Ok)
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
    /// let result = buffer.find("example", LineCol{line: 1, col: 5});
    /// assert_eq!(result, Ok(LineCol{line: 2, col: 10})); // Found on line 2, column 10
    /// ```
    fn find(&self, query: &str, at: LineCol) -> Result<LineCol, BufferError> {
        if query.is_empty() {
            return Err(BufferError::InvalidInput);
        }
        let mut current_line = at.line;
        let mut current_col = at.col;

        while current_line < self.lines.len() {
            if let Some(line) = self.lines.get(current_line) {
                if let Some(pos) = line[current_col..].find(query) {
                    return Ok(LineCol {
                        line: current_line,
                        col: current_col + pos,
                    });
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
    /// let result = buffer.rfind("example", LineCol{line: 2, col: 15});
    /// assert_eq!(result, Ok(LineCol{line: 1, col: 5})); // Found on line 1, column 5
    /// ```
    fn rfind(&self, query: &str, at: LineCol) -> Result<LineCol, BufferError> {
        if query.is_empty() {
            return Err(BufferError::InvalidInput);
        }
        let mut current_line = at.line;
        let mut current_col = at.col;

        loop {
            if let Some(line) = self.lines.get(current_line) {
                if let Some(pos) = line[..current_col].rfind(query) {
                    return Ok(LineCol {
                        line: current_line,
                        col: pos,
                    });
                }
            }
            if current_line == 0 {
                break;
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
    /// Retrieves text from the buffer within the specified range.
    ///
    /// # Arguments
    ///
    /// * `from` - The starting position (line and column) of the text to retrieve.
    /// * `to` - The ending position (line and column) of the text to retrieve.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the requested text if the operation was successful.
    /// * `Err(BufferError::InvalidRange)` if the specified range is invalid.
    ///
    /// # Behavior
    ///
    /// This function extracts text from the buffer between the `from` and `to` positions, inclusive.
    /// It handles multi-line ranges and includes newline characters between lines when appropriate.
    ///
    /// # Errors
    ///
    /// Returns `BufferError::InvalidRange` in the following cases:
    /// - If the start position exceeds the end position.
    /// - If either the start or end position is beyond the buffer's contents.
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer = // ... initialize buffer ...
    /// let from = LineCol{line: 1, col: 5};
    /// let to = LineCol{line: 2, col: 10};
    /// match buffer.get_text(&from, &to) {
    ///     Ok(text) => println!("Retrieved text: {}", text),
    ///     Err(BufferError::InvalidRange) => println!("Invalid range specified"),
    ///     Err(_) => println!("An error occurred"),
    /// }
    /// ```
    fn get_text(&self, from: LineCol, to: LineCol) -> Result<String, BufferError> {
        let start_exceeds_end = from.line > to.line || (from.line == to.line && from.col > to.col);
        let exceeds_file_len = from.line >= self.lines.len()
            || to.line >= self.lines.len()
            || from.col > self.lines[from.line].len()
            || to.col > self.lines[to.line].len();
        if start_exceeds_end || exceeds_file_len {
            return Err(BufferError::InvalidRange);
        }

        let mut result = String::new();

        if from.line == to.line {
            result.push_str(&self.lines[from.line][from.col..to.col]);
        } else {
            result.push_str(&self.lines[from.line][from.col..]);
            result.push('\n');

            for line in &self.lines[from.line + 1..to.line] {
                result.push_str(line);
                result.push('\n');
            }

            result.push_str(&self.lines[to.line][..to.col]);
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
    /// * `Err(BufferError::InvalidInput)` if the input text is empty.
    ///
    /// # Behavior
    ///
    /// This function replaces the text between `from` and `to` positions with the provided `text`.
    /// It handles multi-line replacements, preserving the start of the first line before `from`
    /// and the end of the last line after `to`.
    ///
    /// # Note
    ///
    /// The caller must ensure that `text` is not empty. If empty text replacement is needed,
    /// use the `delete` method instead.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buffer = // ... initialize buffer ...
    /// let from = LineCol{line: 1, col: 5};
    /// let to = LineCol{line: 2, col: 10};
    /// let new_text = "replacement text";
    /// buffer.replace(&from, &to, new_text).expect("Replace operation failed");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `BufferError::InvalidInput` if `text` is empty.
    fn replace(&mut self, from: LineCol, to: LineCol, text: &str) -> Result<(), BufferError> {
        if text.is_empty() {
            return Err(BufferError::InvalidInput);
        }
        let mut new_lines = Vec::new();
        let mut lines = text.lines();

        if let Some(first_line) = lines.next() {
            let start = &self.lines[from.line][..from.col];
            new_lines.push(format!("{}{}", start, first_line));
        } else {
            new_lines.push(self.lines[from.line][..from.col].to_string());
        }

        new_lines.extend(lines.map(String::from));

        let last = new_lines.last_mut().expect("We know there is a last line");
        last.push_str(&self.lines[to.line][to.col..]);

        self.lines.splice(from.line..=to.line, new_lines);

        Ok(())
    }
    /// Inserts text into the buffer at the specified position.
    ///
    /// # Arguments
    ///
    /// * `at` - A `LineCol` struct specifying the line and column where the insertion should begin.
    /// * `text` - The string to be inserted.
    /// * `newline` - A boolean flag indicating whether the text should be inserted as new line(s).
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the insertion was successful.
    /// * `Err(BufferError::InvalidPosition)` if the specified position is out of bounds.
    ///
    /// # Behavior
    ///
    /// If `newline` is true:
    ///   - The entire `text` is inserted as new line(s) starting at the specified line.
    ///   - Existing lines at and after the insertion point are shifted down.
    ///
    /// If `newline` is false:
    ///   - The text is inserted at the specified position within the existing line.
    ///   - If `text` contains multiple lines, it splits the current line and inserts the new lines.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buffer = // ... initialize buffer ...
    /// let result = buffer.insert(LineCol { line: 1, col: 5 }, "Hello, world!".to_string(), false);
    /// assert!(result.is_ok());
    /// ```
    ///
    /// # Note
    ///
    /// This function may change the structure of the buffer by adding or modifying lines.
    /// It's the caller's responsibility to ensure that any existing references or indices
    /// into the buffer are updated appropriately after calling this function.
    fn insert_text(
        &mut self,
        at: LineCol,
        text: String,
        newline: bool,
    ) -> Result<LineCol, BufferError> {
        if at.line >= self.lines.len() || at.col > self.lines[at.line].len() {
            return Err(BufferError::InvalidPosition);
        } else if text.is_empty() {
            return Err(BufferError::InvalidInput);
        }
        let mut resulting_cursor_pos = at;

        let mut lines: Vec<String> = text.lines().map(String::from).collect();
        if newline {
            lines.into_iter().rev().for_each(|line| {
                self.lines.insert(at.line + 1, line);
            });
            resulting_cursor_pos.line += 1;
            resulting_cursor_pos.col = 0;
        } else {
            let current_line = &mut self.lines[at.line];
            let tail = current_line.split_off(at.col);
            current_line.push_str(&lines[0]);

            if lines.len() > 1 {
                lines.last_mut().unwrap().push_str(&tail);
                self.lines
                    .splice(at.line + 1..at.line + 1, lines.into_iter().skip(1));
            } else {
                current_line.push_str(&tail);
            }
        };
        Ok(resulting_cursor_pos)
    }
    /// Deletes text from the buffer within the specified range.
    ///
    /// # Arguments
    ///
    /// * `from` - The starting position (line and column) of the text to delete, inclusive.
    /// * `to` - The ending position (line and column) of the text to delete, exclusive.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the deletion was successful.
    /// * `Err(BufferError::InvalidRange)` if the specified range is invalid.
    ///
    /// # Behavior
    ///
    /// This function removes text from the buffer between the `from` and `to` positions.
    /// It handles various scenarios:
    ///
    /// 1. Full line deletion: If the range starts at the beginning of a line and ends at or beyond
    ///    the end of a line (possibly spanning multiple lines), it removes entire lines.
    /// 2. Single line deletion: If `from` and `to` are on the same line, it removes the specified
    ///    range within that line.
    /// 3. Multi-line deletion: If the range spans multiple lines, it removes the specified content
    ///    and joins the remaining parts of the first and last lines.
    ///
    /// # Errors
    ///
    /// Returns `BufferError::InvalidRange` in the following cases:
    /// - If either `from` or `to` positions are beyond the buffer's contents.
    /// - If `from` position comes after `to` position.
    /// - If `from` and `to` are the same position.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buffer = // ... initialize buffer ...
    /// let from = LineCol { line: 1, col: 5 };
    /// let to = LineCol { line: 2, col: 10 };
    /// match buffer.delete(&from, &to) {
    ///     Ok(_) => println!("Text deleted successfully"),
    ///     Err(BufferError::InvalidRange) => println!("Invalid range specified"),
    ///     Err(_) => println!("An error occurred"),
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// This function modifies the buffer's content. After calling this function,
    /// line numbers and column positions after the deleted range may change.
    fn delete_selection(&mut self, from: LineCol, to: LineCol) -> Result<LineCol, BufferError> {
        if from.line >= self.lines.len()
            || to.line >= self.lines.len()
            || (from.line == to.line && from.col > to.col)
            || from.line > to.line
            || from == to
        {
            return Err(BufferError::InvalidRange);
        }

        if from.col == 0 && to.col >= self.lines[to.line].len() {
            self.lines.drain(from.line..=to.line);
            return Ok(LineCol {
                col: to.col,
                line: from.line,
            });
        }

        if from.line == to.line {
            let line = &mut self.lines[from.line];
            if from.col == 0 && to.col >= line.len() {
                self.lines.remove(from.line);
            } else if to.col >= line.len() {
                line.truncate(from.col);
            } else {
                line.replace_range(from.col..to.col, "");
            }
        } else {
            let new_last_line = self.lines[to.line].split_off(to.col);
            self.lines[from.line].truncate(from.col);
            self.lines[from.line].push_str(&new_last_line);
            self.lines.drain(from.line + 1..=to.line);
        }
        Ok(LineCol {
            col: to.col,
            line: from.line,
        })
    }
    fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    fn get_entire_text(&self) -> &Vec<String> {
        &self.lines
    }

    #[inline]
    fn delete(&mut self, mut at: LineCol) -> Result<LineCol, BufferError> {
        if at.line >= self.lines.len() || at.col > self.lines[at.line].len() {
            return Err(BufferError::InvalidPosition);
        }
        if at.col == 0 {
            if at.line == 0 {
                return Err(BufferError::ImATeacup);
            }

            let line_content = self.lines.remove(at.line);
            at.line -= 1;
            at.col = self.lines[at.line].len();
            self.lines[at.line].push_str(&line_content);
        } else {
            self.lines[at.line].remove(at.col - 1);
            at.col -= 1;
        }
        Ok(at)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    /// "First line"
    /// "Second line"
    /// "Third line"
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
        buf.replace(
            LineCol { line: 0, col: 6 },
            LineCol { line: 0, col: 10 },
            "text",
        )
        .unwrap();
        assert_eq!(buf.lines[0], "First text");
    }

    #[test]
    fn test_replace_across_multiple_lines() {
        let mut buf = new_test_buffer();
        buf.replace(
            LineCol { line: 0, col: 6 },
            LineCol { line: 2, col: 5 },
            "new\nreplacement\ntext",
        )
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
        let res = buf.replace(
            LineCol { line: 1, col: 0 },
            LineCol { line: 1, col: 11 },
            "",
        );
        assert_eq!(res, Err(BufferError::InvalidInput));
    }

    #[test]
    fn test_replacing_at_line_end() {
        let mut buf = new_test_buffer();
        buf.replace(
            LineCol { line: 1, col: 7 },
            LineCol { line: 1, col: 11 },
            "replacement",
        )
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
    fn test_replacing_with_more_new_lines_than_old() {
        let mut buf = new_test_buffer();
        buf.replace(
            LineCol { line: 0, col: 6 },
            LineCol { line: 2, col: 5 },
            "new\nreplacement\ntext\nthis also",
        )
        .unwrap();
        assert_eq!(
            buf.lines,
            vec![
                "First new".to_string(),
                "replacement".to_string(),
                "text".to_string(),
                "this also line".to_string()
            ]
        );
    }

    #[test]
    fn test_replacing_at_buffer_end() {
        let mut buf = new_test_buffer();
        buf.replace(
            LineCol { line: 2, col: 6 },
            LineCol { line: 2, col: 10 },
            "replacement",
        )
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
            past: Stack {
                content: VecDeque::new(),
            },
            future: Stack {
                content: VecDeque::new(),
            },
        }
    }

    #[test]
    fn test_rfind_basic() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.rfind("line", LineCol { line: 2, col: 0 }),
            Ok(LineCol { line: 1, col: 7 })
        );
    }

    #[test]
    fn test_rfind_not_including_start() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.rfind("line", LineCol { line: 1, col: 7 }),
            Ok(LineCol { line: 0, col: 6 })
        );
    }

    #[test]
    fn test_rfind_across_lines() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.rfind("First", LineCol { line: 2, col: 0 }),
            Ok(LineCol { line: 0, col: 0 })
        );
    }

    #[test]
    fn test_rfind_at_start_of_buffer() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.rfind("First", LineCol { line: 0, col: 4 }),
            Err(BufferError::PatternNotFound)
        );
    }

    #[test]
    fn test_rfind_pattern_not_found() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.rfind("nonexistent", LineCol { line: 2, col: 0 }),
            Err(BufferError::PatternNotFound)
        );
    }

    #[test]
    fn test_rfind_empty_query() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.rfind("", LineCol { line: 1, col: 5 }),
            Err(BufferError::InvalidInput)
        );
    }

    #[test]
    fn test_rfind_at_end_of_buffer() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.rfind("too", LineCol { line: 2, col: 22 }),
            Ok(LineCol { line: 2, col: 19 })
        );
    }
    #[test]
    fn test_find_basic() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("line", LineCol { line: 0, col: 0 }),
            Ok(LineCol { line: 0, col: 6 })
        );
    }

    #[test]
    fn test_find_from_middle() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("text", LineCol { line: 0, col: 10 }),
            Ok(LineCol { line: 0, col: 21 })
        );
    }

    #[test]
    fn test_find_across_lines() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("Second", LineCol { line: 0, col: 22 }),
            Ok(LineCol { line: 1, col: 0 })
        );
    }

    #[test]
    fn test_find_at_start_of_line() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("Third", LineCol { line: 2, col: 0 }),
            Ok(LineCol { line: 2, col: 0 })
        );
    }

    #[test]
    fn test_find_at_end_of_line() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("text", LineCol { line: 1, col: 0 }),
            Ok(LineCol { line: 1, col: 21 })
        );
    }

    #[test]
    fn test_find_pattern_not_found() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("nonexistent", LineCol { line: 0, col: 0 }),
            Err(BufferError::PatternNotFound)
        );
    }

    #[test]
    fn test_find_empty_query() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("", LineCol { line: 1, col: 5 }),
            Err(BufferError::InvalidInput)
        );
    }

    #[test]
    fn test_find_at_end_of_buffer() {
        let buf = new_test_buffer_find();
        let last_line = buf.lines.len() - 1;
        let last_col = buf.lines[last_line].len();
        assert_eq!(
            buf.find(
                "too",
                LineCol {
                    line: last_line,
                    col: last_col
                }
            ),
            Err(BufferError::PatternNotFound)
        );
    }

    #[test]
    fn test_find_exact_position() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("Second", LineCol { line: 1, col: 0 }),
            Ok(LineCol { line: 1, col: 0 })
        );
    }

    #[test]
    fn test_find_multiple_occurrences() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("line", LineCol { line: 0, col: 7 }),
            Ok(LineCol { line: 1, col: 7 })
        );
    }

    #[test]
    fn test_find_from_empty_line() {
        let mut buf = new_test_buffer_find();
        buf.lines.insert(1, String::new());
        assert_eq!(
            buf.find("Third", LineCol { line: 1, col: 0 }),
            Ok(LineCol { line: 3, col: 0 })
        );
    }
    /// "First line"
    /// "Second line"
    /// "Third line"
    /// "Fourth line"
    fn new_test_buffer_get() -> VecBuffer {
        VecBuffer {
            lines: vec![
                "First line".to_string(),
                "Second line".to_string(),
                "Third line".to_string(),
                "Fourth line".to_string(),
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
    fn test_get_text_single_line() {
        let buffer = new_test_buffer_get();
        assert_eq!(
            buffer.get_text(LineCol { line: 0, col: 0 }, LineCol { line: 0, col: 5 }),
            Ok("First".to_string())
        );
    }

    #[test]
    fn test_get_text_multiple_lines() {
        let buffer = new_test_buffer_get();
        assert_eq!(
            buffer.get_text(LineCol { line: 0, col: 6 }, LineCol { line: 2, col: 5 }),
            Ok("line\nSecond line\nThird".to_string())
        );
    }

    #[test]
    fn test_get_text_entire_line() {
        let buffer = new_test_buffer_get();
        assert_eq!(
            buffer.get_text(LineCol { line: 1, col: 0 }, LineCol { line: 2, col: 0 }),
            Ok("Second line\n".to_string())
        );
    }

    #[test]
    fn test_get_text_across_all_lines() {
        let buffer = new_test_buffer_get();
        assert_eq!(
            buffer.get_text(LineCol { line: 0, col: 0 }, LineCol { line: 3, col: 11 }),
            Ok("First line\nSecond line\nThird line\nFourth line".to_string())
        );
    }

    #[test]
    fn test_get_text_invalid_range_start_exceeds_end() {
        let buffer = new_test_buffer_get();
        assert_eq!(
            buffer.get_text(LineCol { line: 2, col: 0 }, LineCol { line: 1, col: 0 }),
            Err(BufferError::InvalidRange)
        );
    }

    #[test]
    fn test_get_text_invalid_range_exceeds_file_length() {
        let buffer = new_test_buffer_get();
        assert_eq!(
            buffer.get_text(LineCol { line: 0, col: 0 }, LineCol { line: 4, col: 0 }),
            Err(BufferError::InvalidRange)
        );
    }

    #[test]
    fn test_get_text_empty_range() {
        let buffer = new_test_buffer_get();
        assert_eq!(
            buffer.get_text(LineCol { line: 1, col: 5 }, LineCol { line: 1, col: 5 }),
            Ok("".to_string())
        );
    }

    #[test]
    fn test_get_text_buffer_exceeded() {
        let buffer = new_test_buffer_get();
        assert_eq!(
            buffer.get_text(LineCol { line: 0, col: 0 }, LineCol { line: 0, col: 11 }),
            Err(BufferError::InvalidRange)
        );
    }

    #[test]
    fn test_get_text_newline_handling() {
        let buffer = new_test_buffer_get();
        assert_eq!(
            buffer.get_text(LineCol { line: 0, col: 9 }, LineCol { line: 1, col: 1 }),
            Ok("e\nS".to_string())
        );
    }

    #[test]
    fn test_delete_within_line() {
        let mut buffer = new_test_buffer_get();
        buffer
            .delete_selection(LineCol { line: 0, col: 6 }, LineCol { line: 0, col: 10 })
            .unwrap();
        assert_eq!(buffer.lines[0], "First ");
    }

    #[test]
    fn test_delete_to_end_of_line() {
        let mut buffer = new_test_buffer_get();
        buffer
            .delete_selection(LineCol { line: 0, col: 6 }, LineCol { line: 0, col: 11 })
            .unwrap();
        assert_eq!(buffer.lines[0], "First ");
    }

    #[test]
    fn test_delete_entire_line() {
        let mut buffer = new_test_buffer_get();
        buffer
            .delete_selection(LineCol { line: 1, col: 0 }, LineCol { line: 1, col: 11 })
            .unwrap();
        assert_eq!(buffer.lines.len(), 3);
        assert_eq!(buffer.lines[1], "Third line");
    }

    #[test]
    fn test_delete_across_lines() {
        let mut buffer = new_test_buffer_get();
        buffer
            .delete_selection(LineCol { line: 0, col: 6 }, LineCol { line: 2, col: 6 })
            .unwrap();
        assert_eq!(buffer.lines.len(), 2);
        assert_eq!(buffer.lines[0], "First line");
    }

    #[test]
    fn test_delete_multiple_full_lines() {
        let mut buffer = new_test_buffer_get();
        buffer
            .delete_selection(LineCol { line: 1, col: 0 }, LineCol { line: 2, col: 10 })
            .unwrap();
        assert_eq!(buffer.lines.len(), 2);
        assert_eq!(buffer.lines[1], "Fourth line");
    }

    #[test]
    fn test_delete_invalid_range() {
        let mut buffer = new_test_buffer_get();
        let result =
            buffer.delete_selection(LineCol { line: 2, col: 0 }, LineCol { line: 1, col: 0 });
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_out_of_bounds() {
        let mut buffer = new_test_buffer_get();
        let result =
            buffer.delete_selection(LineCol { line: 0, col: 0 }, LineCol { line: 4, col: 0 });
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_empty_range() {
        let mut buffer = new_test_buffer_get();
        let result =
            buffer.delete_selection(LineCol { line: 1, col: 5 }, LineCol { line: 1, col: 5 });
        assert_eq!(result, Err(BufferError::InvalidRange));
    }
    #[test]
    fn test_insert_single_line_not_newline() {
        let mut buffer = new_test_buffer();
        buffer
            .insert_text(LineCol { line: 0, col: 5 }, "inserted ".to_string(), false)
            .unwrap();
        assert_eq!(buffer.lines[0], "Firstinserted  line");
    }

    #[test]
    fn test_insert_multi_line_not_newline() {
        let mut buffer = new_test_buffer();
        buffer
            .insert_text(
                LineCol { line: 0, col: 5 },
                "inserted\ntext".to_string(),
                false,
            )
            .unwrap();
        assert_eq!(buffer.lines[0], "Firstinserted");
        assert_eq!(buffer.lines[1], "text line");
    }

    #[test]
    fn test_insert_single_line_newline() {
        let mut buffer = new_test_buffer();
        buffer
            .insert_text(LineCol { line: 1, col: 0 }, "New line".to_string(), true)
            .unwrap();
        assert_eq!(buffer.lines[1], "Second line");
        assert_eq!(buffer.lines[2], "New line");
        assert_eq!(buffer.lines[3], "Third line");
    }

    #[test]
    fn test_insert_multi_line_newline() {
        let mut buffer = new_test_buffer();
        buffer
            .insert_text(LineCol { line: 1, col: 0 }, "New\nlines".to_string(), true)
            .unwrap();
        assert_eq!(buffer.lines[1], "Second line");
        assert_eq!(buffer.lines[2], "New");
        assert_eq!(buffer.lines[3], "lines");
        assert_eq!(buffer.lines[4], "Third line");
    }

    #[test]
    fn test_insert_at_end_of_line() {
        let mut buffer = new_test_buffer();
        buffer
            .insert_text(LineCol { line: 0, col: 10 }, " added".to_string(), false)
            .unwrap();
        assert_eq!(buffer.lines[0], "First line added");
    }

    #[test]
    fn test_insert_empty_string() {
        let mut buffer = new_test_buffer();
        let result = buffer.insert_text(LineCol { line: 0, col: 5 }, "".to_string(), false);
        assert_eq!(result, Err(BufferError::InvalidInput));
    }

    #[test]
    fn test_insert_invalid_position() {
        let mut buffer = new_test_buffer();
        let result = buffer.insert_text(LineCol { line: 3, col: 0 }, "Invalid".to_string(), false);
        assert_eq!(result, Err(BufferError::InvalidPosition));
    }

    #[test]
    fn test_insert_at_start_of_buffer() {
        let mut buffer = new_test_buffer();
        buffer
            .insert_text(LineCol { line: 0, col: 0 }, "Start: ".to_string(), false)
            .unwrap();
        assert_eq!(buffer.lines[0], "Start: First line");
    }

    #[test]
    fn test_insert_newline_at_end_of_buffer() {
        let mut buffer = new_test_buffer();
        buffer
            .insert_text(
                LineCol { line: 2, col: 0 },
                "New last line".to_string(),
                true,
            )
            .unwrap();
        assert_eq!(buffer.lines.last().unwrap(), "New last line");
    }
}
