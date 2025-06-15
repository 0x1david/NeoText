mod gapbuf;
mod rope;
mod stringvec;

use crate::Result;
use crate::{modals::Modal, LineCol, Pattern};
pub use stringvec::StringVec;

#[derive(Default, Debug, Clone, Copy)]
enum BufferPlane {
    #[default]
    Normal,
    Terminal,
    Command,
}

/// Trait defining the interface for a text buffer
#[allow(clippy::module_name_repetitions)]
pub trait TextBuffer {
    fn set_plane(&mut self, modal: &Modal);
    fn insert_newline(&mut self, at: LineCol) -> LineCol;
    fn get_byte_offset(&self, to: LineCol) -> usize;
    /// Insert a single symbol at specified position
    fn insert(&mut self, at: LineCol, insertable: char) -> Result<LineCol>;

    /// Insert text at the specified position
    fn insert_text(
        &mut self,
        at: LineCol,
        text: impl Into<String>,
        newline: bool,
    ) -> Result<LineCol>;

    /// Delete text in the specified range
    fn delete_selection(&mut self, from: LineCol, to: LineCol) -> Result<LineCol>;

    /// Delete the symbol at the specified position
    fn delete(&mut self, at: LineCol) -> Result<LineCol>;

    /// Replace text in the specified range with new text
    fn replace(&mut self, from: LineCol, to: LineCol, text: &str) -> Result<()>;

    /// Get the text in the specified range
    fn get_text(&self, from: LineCol, to: LineCol) -> Result<String>;

    /// Get the length of the entire buffer
    fn len(&self) -> usize;

    /// Check if the buffer is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the number of lines in the buffer
    fn line_count(&self) -> usize;

    /// Get a single continuous vec of bytes containing the entire text
    fn get_coalesced_bytes(&self) -> Vec<u8>;

    /// Get the contents of a specific line
    fn line(&self, line_number: usize) -> Result<&str>;

    /// Find the next occurrence of a Pattern
    fn find(&self, query: impl Pattern, at: LineCol) -> Result<LineCol>;

    /// Find the previous occurrence of a Pattern
    fn rfind(&self, query: impl Pattern, at: LineCol) -> Result<LineCol>;

    /// Undo the last operation
    fn undo(&mut self, at: LineCol) -> Result<LineCol>;

    /// Redo the last undone operation
    fn redo(&mut self, at: LineCol) -> Result<LineCol>;

    /// Get the entire text for the current buffer
    fn get_entire_text(&self) -> &[String];
    /// Get the entire text for the normal buffer
    fn get_normal_text(&self) -> &[String];

    /// Get partial window to the normal buffer, ranging from -> to
    fn get_buffer_window(&self, from: Option<LineCol>, to: Option<LineCol>) -> Result<Vec<String>>;

    /// Get the entire text for the terminal buffer
    fn get_terminal_text(&self) -> &str;
    /// Get the entire text for the command buffer
    fn get_command_text(&self) -> &[String];
    /// Get the entire text for the command buffer
    fn replace_command_text(&mut self, new: impl Into<String>);

    /// Get maximum line bound for the current buffer
    fn max_line(&self) -> usize;
    /// Get maximum column bound for the current buffer
    fn max_col(&self, at: LineCol) -> usize;
    fn is_command_empty(&self) -> bool;
    fn clear_command(&mut self);
    fn max_linecol(&self) -> LineCol;
    fn delete_line(&mut self, at: usize);
    fn get_full_lines_buffer_window(
        &self,
        from: Option<LineCol>,
        to: Option<LineCol>,
    ) -> Result<Vec<String>>;
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::stringvec::{Stack, StringVec};
    use super::*;
    /// "First line"
    /// "Second line"
    /// "Third line"
    fn new_test_buffer() -> StringVec {
        StringVec {
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
    fn new_test_buffer_find() -> StringVec {
        StringVec {
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
            buf.rfind("line", LineCol { line: 2, col: 0 }).unwrap(),
            LineCol { line: 1, col: 7 }
        );
    }

    #[test]
    fn test_rfind_not_including_start() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.rfind("line", LineCol { line: 1, col: 7 }).unwrap(),
            LineCol { line: 0, col: 6 }
        );
    }

    #[test]
    fn test_rfind_across_lines() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.rfind("First", LineCol { line: 2, col: 0 }).unwrap(),
            LineCol { line: 0, col: 0 }
        );
    }

    #[test]
    fn test_rfind_at_end_of_buffer() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.rfind("too", LineCol { line: 2, col: 22 }).unwrap(),
            LineCol { line: 2, col: 19 }
        );
    }
    #[test]
    fn test_find_basic() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("line", LineCol { line: 0, col: 0 }).unwrap(),
            LineCol { line: 0, col: 6 }
        );
    }

    #[test]
    fn test_find_from_middle() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("text", LineCol { line: 0, col: 10 }).unwrap(),
            LineCol { line: 0, col: 21 }
        );
    }

    #[test]
    fn test_find_across_lines() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("Second", LineCol { line: 0, col: 22 }).unwrap(),
            LineCol { line: 1, col: 0 }
        );
    }

    #[test]
    fn test_find_at_start_of_line() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("Third", LineCol { line: 2, col: 0 }).unwrap(),
            LineCol { line: 2, col: 0 }
        );
    }

    #[test]
    fn test_find_at_end_of_line() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("text", LineCol { line: 1, col: 0 }).unwrap(),
            LineCol { line: 1, col: 21 }
        );
    }

    #[test]
    fn test_find_exact_position() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("Second", LineCol { line: 1, col: 0 }).unwrap(),
            LineCol { line: 1, col: 0 }
        );
    }

    #[test]
    fn test_find_multiple_occurrences() {
        let buf = new_test_buffer_find();
        assert_eq!(
            buf.find("line", LineCol { line: 0, col: 10 }).unwrap(),
            LineCol { line: 1, col: 7 }
        );
    }

    #[test]
    fn test_find_from_empty_line() {
        let mut buf = new_test_buffer_find();
        buf.text.insert(1, String::new());
        assert_eq!(
            buf.find("Third", LineCol { line: 1, col: 0 }).unwrap(),
            LineCol { line: 3, col: 0 }
        );
    }
    /// "First line"
    /// "Second line"
    /// "Third line"
    /// "Fourth line"
    fn new_test_buffer_get() -> StringVec {
        StringVec {
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
    fn test_get_text_single_line() -> Result<()> {
        let buffer = new_test_buffer_get();
        assert_eq!(
            buffer.get_text(LineCol { line: 0, col: 0 }, LineCol { line: 0, col: 5 })?,
            "First".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_get_text_multiple_lines() -> Result<()> {
        let buffer = new_test_buffer_get();
        assert_eq!(
            buffer.get_text(LineCol { line: 0, col: 6 }, LineCol { line: 2, col: 5 })?,
            "line\nSecond line\nThird".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_get_text_entire_line() -> Result<()> {
        let buffer = new_test_buffer_get();
        assert_eq!(
            buffer.get_text(LineCol { line: 1, col: 0 }, LineCol { line: 2, col: 0 })?,
            "Second line\n".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_get_text_across_all_lines() -> Result<()> {
        let buffer = new_test_buffer_get();
        assert_eq!(
            buffer.get_text(LineCol { line: 0, col: 0 }, LineCol { line: 3, col: 11 })?,
            "First line\nSecond line\nThird line\nFourth line".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_get_text_empty_range() -> Result<()> {
        let buffer = new_test_buffer_get();
        assert_eq!(
            buffer.get_text(LineCol { line: 1, col: 5 }, LineCol { line: 1, col: 5 })?,
            "".to_string()
        );
        Ok(())
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
        let mut buffer = StringVec::default();

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
        let mut buffer = StringVec::default();

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
    fn test_delete_across_buffers() {
        let mut buffer = StringVec::default();

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

    #[test]
    fn test_find_first_uppercase() {
        let buf = new_test_buffer_find();
        let pattern = |c: char| c.is_uppercase();
        assert_eq!(
            buf.find(pattern, LineCol { line: 0, col: 1 }).unwrap(),
            LineCol { line: 1, col: 0 }
        );
    }

    #[test]
    fn test_find_first_whitespace() {
        let buf = new_test_buffer_find();
        let pattern = char::is_whitespace;
        assert_eq!(
            buf.find(pattern, LineCol { line: 0, col: 0 }).unwrap(),
            LineCol { line: 0, col: 5 }
        );
    }

    #[test]
    fn test_find_specific_char() {
        let buf = new_test_buffer_find();
        let pattern = |c: char| c == 'e';
        assert_eq!(
            buf.find(pattern, LineCol { line: 0, col: 0 }).unwrap(),
            LineCol { line: 0, col: 9 }
        );
    }

    #[test]
    fn test_find_no_match() {
        let buf = new_test_buffer_find();
        let pattern = |c: char| c.is_ascii_punctuation() && c != ',';
        assert!(buf.find(pattern, LineCol { line: 0, col: 0 }).is_err());
    }

    #[test]
    fn test_find_complex_condition() {
        let buf = new_test_buffer_find();
        let pattern = |c: char| c.is_lowercase() && "aeiou".contains(c);
        assert_eq!(
            buf.find(pattern, LineCol { line: 0, col: 0 }).unwrap(),
            LineCol { line: 0, col: 1 } // Should find 'i' in "First"
        );
    }
    #[test]
    fn test_get_partial_buffer_full_range() {
        let buf = new_test_buffer_find();
        let result = buf.get_buffer_window(None, None).unwrap();
        assert_eq!(result, buf.text);
    }

    #[test]
    fn test_get_partial_buffer_single_line() {
        let buf = new_test_buffer_find();
        let result = buf
            .get_buffer_window(
                Some(LineCol { line: 0, col: 6 }),
                Some(LineCol { line: 0, col: 10 }),
            )
            .unwrap();
        assert_eq!(result, vec!["line"]);
    }

    #[test]
    fn test_get_partial_buffer_multiple_lines() {
        let buf = new_test_buffer_find();
        let result = buf
            .get_buffer_window(
                Some(LineCol { line: 0, col: 6 }),
                Some(LineCol { line: 2, col: 5 }),
            )
            .unwrap();
        assert_eq!(
            result,
            vec!["line with some text", "Second line also has text", "Third"]
        );
    }

    #[test]
    fn test_get_partial_buffer_from_middle_to_end() {
        let buf = new_test_buffer_find();
        let result = buf
            .get_buffer_window(Some(LineCol { line: 1, col: 7 }), None)
            .unwrap();
        assert_eq!(result, vec!["line also has text", "Third line is here too"]);
    }

    #[test]
    fn test_get_partial_buffer_from_start_to_middle() {
        let buf = new_test_buffer_find();
        let result = buf
            .get_buffer_window(None, Some(LineCol { line: 1, col: 7 }))
            .unwrap();
        assert_eq!(result, vec!["First line with some text", "Second "]);
    }

    #[test]
    fn test_get_partial_buffer_invalid_range() {
        let buf = new_test_buffer_find();
        let result = buf.get_buffer_window(
            Some(LineCol { line: 2, col: 0 }),
            Some(LineCol { line: 1, col: 0 }),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_get_partial_buffer_empty_range() {
        let buf = new_test_buffer_find();
        let result = buf
            .get_buffer_window(
                Some(LineCol { line: 1, col: 5 }),
                Some(LineCol { line: 1, col: 5 }),
            )
            .unwrap();
        assert_eq!(result, vec![""]);
    }

    #[test]
    fn test_get_partial_buffer_last_line() {
        let buf = new_test_buffer_find();
        let result = buf
            .get_buffer_window(Some(LineCol { line: 2, col: 6 }), None)
            .unwrap();
        assert_eq!(result, vec!["line is here too"]);
    }
}
