use std::collections::{HashMap, VecDeque};

use super::{BufferPlane, TextBuffer};
use crate::modals::Modal;
use crate::{Error, LineCol, Pattern, Result};
const GROWTH_FACTOR_PERCENT: usize = 150; // If value is 150, new buffer will be 150% size of the last
const MIN_GROWTH: usize = 512;

struct GapBuf {
    buffer: Vec<char>,
    start: usize,
    current_line: usize,
    end: usize,
    plane: BufferPlane,
    line_starts: Vec<usize>,
}
impl GapBuf {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec!['\0'; capacity],
            start: 0,
            current_line: 0,
            end: capacity,
            plane: BufferPlane::default(),
            line_starts: vec![0],
        }
    }

    fn insert(&mut self, ch: char) {
        if self.start == self.end {
            self.grow()
        }
        if ch == '\n' {
            self.current_line += 1;
            self.line_starts.insert(self.current_line, self.start + 1);
        }
        self.buffer[self.start] = ch;
        self.start += 1;

        self.update_line_starts_after_insertion();
    }

    fn remove(&mut self) {
        if self.start > 0 {
            let char_at_cursor = self.buffer[self.start - 1];
            self.start -= 1;

            if char_at_cursor == '\n' {
                if self.current_line < self.line_starts.len() {
                    self.line_starts.remove(self.current_line);
                }
                self.current_line = self.current_line.saturating_sub(1);
            }

            self.update_line_starts_after_deletion();
        }
    }

    fn move_left(&mut self) {
        if self.start > 0 {
            let char_at_cursor = self.buffer[self.start - 1];
            self.start -= 1;
            self.end -= 1;
            self.buffer[self.end] = self.buffer[self.start];
            self.buffer[self.start] = '\0';

            if char_at_cursor == '\n' {
                self.current_line = self.current_line.saturating_sub(1);
            }
        }
    }

    fn move_right(&mut self) {
        if self.end + 1 < self.buffer.len() {
            let char_moving = self.buffer[self.end];
            self.buffer[self.start] = char_moving;
            self.buffer[self.end] = '\0';
            self.start += 1;
            self.end += 1;

            if char_moving == '\n' {
                self.current_line += 1;
            }
        }
    }

    fn grow(&mut self) {
        let old_capacity = self.buffer.len();
        let mut new_capacity = (old_capacity / 2) * GROWTH_FACTOR_PERCENT;
        new_capacity = new_capacity.max(MIN_GROWTH);
        let mut new_buf = vec!['\0'; new_capacity];
        new_buf[..self.start].copy_from_slice(&self.buffer[..self.start]);
        let gap_size = new_capacity - old_capacity;
        new_buf[self.start + gap_size..].copy_from_slice(&self.buffer[self.end..]);
        self.buffer = new_buf;
        self.end = self.start + gap_size;

        for line_start in &mut self.line_starts {
            if *line_start > self.start {
                *line_start += gap_size;
            }
        }
    }

    fn current_linecol(&self) -> LineCol {
        let line_start = self
            .line_starts
            .get(self.current_line)
            .copied()
            .unwrap_or(0);
        LineCol {
            line: self.current_line,
            col: self.start - line_start,
        }
    }

    fn update_line_starts_after_insertion(&mut self) {
        for i in (self.current_line + 1)..self.line_starts.len() {
            self.line_starts[i] += 1;
        }
    }

    fn update_line_starts_after_deletion(&mut self) {
        for i in (self.current_line + 1)..self.line_starts.len() {
            self.line_starts[i] -= 1;
        }
    }

    fn move_to_line(&mut self, target_line: usize) {
        if target_line < self.line_starts.len() {
            let line_start = self.line_starts[target_line];
            self.move_cursor_to(line_start, false);
            self.current_line = target_line
        } else {
            panic!("Line {} does not exist", target_line);
        }
    }

    // Recalculate line is not necessary if you already know the line
    fn move_cursor_to(&mut self, position: usize, recalculate_line: bool) {
        match position.cmp(&self.start) {
            std::cmp::Ordering::Less => {
                let chars_to_move = self.start - position;
                self.buffer
                    .copy_within(position..self.start, self.end - chars_to_move);
                self.start = position;
                self.end -= chars_to_move;
            }
            std::cmp::Ordering::Greater => {
                let chars_to_move = position - self.start;
                let available = self.buffer.len() - self.end;
                let actual_move = chars_to_move.min(available);
                self.buffer
                    .copy_within(self.end..self.end + actual_move, self.start);
                self.start += actual_move;
                self.end += actual_move;
            }
            _ => (),
        }
        if recalculate_line {
            self.current_line = self
                .line_starts
                .partition_point(|&start| start <= self.start)
                .saturating_sub(1);
        }
    }
}

// impl TextBuffer for GapBuf {
//     fn set_plane(&mut self, modal: &Modal) {
//         self.plane = match modal {
//             Modal::Command | Modal::Find(_) => BufferPlane::Command,
//             Modal::Normal | Modal::Insert | Modal::Visual | Modal::VisualLine => {
//                 BufferPlane::Normal
//             }
//         };
//     }
//     fn insert_newline(&mut self, at: LineCol) -> LineCol;
//     fn get_byte_offset(&self, to: LineCol) -> usize;
//     /// Insert a single symbol at specified position
//     fn insert(&mut self, at: LineCol, insertable: char) -> Result<LineCol>;

//     /// Insert text at the specified position
//     fn insert_text(
//         &mut self,
//         at: LineCol,
//         text: impl Into<String>,
//         newline: bool,
//     ) -> Result<LineCol>;

//     /// Delete text in the specified range
//     fn delete_selection(&mut self, from: LineCol, to: LineCol) -> Result<LineCol>;

//     /// Delete the symbol at the specified position
//     fn delete(&mut self, at: LineCol) -> Result<LineCol>;

//     /// Replace text in the specified range with new text
//     fn replace(&mut self, from: LineCol, to: LineCol, text: &str) -> Result<()>;

//     /// Get the text in the specified range
//     fn get_text(&self, from: LineCol, to: LineCol) -> Result<String>;

//     /// Get the length of the entire buffer
//     fn len(&self) -> usize;

//     /// Check if the buffer is empty
//     fn is_empty(&self) -> bool {
//         self.len() == 0
//     }

//     /// Get the number of lines in the buffer
//     fn line_count(&self) -> usize;

//     /// Get a single continuous vec of bytes containing the entire text
//     fn get_coalesced_bytes(&self) -> Vec<u8>;

//     /// Get the contents of a specific line
//     fn line(&self, line_number: usize) -> Result<&str>;

//     /// Find the next occurrence of a Pattern
//     fn find(&self, query: impl Pattern, at: LineCol) -> Result<LineCol>;

//     /// Find the previous occurrence of a Pattern
//     fn rfind(&self, query: impl Pattern, at: LineCol) -> Result<LineCol>;

//     /// Undo the last operation
//     fn undo(&mut self, at: LineCol) -> Result<LineCol>;

//     /// Redo the last undone operation
//     fn redo(&mut self, at: LineCol) -> Result<LineCol>;

//     /// Get the entire text for the current buffer
//     fn get_entire_text(&self) -> &[String];
//     /// Get the entire text for the normal buffer
//     fn get_normal_text(&self) -> &[String];

//     /// Get partial window to the normal buffer, ranging from -> to
//     fn get_buffer_window(&self, from: Option<LineCol>, to: Option<LineCol>) -> Result<Vec<String>>;

//     /// Get the entire text for the terminal buffer
//     fn get_terminal_text(&self) -> &str;
//     /// Get the entire text for the command buffer
//     fn get_command_text(&self) -> &[String];
//     /// Get the entire text for the command buffer
//     fn replace_command_text(&mut self, new: impl Into<String>);

//     /// Get maximum line bound for the current buffer
//     fn max_line(&self) -> usize;
//     /// Get maximum column bound for the current buffer
//     fn max_col(&self, at: LineCol) -> usize;
//     fn is_command_empty(&self) -> bool;
//     fn clear_command(&mut self);
//     fn max_linecol(&self) -> LineCol;
//     fn delete_line(&mut self, at: usize);
//     fn get_full_lines_buffer_window(
//         &self,
//         from: Option<LineCol>,
//         to: Option<LineCol>,
//     ) -> Result<Vec<String>>;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper constants (assuming these are defined in your main code)
    const GROWTH_FACTOR_PERCENT: usize = 150;
    const MIN_GROWTH: usize = 10;

    // Helper struct (assuming this is defined in your main code)

    // Placeholder for BufferPlane (assuming this exists in your code)
    #[derive(Default)]
    struct BufferPlane;

    #[test]
    fn test_new() {
        let gap_buf = GapBuf::new(10);

        assert_eq!(gap_buf.buffer.len(), 10);
        assert_eq!(gap_buf.start, 0);
        assert_eq!(gap_buf.end, 10);
        assert_eq!(gap_buf.current_line, 0);
        assert_eq!(gap_buf.line_starts, vec![0]);

        // Check that buffer is initialized with null chars
        for &ch in &gap_buf.buffer {
            assert_eq!(ch, '\0');
        }
    }

    #[test]
    fn test_insert() {
        let mut gap_buf = GapBuf::new(10);

        // Test inserting regular characters
        gap_buf.insert('a');
        gap_buf.insert('b');
        assert_eq!(gap_buf.start, 2);
        assert_eq!(gap_buf.buffer[0], 'a');
        assert_eq!(gap_buf.buffer[1], 'b');
        assert_eq!(gap_buf.current_line, 0);

        // Test inserting newline
        gap_buf.insert('\n');
        assert_eq!(gap_buf.start, 3);
        assert_eq!(gap_buf.current_line, 1);
        assert_eq!(gap_buf.line_starts, vec![0, 3]);

        // Insert more text after newline
        gap_buf.insert('c');
        gap_buf.insert('d');
        assert_eq!(gap_buf.start, 5);
        assert_eq!(gap_buf.current_line, 1);
    }

    #[test]
    fn test_remove() {
        let mut gap_buf = GapBuf::new(10);

        // Insert some text with newlines
        gap_buf.insert('a');
        gap_buf.insert('\n');
        gap_buf.insert('b');
        gap_buf.insert('c');

        // Remove regular character
        gap_buf.remove();
        assert_eq!(gap_buf.start, 3);
        assert_eq!(gap_buf.current_line, 1);

        // Remove another character
        gap_buf.remove();
        assert_eq!(gap_buf.start, 2);
        assert_eq!(gap_buf.current_line, 1);

        // Remove newline
        gap_buf.remove();
        assert_eq!(gap_buf.start, 1);
        assert_eq!(gap_buf.current_line, 0);
        assert_eq!(gap_buf.line_starts, vec![0]);

        // Remove last character
        gap_buf.remove();
        assert_eq!(gap_buf.start, 0);

        // Try to remove from empty buffer (should not panic)
        gap_buf.remove();
        assert_eq!(gap_buf.start, 0);
    }

    #[test]
    fn test_move_left() {
        let mut gap_buf = GapBuf::new(10);

        // Insert text with newlines
        gap_buf.insert('a');
        gap_buf.insert('\n');
        gap_buf.insert('b');
        gap_buf.insert('c');

        let initial_start = gap_buf.start;
        let initial_end = gap_buf.end;

        // Move left over regular character
        gap_buf.move_left();
        assert_eq!(gap_buf.start, initial_start - 1);
        assert_eq!(gap_buf.end, initial_end - 1);
        assert_eq!(gap_buf.current_line, 1);

        // Move left over another character
        gap_buf.move_left();
        assert_eq!(gap_buf.current_line, 1);

        // Move left over newline
        gap_buf.move_left();
        assert_eq!(gap_buf.current_line, 0);

        // Move left over first character
        gap_buf.move_left();

        // Try to move left at beginning (should not panic or move)
        let start_before = gap_buf.start;
        gap_buf.move_left();
        assert_eq!(gap_buf.start, start_before);
    }

    #[test]
    fn test_move_right() {
        let mut gap_buf = GapBuf::new(10);

        // Insert text
        gap_buf.insert('a');
        gap_buf.insert('\n');
        gap_buf.insert('b');

        // Move cursor to beginning
        while gap_buf.start > 0 {
            gap_buf.move_left();
        }

        let initial_start = gap_buf.start;
        let initial_end = gap_buf.end;

        // Move right over 'a'
        gap_buf.move_right();
        assert_eq!(gap_buf.start, initial_start + 1);
        assert_eq!(gap_buf.end, initial_end + 1);
        assert_eq!(gap_buf.current_line, 0);

        // Move right over newline
        gap_buf.move_right();
        assert_eq!(gap_buf.current_line, 1);

        // Move right over 'b'
        gap_buf.move_right();
        assert_eq!(gap_buf.current_line, 1);

        // Try to move right at end (should not panic)
        gap_buf.move_right();
    }

    #[test]
    fn test_grow() {
        let mut gap_buf = GapBuf::new(4);

        // Fill the buffer to trigger growth
        gap_buf.insert('a');
        gap_buf.insert('b');
        gap_buf.insert('c');
        gap_buf.insert('d');
        gap_buf.insert('e');

        // Buffer should have grown
        assert!(gap_buf.buffer.len() > 4);

        // Content should be preserved
        assert_eq!(gap_buf.buffer[0], 'a');
        assert_eq!(gap_buf.buffer[1], 'b');
        assert_eq!(gap_buf.buffer[2], 'c');
        assert_eq!(gap_buf.buffer[3], 'd');
    }

    #[test]
    fn test_current_linecol() {
        let mut gap_buf = GapBuf::new(20);

        // Test at beginning
        let linecol = gap_buf.current_linecol();
        assert_eq!(linecol, LineCol { line: 0, col: 0 });

        // Insert some text
        gap_buf.insert('h');
        gap_buf.insert('e');
        gap_buf.insert('l');
        gap_buf.insert('l');
        gap_buf.insert('o');

        let linecol = gap_buf.current_linecol();
        assert_eq!(linecol, LineCol { line: 0, col: 5 });

        // Insert newline and more text
        gap_buf.insert('\n');
        gap_buf.insert('w');
        gap_buf.insert('o');
        gap_buf.insert('r');
        gap_buf.insert('l');
        gap_buf.insert('d');

        let linecol = gap_buf.current_linecol();
        assert_eq!(linecol, LineCol { line: 1, col: 5 });

        // Insert another newline
        gap_buf.insert('\n');
        let linecol = gap_buf.current_linecol();
        assert_eq!(linecol, LineCol { line: 2, col: 0 });
    }

    #[test]
    fn test_update_line_starts_after_insertion() {
        let mut gap_buf = GapBuf::new(20);

        // Insert text to create multiple lines
        gap_buf.insert('a');
        gap_buf.insert('\n');
        gap_buf.insert('b');
        gap_buf.insert('\n');
        gap_buf.insert('c');

        // Move cursor back to first line and insert
        gap_buf.move_cursor_to(1, true);
        gap_buf.insert('X');

        // Line starts should be updated correctly
        // Original: "a\nb\nc" -> "aX\nb\nc"
        // line_starts should reflect the shift
        assert!(gap_buf.line_starts[1] > 2); // Second line start moved right
        assert!(gap_buf.line_starts[2] > gap_buf.line_starts[1]); // Third line start moved right
    }

    #[test]
    fn test_update_line_starts_after_deletion() {
        let mut gap_buf = GapBuf::new(20);

        // Insert text to create multiple lines
        gap_buf.insert('a');
        gap_buf.insert('X');
        gap_buf.insert('\n');
        gap_buf.insert('b');
        gap_buf.insert('\n');
        gap_buf.insert('c');

        // Move cursor back and delete
        gap_buf.move_cursor_to(2, true);
        gap_buf.remove(); // Remove 'X'

        // Line starts should be updated correctly
        // The positions of subsequent line starts should decrease
    }

    #[test]
    fn test_move_to_line() {
        let mut gap_buf = GapBuf::new(20);

        // Create multiple lines
        gap_buf.insert('l');
        gap_buf.insert('i');
        gap_buf.insert('n');
        gap_buf.insert('e');
        gap_buf.insert('0');
        gap_buf.insert('\n');
        gap_buf.insert('l');
        gap_buf.insert('i');
        gap_buf.insert('n');
        gap_buf.insert('e');
        gap_buf.insert('1');
        gap_buf.insert('\n');
        gap_buf.insert('l');
        gap_buf.insert('i');
        gap_buf.insert('n');
        gap_buf.insert('e');
        gap_buf.insert('2');

        // Move to line 1
        gap_buf.move_to_line(1);
        assert_eq!(gap_buf.current_line, 1);
        let linecol = gap_buf.current_linecol();
        assert_eq!(linecol.line, 1);
        assert_eq!(linecol.col, 0);

        // Move to line 0
        gap_buf.move_to_line(0);
        assert_eq!(gap_buf.current_line, 0);
        let linecol = gap_buf.current_linecol();
        assert_eq!(linecol.line, 0);
        assert_eq!(linecol.col, 0);
    }

    #[test]
    #[should_panic(expected = "Line 5 does not exist")]
    fn test_move_to_line_panic() {
        let mut gap_buf = GapBuf::new(10);
        gap_buf.insert('a');
        gap_buf.move_to_line(5); // Should panic
    }

    #[test]
    fn test_move_cursor_to() {
        let mut gap_buf = GapBuf::new(20);

        // Insert some text
        gap_buf.insert('h');
        gap_buf.insert('e');
        gap_buf.insert('l');
        gap_buf.insert('l');
        gap_buf.insert('o');
        gap_buf.insert('\n');
        gap_buf.insert('w');
        gap_buf.insert('o');
        gap_buf.insert('r');
        gap_buf.insert('l');
        gap_buf.insert('d');

        // Move cursor to position 2 (middle of "hello")
        gap_buf.move_cursor_to(2, true);
        assert_eq!(gap_buf.start, 2);
        let linecol = gap_buf.current_linecol();
        assert_eq!(linecol, LineCol { line: 0, col: 2 });

        // Move cursor to position 7 (middle of "world")
        gap_buf.move_cursor_to(7, true);
        assert_eq!(gap_buf.start, 7);
        let linecol = gap_buf.current_linecol();
        assert_eq!(linecol, LineCol { line: 1, col: 1 });

        // Move cursor back to beginning
        gap_buf.move_cursor_to(0, true);
        assert_eq!(gap_buf.start, 0);
        let linecol = gap_buf.current_linecol();
        assert_eq!(linecol, LineCol { line: 0, col: 0 });
    }

    #[test]
    fn test_comprehensive_newline_handling() {
        let mut gap_buf = GapBuf::new(30);

        // Test complex newline scenarios
        gap_buf.insert('F');
        gap_buf.insert('i');
        gap_buf.insert('r');
        gap_buf.insert('s');
        gap_buf.insert('t');
        gap_buf.insert('\n');
        gap_buf.insert('S');
        gap_buf.insert('e');
        gap_buf.insert('c');
        gap_buf.insert('o');
        gap_buf.insert('n');
        gap_buf.insert('d');
        gap_buf.insert('\n');
        gap_buf.insert('\n'); // Empty line
        gap_buf.insert('F');
        gap_buf.insert('o');
        gap_buf.insert('u');
        gap_buf.insert('r');
        gap_buf.insert('t');
        gap_buf.insert('h');

        // Should have 4 lines (0, 1, 2, 3)
        assert_eq!(gap_buf.line_starts.len(), 4);
        assert_eq!(gap_buf.current_line, 3);

        let linecol = gap_buf.current_linecol();
        assert_eq!(linecol, LineCol { line: 3, col: 6 });

        // Test moving between lines with newlines
        gap_buf.move_to_line(1);
        let linecol = gap_buf.current_linecol();
        assert_eq!(linecol, LineCol { line: 1, col: 0 });

        gap_buf.move_to_line(2); // Empty line
        let linecol = gap_buf.current_linecol();
        assert_eq!(linecol, LineCol { line: 2, col: 0 });
    }
}
