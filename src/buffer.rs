use crate::{cursor::LineCol, modal::Modal};
use std::collections::VecDeque;

/// Trait defining the interface for a text buffer
pub trait TextBuffer {
    fn set_plane(&mut self, modal: &Modal);
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

    /// Get the entire text for the current buffer
    fn get_entire_text(&self) -> &Vec<String>;
    /// Get the entire text for the normal buffer
    fn get_normal_text(&self) -> &Vec<String>;
    /// Get the entire text for the terminal buffer
    fn get_terminal_text(&self) -> &str;
    /// Get the entire text for the command buffer
    fn get_command_text(&self) -> &Vec<String>;

    /// Get maximum line bound for the current buffer
    fn max_line(&self) -> usize;
    /// Get maximum column bound for the current buffer
    fn max_col(&self, at: LineCol) -> usize;
    fn is_command_empty(&self) -> bool;
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
    /// The current state of the normal text buffer, stored as a vector of lines.
    text: Vec<String>,
    /// The current state of the terminal buffer, stored as a vector of lines.
    terminal: Vec<String>,
    /// The current state of the command bar buffer, stored as a vector of a single line.
    command: Vec<String>,
    /// Stack to store past states for undo operations.
    past: Stack,
    /// Stack to store future states for redo operations.
    future: Stack,
    plane: BufferPlane,
}

#[derive(Default, Debug, Clone, Copy)]
enum BufferPlane {
    #[default]
    Normal,
    Terminal,
    Command,
}

impl Default for VecBuffer {
    fn default() -> Self {
        Self {
            text: vec!["".to_string()],
            terminal: vec!["".to_string()],
            command: vec!["".to_string()],
            past: Stack::default(),
            future: Stack::default(),
            plane: BufferPlane::Normal,
        }
    }
}

impl VecBuffer {
    fn get_mut_buffer(&mut self) -> &mut Vec<String> {
        match &self.plane {
            BufferPlane::Normal => &mut self.text,
            BufferPlane::Terminal => &mut self.terminal,
            BufferPlane::Command => &mut self.command,
        }
    }
    fn get_buffer(&self) -> &Vec<String> {
        match &self.plane {
            BufferPlane::Normal => &self.text,
            BufferPlane::Terminal => &self.terminal,
            BufferPlane::Command => &self.command,
        }
    }
}

impl TextBuffer for VecBuffer {
    fn is_command_empty(&self) -> bool {
        self.command[0].is_empty()
    }
    fn set_plane(&mut self, modal: &Modal) {
        self.plane = match modal {
            Modal::Command | Modal::Find => BufferPlane::Command,
            Modal::Normal | Modal::Insert | Modal::Visual => BufferPlane::Normal,
        };
    }
    fn max_col(&self, at: LineCol) -> usize {
        self.get_buffer()[at.line].len()
    }
    fn max_line(&self) -> usize {
        self.get_buffer().len() - 1
    }
    fn insert_newline(&mut self, mut at: LineCol) -> LineCol {
        self.get_mut_buffer()
            .insert(at.line + 1, Default::default());
        at.line += 1;
        at.col = 0;
        at
    }
    fn insert(&mut self, mut at: LineCol, ch: char) -> Result<LineCol, BufferError> {
        if at.line > self.get_buffer().len() || at.col > self.get_buffer()[at.line].len() {
            return Err(BufferError::InvalidPosition);
        }
        self.get_mut_buffer()[at.line].insert(at.col, ch);
        at.col += 1;
        Ok(at)
    }
    /// Performs a redo operation, moving the current state to the next future state if available.
    /// Returns an error if there are no `future` states to redo to.
    fn redo(&mut self, at: LineCol) -> Result<LineCol, BufferError> {
        self.future
            .pop()
            .map(|future_state| {
                let current_state = std::mem::replace(&mut self.text, future_state.content);
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
                let current_state = std::mem::replace(&mut self.text, past_state.content);
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

        while current_line < self.get_buffer().len() {
            if let Some(line) = self.get_buffer().get(current_line) {
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
            if let Some(line) = self.get_buffer().get(current_line) {
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
            current_col = self.get_buffer()[current_line].len();
        }

        Err(BufferError::PatternNotFound)
    }

    fn len(&self) -> usize {
        // Currently length of the entire file seems unnecessary to implement. If I realize it
        // needs to be implemented it might be as a counter at the level of a struct attribute.
        0
    }

    fn line_count(&self) -> usize {
        self.get_buffer().len()
    }
    fn line(&self, line_number: usize) -> Result<&str, BufferError> {
        if line_number > 0 && line_number <= self.line_count() {
            Ok(self
                .get_buffer()
                .get(line_number)
                .expect("Checks already passed"))
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
        let exceeds_file_len = from.line >= self.get_buffer().len()
            || to.line >= self.get_buffer().len()
            || from.col > self.get_buffer()[from.line].len()
            || to.col > self.get_buffer()[to.line].len();
        if start_exceeds_end || exceeds_file_len {
            return Err(BufferError::InvalidRange);
        }

        let mut result = String::new();

        if from.line == to.line {
            result.push_str(&self.get_buffer()[from.line][from.col..to.col]);
        } else {
            result.push_str(&self.get_buffer()[from.line][from.col..]);
            result.push('\n');

            for line in &self.get_buffer()[from.line + 1..to.line] {
                result.push_str(line);
                result.push('\n');
            }

            result.push_str(&self.get_buffer()[to.line][..to.col]);
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
            let start = &self.get_buffer()[from.line][..from.col];
            new_lines.push(format!("{}{}", start, first_line));
        } else {
            new_lines.push(self.get_buffer()[from.line][..from.col].to_string());
        }

        new_lines.extend(lines.map(String::from));

        let last = new_lines.last_mut().expect("We know there is a last line");
        last.push_str(&self.get_buffer()[to.line][to.col..]);

        self.get_mut_buffer().splice(from.line..=to.line, new_lines);

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
        if at.line >= self.get_buffer().len() || at.col > self.get_buffer()[at.line].len() {
            return Err(BufferError::InvalidPosition);
        } else if text.is_empty() {
            return Err(BufferError::InvalidInput);
        }
        let mut resulting_cursor_pos = at;

        let mut lines: Vec<String> = text.lines().map(String::from).collect();
        if newline {
            lines.into_iter().rev().for_each(|line| {
                self.get_mut_buffer().insert(at.line + 1, line);
            });
            resulting_cursor_pos.line += 1;
            resulting_cursor_pos.col = 0;
        } else {
            let current_line = &mut self.get_mut_buffer()[at.line];
            let tail = current_line.split_off(at.col);
            current_line.push_str(&lines[0]);

            if lines.len() > 1 {
                lines.last_mut().unwrap().push_str(&tail);
                self.get_mut_buffer()
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
        if from.line >= self.get_buffer().len()
            || to.line >= self.get_buffer().len()
            || (from.line == to.line && from.col > to.col)
            || from.line > to.line
            || from == to
        {
            return Err(BufferError::InvalidRange);
        }

        if from.col == 0 && to.col >= self.get_buffer()[to.line].len() {
            self.get_mut_buffer().drain(from.line..=to.line);
            return Ok(LineCol {
                col: to.col,
                line: from.line,
            });
        }

        if from.line == to.line {
            let line = &mut self.get_mut_buffer()[from.line];
            if from.col == 0 && to.col >= line.len() {
                self.get_mut_buffer().remove(from.line);
            } else if to.col >= line.len() {
                line.truncate(from.col);
            } else {
                line.replace_range(from.col..to.col, "");
            }
        } else {
            let new_last_line = self.get_mut_buffer()[to.line].split_off(to.col);
            self.get_mut_buffer()[from.line].truncate(from.col);
            self.get_mut_buffer()[from.line].push_str(&new_last_line);
            self.get_mut_buffer().drain(from.line + 1..=to.line);
        }
        Ok(LineCol {
            col: to.col,
            line: from.line,
        })
    }
    fn is_empty(&self) -> bool {
        self.get_buffer().is_empty()
    }
    fn get_entire_text(&self) -> &Vec<String> {
        self.get_buffer()
    }
    fn get_normal_text(&self) -> &Vec<String> {
        &self.text
    }
    fn get_command_text(&self) -> &Vec<String> {
        &self.command
    }
    fn get_terminal_text(&self) -> &str {
        &self.terminal[0]
    }
    #[inline]
    fn delete(&mut self, mut at: LineCol) -> Result<LineCol, BufferError> {
        if at.line >= self.get_buffer().len() || at.col > self.get_buffer()[at.line].len() {
            return Err(BufferError::InvalidPosition);
        }
        if at.col == 0 {
            if at.line == 0 {
                return Err(BufferError::ImATeacup);
            }

            let line_content = self.get_mut_buffer().remove(at.line);
            at.line -= 1;
            at.col = self.get_buffer()[at.line].len();
            self.get_mut_buffer()[at.line].push_str(&line_content);
        } else {
            self.get_mut_buffer()[at.line].remove(at.col - 1);
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
            text: vec![
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
            command: vec![],
            terminal: vec![],
            plane: BufferPlane::Normal,
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
        assert_eq!(buf.text[0], "First text");
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
            buf.text,
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
            buf.text,
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
            buf.text,
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
            buf.text,
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
            text: vec![
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
            command: vec![],
            terminal: vec![],
            plane: BufferPlane::Normal,
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
        let last_line = buf.text.len() - 1;
        let last_col = buf.text[last_line].len();
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
        buf.text.insert(1, String::new());
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
            text: vec![
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
            command: vec![],
            terminal: vec![],
            plane: BufferPlane::Normal,
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
        assert_eq!(buffer.text[0], "First ");
    }

    #[test]
    fn test_delete_to_end_of_line() {
        let mut buffer = new_test_buffer_get();
        buffer
            .delete_selection(LineCol { line: 0, col: 6 }, LineCol { line: 0, col: 11 })
            .unwrap();
        assert_eq!(buffer.text[0], "First ");
    }

    #[test]
    fn test_delete_entire_line() {
        let mut buffer = new_test_buffer_get();
        buffer
            .delete_selection(LineCol { line: 1, col: 0 }, LineCol { line: 1, col: 11 })
            .unwrap();
        assert_eq!(buffer.text.len(), 3);
        assert_eq!(buffer.text[1], "Third line");
    }

    #[test]
    fn test_delete_across_lines() {
        let mut buffer = new_test_buffer_get();
        buffer
            .delete_selection(LineCol { line: 0, col: 6 }, LineCol { line: 2, col: 6 })
            .unwrap();
        assert_eq!(buffer.text.len(), 2);
        assert_eq!(buffer.text[0], "First line");
    }

    #[test]
    fn test_delete_multiple_full_lines() {
        let mut buffer = new_test_buffer_get();
        buffer
            .delete_selection(LineCol { line: 1, col: 0 }, LineCol { line: 2, col: 10 })
            .unwrap();
        assert_eq!(buffer.text.len(), 2);
        assert_eq!(buffer.text[1], "Fourth line");
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
        assert_eq!(buffer.text[0], "Firstinserted  line");
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
        assert_eq!(buffer.text[0], "Firstinserted");
        assert_eq!(buffer.text[1], "text line");
    }

    #[test]
    fn test_insert_single_line_newline() {
        let mut buffer = new_test_buffer();
        buffer
            .insert_text(LineCol { line: 1, col: 0 }, "New line".to_string(), true)
            .unwrap();
        assert_eq!(buffer.text[1], "Second line");
        assert_eq!(buffer.text[2], "New line");
        assert_eq!(buffer.text[3], "Third line");
    }

    #[test]
    fn test_insert_multi_line_newline() {
        let mut buffer = new_test_buffer();
        buffer
            .insert_text(LineCol { line: 1, col: 0 }, "New\nlines".to_string(), true)
            .unwrap();
        assert_eq!(buffer.text[1], "Second line");
        assert_eq!(buffer.text[2], "New");
        assert_eq!(buffer.text[3], "lines");
        assert_eq!(buffer.text[4], "Third line");
    }

    #[test]
    fn test_insert_at_end_of_line() {
        let mut buffer = new_test_buffer();
        buffer
            .insert_text(LineCol { line: 0, col: 10 }, " added".to_string(), false)
            .unwrap();
        assert_eq!(buffer.text[0], "First line added");
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
        assert_eq!(buffer.text[0], "Start: First line");
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
        assert_eq!(buffer.text.last().unwrap(), "New last line");
    }
    #[test]
    fn test_set_plane_and_buffer_operations() {
        let mut buffer = VecBuffer::default();

        // Start in Normal mode
        assert_eq!(buffer.get_buffer(), &buffer.text);

        // Insert text in Normal mode
        buffer
            .insert_text(
                LineCol { line: 0, col: 0 },
                "Normal text".to_string(),
                false,
            )
            .unwrap();
        assert_eq!(buffer.text, vec!["Normal text"]);

        // Switch to Command mode
        buffer.set_plane(&Modal::Command);
        assert_eq!(buffer.get_buffer(), &buffer.command);

        // Insert text in Command mode
        buffer
            .insert_text(
                LineCol { line: 0, col: 0 },
                "Command text".to_string(),
                false,
            )
            .unwrap();
        assert_eq!(buffer.command, vec!["Command text"]);

        // Switch to Normal mode and verify text
        buffer.set_plane(&Modal::Normal);
        assert_eq!(buffer.get_buffer(), &buffer.text);
        assert_eq!(buffer.text, vec!["Normal text"]);
    }

    #[test]
    fn test_buffer_independence() {
        let mut buffer = VecBuffer::default();

        // Insert text in Normal mode
        buffer.set_plane(&Modal::Normal);
        buffer
            .insert_text(
                LineCol { line: 0, col: 0 },
                "Normal text".to_string(),
                false,
            )
            .unwrap();

        // Insert text in Command mode
        buffer.set_plane(&Modal::Command);
        buffer
            .insert_text(
                LineCol { line: 0, col: 0 },
                "Command text".to_string(),
                false,
            )
            .unwrap();

        // Verify that buffers remain independent
        buffer.set_plane(&Modal::Normal);
        assert_eq!(buffer.text, vec!["Normal text"]);
        buffer.set_plane(&Modal::Command);
        assert_eq!(buffer.command, vec!["Command text"]);
    }

    #[test]
    fn test_find_across_buffers() {
        let mut buffer = VecBuffer::default();

        // Insert text in Normal mode
        buffer.set_plane(&Modal::Normal);
        buffer
            .insert_text(
                LineCol { line: 0, col: 0 },
                "Normal text to find".to_string(),
                false,
            )
            .unwrap();

        // Insert text in Command mode
        buffer.set_plane(&Modal::Command);
        buffer
            .insert_text(
                LineCol { line: 0, col: 0 },
                "Command text to find".to_string(),
                false,
            )
            .unwrap();

        // Find in Normal mode
        buffer.set_plane(&Modal::Normal);
        let result = buffer.find("to find", LineCol { line: 0, col: 0 });
        assert_eq!(result, Ok(LineCol { line: 0, col: 12 }));

        // Find in Command mode
        buffer.set_plane(&Modal::Command);
        let result = buffer.find("to find", LineCol { line: 0, col: 0 });
        assert_eq!(result, Ok(LineCol { line: 0, col: 13 }));
    }

    #[test]
    fn test_delete_across_buffers() {
        let mut buffer = VecBuffer::default();

        // Insert and delete in Normal mode
        buffer.set_plane(&Modal::Normal);
        buffer
            .insert_text(
                LineCol { line: 0, col: 0 },
                "Normal text".to_string(),
                false,
            )
            .unwrap();
        buffer
            .delete_selection(LineCol { line: 0, col: 0 }, LineCol { line: 0, col: 6 })
            .unwrap();
        assert_eq!(buffer.text, vec![" text"]);

        // Insert and delete in Command mode
        buffer.set_plane(&Modal::Command);
        buffer
            .insert_text(
                LineCol { line: 0, col: 0 },
                "Command text".to_string(),
                false,
            )
            .unwrap();
        buffer
            .delete_selection(LineCol { line: 0, col: 0 }, LineCol { line: 0, col: 7 })
            .unwrap();
        assert_eq!(buffer.command, vec![" text"]);

        // Verify Normal mode text remains unchanged
        buffer.set_plane(&Modal::Normal);
        assert_eq!(buffer.text, vec![" text"]);
    }
}
