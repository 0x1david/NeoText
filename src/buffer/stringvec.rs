use std::collections::VecDeque;

use super::{BufferPlane, TextBuffer};
use crate::modals::Modal;
use crate::{Error, LineCol, Pattern, Result};

/// A buffer implementation for storing text as a vector of lines,
/// with undo and redo functionality. Highly inefficient, both tim complexity wise and implementation wise. Simply a placeholder for testing.
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct StringVec {
    /// The current state of the normal text buffer, stored as a vector of lines.
    pub(super) text: Vec<String>,
    /// The current state of the terminal buffer, stored as a vector of lines.
    pub(super) terminal: Vec<String>,
    /// The current state of the command bar buffer, stored as a vector of a single line.
    pub(super) command: Vec<String>,
    /// Stack to store past states for undo operations.
    pub(super) past: Stack,
    /// Stack to store future states for redo operations.
    pub(super) future: Stack,
    pub(super) plane: BufferPlane,
}

#[derive(Debug, Default)]
pub struct Stack {
    pub(super) content: VecDeque<StateCapsule>,
}

impl Stack {
    /// Truncates the stack to a maximum of 1000 elements.
    /// If the stack has more than 1000 elements, it removes the excess from the back.
    fn truncate(&mut self) {
        let len = self.content.len();
        if len > 1000 {
            self.content.truncate(1000);
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

impl Default for StringVec {
    fn default() -> Self {
        Self {
            text: vec![String::new()],
            terminal: vec![String::new()],
            command: vec![String::new()],
            past: Stack::default(),
            future: Stack::default(),
            plane: BufferPlane::Normal,
        }
    }
}

impl StringVec {
    pub fn new(text: Vec<String>) -> Self {
        Self {
            text,
            terminal: vec![String::new()],
            command: vec![String::new()],
            past: Stack::default(),
            future: Stack::default(),
            plane: BufferPlane::Normal,
        }
    }
    fn get_mut_buffer(&mut self) -> &mut Vec<String> {
        match &self.plane {
            BufferPlane::Normal => &mut self.text,
            BufferPlane::Terminal => &mut self.terminal,
            BufferPlane::Command => &mut self.command,
        }
    }
    pub(super) fn get_buffer(&self) -> &[String] {
        match &self.plane {
            BufferPlane::Normal => &self.text,
            BufferPlane::Terminal => &self.terminal,
            BufferPlane::Command => &self.command,
        }
    }
}

impl TextBuffer for StringVec {
    /// Get entire text as a single vec of bytes.
    /// This method clones the buffer, and thus should be only done for the initial parsing of the
    /// tree
    fn get_coalesced_bytes(&self) -> Vec<u8> {
        self.text.join("\n").as_bytes().to_owned()
    }
    // Gets only partial buffer from a position to a position
    fn get_buffer_window(&self, from: Option<LineCol>, to: Option<LineCol>) -> Result<Vec<String>> {
        if from.is_none() && to.is_none() {
            return Ok(self.get_normal_text().to_owned());
        }
        let from = from.unwrap_or(LineCol { line: 0, col: 0 });
        let mut to = to.unwrap_or_else(|| self.max_linecol());
        to.line = self.max_line().min(to.line);
        if from.line > to.line || (from.line == to.line && from.col > to.col) {
            return Err(Error::InvalidInput);
        }

        let mut vec = self.get_normal_text()[from.line..=to.line].to_owned();
        vec[0] = vec[0][from.col..].to_string();
        let last = vec.len() - 1;
        if from.line == to.line {
            vec[last] = vec[last][..to.col - from.col].to_string();
        } else {
            vec[last].truncate(to.col);
        }
        if to.col == 0 {
            let _ = vec.pop();
        }

        Ok(vec)
    }
    fn get_full_lines_buffer_window(
        &self,
        from: Option<LineCol>,
        to: Option<LineCol>,
    ) -> Result<Vec<String>> {
        let full_text = self.get_normal_text();

        let start_line = from.map_or(0, |lc| lc.line);
        let end_line = to.map_or_else(|| full_text.len().saturating_sub(1), |lc| lc.line);

        if start_line > end_line || start_line >= full_text.len() {
            return Err(Error::InvalidInput);
        }

        let end_line = end_line.min(full_text.len().saturating_sub(1));
        let result = full_text[start_line..=end_line].to_vec();
        Ok(result)
    }
    fn replace_command_text(&mut self, new: impl Into<String>) {
        self.command = vec![new.into()];
    }
    fn delete_line(&mut self, at: usize) {
        let _ = self.text.remove(at);
    }
    fn clear_command(&mut self) {
        self.command.clear();
        self.command.push(String::new());
    }
    fn is_command_empty(&self) -> bool {
        self.command[0].is_empty()
    }
    fn set_plane(&mut self, modal: &Modal) {
        self.plane = match modal {
            Modal::Command | Modal::Find(_) => BufferPlane::Command,
            Modal::Normal | Modal::Insert | Modal::Visual | Modal::VisualLine => {
                BufferPlane::Normal
            }
        };
    }
    fn max_col(&self, at: LineCol) -> usize {
        let buf = self.get_buffer();
        if buf.is_empty() {
            0
        } else {
            buf[at.line].len()
        }
    }
    fn max_line(&self) -> usize {
        self.get_normal_text().len().saturating_sub(1)
    }
    fn max_linecol(&self) -> LineCol {
        let buf = self.get_normal_text();
        let line = buf.len() - 1;
        let col = buf[line].len();
        LineCol { line, col }
    }
    fn insert_newline(&mut self, mut at: LineCol) -> LineCol {
        self.get_mut_buffer().insert(at.line + 1, String::new());
        at.line += 1;
        at.col = 0;
        at
    }
    fn insert(&mut self, mut at: LineCol, ch: char) -> Result<LineCol> {
        if at.line > self.get_buffer().len() || at.col > self.get_buffer()[at.line].len() {
            return Err(Error::InvalidPosition);
        }
        self.get_mut_buffer()[at.line].insert(at.col, ch);
        at.col += 1;
        Ok(at)
    }
    /// Performs a redo operation, moving the current state to the next future state if available.
    /// Returns an error if there are no `future` states to redo to.
    fn redo(&mut self, at: LineCol) -> Result<LineCol> {
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
            .map_or_else(|| Err(Error::NowhereToGo), Ok)
    }

    /// Performs an undo operation, moving the current state to the previous past state if available.
    /// Returns an error if there are no `past` states to undo to.
    fn undo(&mut self, at: LineCol) -> Result<LineCol> {
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
            .map_or_else(|| Err(Error::NowhereToGo), Ok)
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
    fn find(&self, query: impl Pattern, at: LineCol) -> Result<LineCol> {
        query
            .find_pattern(&self.get_buffer_window(Some(at), None)?)
            .ok_or(Error::PatternNotFound)
            .map(|v| LineCol {
                line: v.line + at.line,
                col: if v.line == 0 { v.col + at.col } else { v.col },
            })
    }

    /// Searches backwards for a query string in the buffer, ending at a given position.
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
    fn rfind(&self, query: impl Pattern, at: LineCol) -> Result<LineCol> {
        query
            .rfind_pattern(&self.get_buffer_window(None, Some(at))?)
            .ok_or(Error::PatternNotFound)
            .map(|v| LineCol {
                line: v.line,
                col: v.col,
            })
    }

    fn len(&self) -> usize {
        // Currently length of the entire file seems unnecessary to implement. If I realize it
        // needs to be implemented it might be as a counter at the level of a struct attribute.
        0
    }

    fn line_count(&self) -> usize {
        self.get_buffer().len()
    }
    fn line(&self, line_number: usize) -> Result<&str> {
        if line_number <= self.line_count() {
            Ok(self
                .get_buffer()
                .get(line_number)
                .expect("Checks already passed"))
        } else {
            Err(Error::InvalidLineNumber)
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
    fn get_text(&self, from: LineCol, to: LineCol) -> Result<String> {
        let buffer = self.get_buffer();
        let start_exceeds_end = from.line > to.line || (from.line == to.line && from.col > to.col);
        let exceeds_file_len = from.line >= buffer.len()
            || to.line >= buffer.len()
            || from.col > buffer[from.line].len()
            || to.col > buffer[to.line].len();
        if start_exceeds_end || exceeds_file_len {
            return Err(Error::InvalidRange);
        }

        if from.line == to.line {
            Ok(buffer[from.line][from.col..to.col].to_string())
        } else {
            Ok(buffer[from.line..=to.line]
                .iter()
                .enumerate()
                .map(|(i, line)| match i {
                    0 => line[from.col..].to_string(),
                    i if i == to.line - from.line => line[..to.col].to_string(),
                    _ => line.to_string(),
                })
                .collect::<Vec<_>>()
                .join("\n"))
        }
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
    fn replace(&mut self, from: LineCol, to: LineCol, text: &str) -> Result<()> {
        if text.is_empty() {
            return Err(Error::InvalidInput);
        }
        let mut new_lines = Vec::new();
        let mut lines = text.lines();

        if let Some(first_line) = lines.next() {
            let start = &self.get_buffer()[from.line][..from.col];
            new_lines.push(format!("{start}{first_line}"));
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
        text: impl Into<String>,
        newline: bool,
    ) -> Result<LineCol> {
        let text = text.into();
        if at.line >= self.get_buffer().len() || at.col > self.get_buffer()[at.line].len() {
            return Err(Error::InvalidPosition);
        } else if text.is_empty() {
            return Err(Error::InvalidInput);
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
                    .splice(at.line + 1..=at.line, lines.into_iter().skip(1));
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
    fn delete_selection(&mut self, from: LineCol, to: LineCol) -> Result<LineCol> {
        let buf = self.get_mut_buffer();
        if from.line >= buf.len()
            || to.line >= buf.len()
            || (from.line == to.line && from.col > to.col)
            || from.line > to.line
            || from == to
        {
            return Err(Error::InvalidRange);
        }

        if from.col == 0 && to.col >= buf[to.line].len() {
            buf.drain(from.line..=to.line);
            return Ok(LineCol {
                col: to.col,
                line: from.line,
            });
        }

        if from.line == to.line {
            let line = &mut buf[from.line];
            if from.col == 0 && to.col >= line.len() {
                buf.remove(from.line);
            } else if to.col >= line.len() {
                line.truncate(from.col);
            } else {
                line.replace_range(from.col..to.col, "");
            }
        } else {
            let end_line_tail = buf[to.line].split_off(to.col);
            buf[from.line].truncate(from.col);
            buf[from.line].push_str(&end_line_tail);
            buf.drain(from.line + 1..=to.line);
        }
        Ok(LineCol {
            col: to.col,
            line: from.line,
        })
    }
    fn is_empty(&self) -> bool {
        self.get_buffer().is_empty()
    }
    fn get_entire_text(&self) -> &[String] {
        self.get_buffer()
    }
    fn get_normal_text(&self) -> &[String] {
        &self.text
    }
    fn get_command_text(&self) -> &[String] {
        &self.command
    }
    fn get_terminal_text(&self) -> &str {
        &self.terminal[0]
    }
    fn delete(&mut self, mut at: LineCol) -> Result<LineCol> {
        let buf = self.get_mut_buffer();
        if at.line >= buf.len() || at.col > buf[at.line].len() {
            return Err(Error::InvalidPosition);
        }
        if at.col == 0 {
            if at.line == 0 {
                return Err(Error::ImATeacup);
            }

            let line_content = buf.remove(at.line);
            at.line -= 1;
            at.col = buf[at.line].len();
            buf[at.line].push_str(&line_content);
        } else {
            buf[at.line].remove(at.col - 1);
            at.col -= 1;
        }
        Ok(at)
    }
    /// Return the byte offset at which a character at a given linecol starts.
    fn get_byte_offset(&self, at: LineCol) -> usize {
        self.get_buffer_window(None, Some(at))
            .map(|window| window.iter().map(|s| s.len() + 1).sum())
            .unwrap_or(0)
    }
}
