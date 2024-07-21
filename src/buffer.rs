use std::{collections::VecDeque, ops::Range};
use crate::cursor::{Cursor, LineCol};



/// Trait defining the interface for a text buffer
pub trait TextBuffer {
    /// Insert text at the specified position
    fn insert(&mut self, cursor: &mut Cursor, text: &str) -> Result<(), BufferError>;

    /// Delete text in the specified range
    fn delete(&mut self, from: LineCol, to: LineCol) -> Result<(), BufferError>;

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
    fn find(&self, query: &str, cursor: Cursor) -> Option<usize>;

    /// Find the previous occurrence of a substring
    fn rfind(&self, query: &str, cursor: Cursor) -> Option<usize>;

    /// Undo the last operation
    fn undo(&mut self) -> Result<(), BufferError>;

    /// Redo the last undone operation
    fn redo(&mut self) -> Result<(), BufferError>;
}

/// Error type for buffer operations
#[derive(Debug)]
pub enum BufferError {
    InvalidPosition,
    InvalidRange,
    InvalidLineNumber,
    OperationFailed,
    NowhereToGo
}


/// A stack implementation using a VecDeque as the underlying storage.
pub struct Stack<T> {
    content: VecDeque<T>
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
        self.future.pop()
            .map(|future_state| {
                let current_state = std::mem::replace(&mut self.lines, future_state);
                self.past.push(current_state);
            })
            .map_or_else(
                || Err(BufferError::NowhereToGo),
                |_| Ok(())
            )
    }

    /// Performs an undo operation, moving the current state to the previous past state if available.
    /// Returns an error if there are no `past` states to undo to.
    fn undo(&mut self) -> Result<(), BufferError> {
        self.past.pop()
            .map(|past_state| {
                let current_state = std::mem::replace(&mut self.lines, past_state);
                self.future.push(current_state);
            })
            .map_or_else(
                || Err(BufferError::NowhereToGo),
                |_| Ok(())
            )
    }

    fn find(&self, query: &str, cursor: Cursor) -> Option<usize> {
        let (_, right) = self.lines[cursor.line()].split_at(cursor.col());
        right.find(&query)
    }

    fn rfind(&self, query: &str, cursor: Cursor) -> Option<usize> {
        let (_, left) = self.lines[cursor.line()].split_at(cursor.col());
        left.find(&query)
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
        if line_number >= 0 && line_number <= self.line_count() {
            Ok(self.lines.get(line_number).expect("Checks already passed"))
        } else {
            Err(BufferError::InvalidLineNumber)
        }
    }
    fn get_text(&self, from: LineCol, to: LineCol) -> Result<String, BufferError> {
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
    fn replace(&mut self, from: LineCol, to: LineCol, text: &str) -> Result<(), BufferError> {
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

       for l in from.0+1..=to.0 {
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
    fn insert(&mut self, cursor: &mut Cursor, text: &str) -> Result<(), BufferError> {
        unimplemented!()
    }
    fn delete(&mut self, from: LineCol, to: LineCol) -> Result<(), BufferError> {
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
            past: Stack { content: VecDeque::new() },
            future: Stack { content: VecDeque::new() },
        }
    }

    #[test]
    fn test_replace_within_single_line() {
        let mut buf = new_test_buffer();
        buf.replace(LineCol(0, 6), LineCol(0, 10), "text").unwrap();
        assert_eq!(buf.lines[0], "First text");
    }

    #[test]
    fn test_replace_across_multiple_lines() {
        let mut buf = new_test_buffer();
        buf.replace(LineCol(0, 6), LineCol(2, 5), "new\nreplacement\ntext").unwrap();
        assert_eq!(buf.lines, vec![
            "First new".to_string(),
            "replacement".to_string(),
            "text line".to_string(),
        ]);
    }

    #[test]
    fn test_replacing_with_empty_string() {
        let mut buf = new_test_buffer();
        buf.replace(LineCol(1, 0), LineCol(1, 11), "").unwrap();
        assert_eq!(buf.lines, vec![
            "First line".to_string(),
            "Third line".to_string(),
        ]);
    }

    #[test]
    fn test_replacing_at_line_end() {
        let mut buf = new_test_buffer();
        buf.replace(LineCol(1, 7), LineCol(1, 11), "replacement").unwrap();
        assert_eq!(buf.lines, vec![
            "First line".to_string(),
            "Second replacement".to_string(),
            "Third line".to_string(),
        ]);
    }

    #[test]
    fn test_replacing_with_overflowing_lines() {
        let mut buf = new_test_buffer();
        buf.replace(LineCol(0, 6), LineCol(2, 5), "new\nreplacement\ntext\nthisalso").unwrap();
        assert_eq!(buf.lines, vec![
            "First new".to_string(),
            "replacement".to_string(),
            "text".to_string(),
            "thisalso line".to_string()
        ]);
    }

    #[test]
    fn test_replacing_at_buffer_end() {
        let mut buf = new_test_buffer();
        buf.replace(LineCol(2, 5), LineCol(2, 10), "replacement").unwrap();
        assert_eq!(buf.lines, vec![
            "First line".to_string(),
            "Second line".to_string(),
            "Third replacement".to_string(),
        ]);
    }
}
