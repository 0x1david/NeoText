use std::collections::VecDeque;

use super::{BufferPlane, TextBuffer};
use crate::modals::Modal;
use crate::{Error, LineCol, Pattern, Result};
const GROWTH_FACTOR_PERCENT: usize = 150; // If value is 150, new buffer will be 150% size of the last
const MIN_GROWTH: usize = 512;

struct GapBuf {
    buffer: Vec<char>,
    start: usize,
    end: usize,
}

impl GapBuf {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec!['\0'; capacity],
            start: 0,
            end: capacity,
        }
    }
    fn insert(&mut self, ch: char) {
        if self.start == self.end {
            self.grow()
        }
        self.buffer[self.start] = ch;
        self.start += 1
    }

    fn remove(&mut self) {
        self.start = self.start.saturating_sub(1);
    }

    fn move_left(&mut self) {
        if self.start > 0 {
            self.start -= 1;
            self.end -= 1;
            self.buffer[self.end] = self.buffer[self.start]
        }
    }

    fn move_right(&mut self) {
        if self.end < self.buffer.len() {
            self.buffer[self.start] = self.buffer[self.end];
            self.start += 1;
            self.end += 1;
        }
    }
    // Grow the buffer, only accounts for growth when gap == 0
    fn grow(&mut self) {
        let old_capacity = self.buffer.len();
        let mut new_capacity = (old_capacity / 2) * GROWTH_FACTOR_PERCENT;
        new_capacity = new_capacity.max(MIN_GROWTH);
        let mut new_buf = vec!['\0'; new_capacity];
        new_buf[..self.start].copy_from_slice(&self.buffer[..self.start]);

        let gap_size = new_capacity - old_capacity;
        new_buf[self.start + gap_size..].copy_from_slice(&self.buffer[self.end..]);

        self.buffer = new_buf;
        self.end = self.start + gap_size - 1
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let buf = GapBuf::new(10);
        assert_eq!(buf.start, 0);
        assert_eq!(buf.end, 10);
        assert_eq!(buf.buffer.len(), 10);
        // All should be null chars initially
        assert!(buf.buffer.iter().all(|&c| c == '\0'));
    }

    #[test]
    fn test_insert() {
        let mut buf = GapBuf::new(5);
        buf.insert('a');
        buf.insert('b');

        assert_eq!(buf.start, 2);
        assert_eq!(buf.end, 5);
        assert_eq!(buf.buffer[0], 'a');
        assert_eq!(buf.buffer[1], 'b');
    }

    #[test]
    fn test_remove() {
        let mut buf = GapBuf::new(5);
        buf.insert('a');
        buf.insert('b');
        buf.remove();

        assert_eq!(buf.start, 1);
        assert_eq!(buf.buffer[0], 'a');
        // 'b' is still there but cursor moved back
    }

    #[test]
    fn test_move_right() {
        let mut buf = GapBuf::new(10);
        // Insert some text: "abc"
        buf.insert('a');
        buf.insert('b');
        buf.insert('c');
        // Now cursor is after 'c'

        // Move cursor left twice to be between 'a' and 'b'
        buf.move_left();
        buf.move_left();

        // Now move right - should move 'b' from right side to left side
        buf.move_right();

        assert_eq!(buf.buffer[0], 'a');
        assert_eq!(buf.buffer[1], 'b');
    }

    #[test]
    fn test_move_left() {
        let mut buf = GapBuf::new(10);
        buf.insert('a');
        buf.insert('b');
        buf.insert('c');

        // Move cursor left - should move 'c' from left side to right side
        buf.move_left();

        assert_eq!(buf.start, 2);
        assert_eq!(buf.buffer[0], 'a');
    }

    #[test]
    fn test_grow() {
        let mut buf = GapBuf::new(2);
        buf.insert('a');
        buf.insert('b');
        // Gap is now full (start == end)

        // This should trigger grow()
        buf.insert('c');

        // Buffer should be larger now
        assert!(buf.buffer.len() > 2);
        assert_eq!(buf.buffer[0], 'a');
        assert_eq!(buf.buffer[1], 'b');
        assert_eq!(buf.buffer[2], 'c');
    }

    #[test]
    fn test_complex_editing() {
        let mut buf = GapBuf::new(10);

        // Type "hello"
        "hello".chars().for_each(|c| buf.insert(c));

        // Move cursor to beginning
        for _ in 0..5 {
            buf.move_left();
        }

        // Insert "Hi " at beginning
        "Hi ".chars().for_each(|c| buf.insert(c));

        // Should have "Hi hello" with cursor between "Hi " and "hello"
        assert_eq!(buf.buffer[0], 'H');
        assert_eq!(buf.buffer[1], 'i');
        assert_eq!(buf.buffer[2], ' ');
    }

    #[test]
    fn test_edge_cases() {
        let mut buf = GapBuf::new(5);

        // Remove from empty buffer
        buf.remove();
        assert_eq!(buf.start, 0); // Should stay at 0

        // Move left at beginning
        buf.move_left();
        assert_eq!(buf.start, 0); // Should stay at 0

        // Insert at capacity and test grow
        for i in 0..5 {
            buf.insert(char::from_digit(i, 10).unwrap());
        }
        // Buffer should be full, next insert should grow
        buf.insert('x');
        assert!(buf.buffer.len() > 5);
    }
}
