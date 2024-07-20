use std::{collections::VecDeque, ops::Range};


/// Trait defining the interface for a text buffer
pub trait TextBuffer {
    /// Insert text at the specified position
    fn insert(&mut self, position: usize, text: &str) -> Result<(), BufferError>;

    /// Delete text in the specified range
    fn delete(&mut self, range: Range<usize>) -> Result<(), BufferError>;

    /// Replace text in the specified range with new text
    fn replace(&mut self, range: Range<usize>, text: &str) -> Result<(), BufferError>;

    /// Get the text in the specified range
    fn get_text(&self, range: Range<usize>) -> Result<String, BufferError>;

    /// Get the length of the entire buffer
    fn len(&self) -> usize;

    /// Check if the buffer is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the number of lines in the buffer
    fn line_count(&self) -> usize;

    /// Get the contents of a specific line
    fn line(&self, line_number: usize) -> Result<String, BufferError>;

    /// Convert a position to a (line, column) pair
    fn position_to_line_col(&self, position: usize) -> Result<(usize, usize), BufferError>;

    /// Convert a (line, column) pair to a position
    fn line_col_to_position(&self, line: usize, column: usize) -> Result<usize, BufferError>;

    /// Find the next occurrence of a substring
    fn find(&self, query: &str, start_position: usize) -> Option<usize>;

    /// Find the previous occurrence of a substring
    fn rfind(&self, query: &str, start_position: usize) -> Option<usize>;

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
    /// Returns an error if there are no future states to redo to.
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
    /// Returns an error if there are no past states to undo to.
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
}

