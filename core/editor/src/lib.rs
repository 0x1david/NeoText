#![allow(clippy::cast_possible_wrap)]
use buffer::TextBuffer;
use crossterm::{
    execute,
    terminal::{DisableLineWrap, EnterAlternateScreen},
};

pub(crate) mod buffer;
pub(crate) mod copy_register;
pub(crate) mod cursor;
pub mod editor;
pub(crate) mod error;
pub(crate) mod modals;
pub(crate) mod searcher;
pub(crate) mod utils;
pub(crate) mod view_window;
#[macro_use]
pub mod bars;

use cursor::LineCol;
use editor::EditorInner;
pub use error::{Error, Result};
use modals::normal::SCROLL_JUMP_DISTANCE;
pub use modals::Modal;
use searcher::Pattern;

/// Initializes the terminal for the editor.
pub fn initialize_terminal() -> std::io::Result<()> {
    execute!(std::io::stdout(), EnterAlternateScreen, DisableLineWrap)
}

pub struct Editor<Buff: TextBuffer> {
    inner: EditorInner<Buff>,
}

impl<Buff: TextBuffer> Editor<Buff> {
    /// Creates a new `Editor` instance with the given buffer and initial launch state.
    ///
    /// # Arguments
    /// * `buffer` - The text buffer to be edited.
    /// * `launch_without_target` - Whether the editor is launched without a target file.
    pub fn new(buffer: Buff, launch_without_target: bool) -> Self {
        Self {
            inner: EditorInner::new(buffer, launch_without_target),
        }
    }

    /// Moves the cursor up one line.
    pub fn bump_up(&mut self) {
        self.inner.cursor.bump_up();
    }

    /// Moves the cursor down one line.
    pub fn bump_down(&mut self) {
        self.inner.cursor.bump_down();
    }

    /// Moves the cursor one character to the left.
    pub fn bump_left(&mut self) {
        self.inner.cursor.bump_left();
    }

    /// Moves the cursor one character to the right.
    pub fn bump_right(&mut self) {
        self.inner.cursor.bump_right();
    }

    /// Jumps the cursor up by a preset number of lines.
    pub fn jump_up(&mut self) {
        self.inner.cursor.jump_up(SCROLL_JUMP_DISTANCE);
    }

    /// Jumps the cursor down by a preset number of lines.
    pub fn jump_down(&mut self) {
        self.inner
            .cursor
            .jump_down(SCROLL_JUMP_DISTANCE, self.inner.buffer.max_line());
    }

    /// Sets the cursor to a specific line and column.
    ///
    /// # Arguments
    /// * `to` - The `LineCol` position to move the cursor to.
    pub fn set_cursor(&mut self, to: LineCol) {
        self.inner.cursor.set_line(to.line);
        self.inner.cursor.set_col(to.col);
    }

    /// Jumps the cursor after the next occurrence of the specified character.
    ///
    /// # Arguments
    /// * `c` - The character to search for.
    /// * `repeat` - Optional number of times to repeat the search.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn jump_after_letter(&mut self, c: char, repeat: Option<u32>) -> Result<()> {
        self.inner.find_previous_char(c, repeat)
    }

    /// Moves the cursor to the end of the current line.
    pub fn jump_eol(&mut self) {
        self.inner.move_to_end_of_line()
    }

    /// Changes the editor's current mode.
    ///
    /// # Arguments
    /// * `mode` - The new `Modal` to set.
    pub fn change_mode(&mut self, mode: Modal) {
        self.inner.set_mode(mode);
    }

    /// Jumps the cursor back to the previous occurrence of the specified character.
    ///
    /// # Arguments
    /// * `c` - The character to search for.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn jump_back_to(&mut self, c: char) -> Result<()> {
        self.inner.move_back_to_char(c)
    }

    /// Jumps the cursor to the next alphanumeric character after whitespace.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn jump_to_next_alnum_post_whitespace(&mut self) -> Result<()> {
        self.inner.move_to_next_word_after_whitespace()
    }

    /// Jumps the cursor to the next non-alphanumeric character after whitespace.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn jump_to_next_symbol_post_whitespace(&mut self) -> Result<()> {
        self.inner.move_to_next_non_alphanumeric()
    }

    /// Jumps the cursor to the first non-whitespace character of the current line.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn jump_to_start_of_line(&mut self) -> Result<()> {
        self.inner.move_to_first_non_whitespace_col()
    }

    /// Moves the cursor to the end of the file.
    pub fn jump_eof(&mut self) {
        let lc = LineCol {
            line: self.inner.buffer.max_line(),
            col: self.inner.cursor.col(),
        };
        self.set_cursor(lc)
    }

    /// Moves the cursor to the start of the file.
    pub fn jump_sof(&mut self) {
        let lc = LineCol {
            line: 0,
            col: self.inner.cursor.col(),
        };
        self.set_cursor(lc)
    }

    /// Moves the cursor to the end of the line and switches to insert mode.
    pub fn insert_mode_eol(&mut self) {
        self.jump_eol();
        self.inner.set_mode(Modal::Insert)
    }

    /// Finds the next occurrence of the given pattern and moves the cursor to it.
    ///
    /// # Arguments
    /// * `pattern` - The pattern to search for.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn find(&mut self, pattern: impl Pattern) -> Result<()> {
        let lc_opt = pattern.find_pattern(&self.inner.get_selected()?);
        if let Some(lc) = lc_opt {
            self.inner.go(lc);
        }
        Ok(())
    }

    /// Finds the previous occurrence of the given pattern and moves the cursor to it.
    ///
    /// # Arguments
    /// * `pattern` - The pattern to search for.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn rfind(&mut self, pattern: impl Pattern) -> Result<()> {
        let lc_opt = pattern.rfind_pattern(&self.inner.get_selected()?);
        if let Some(lc) = lc_opt {
            self.inner.go(lc);
        }
        Ok(())
    }

    /// Finds the next occurrence of the given character and moves the cursor to it.
    ///
    /// # Arguments
    /// * `c` - The character to search for.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn find_char(&mut self, c: char) -> Result<()> {
        let lc_opt = c.find_pattern(self.inner.buffer.get_normal_text());
        if let Some(lc) = lc_opt {
            self.inner.go(lc);
        }
        Ok(())
    }

    /// Finds the previous occurrence of the given character and moves the cursor to it.
    ///
    /// # Arguments
    /// * `c` - The character to search for.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn rfind_char(&mut self, c: char) -> Result<()> {
        let lc_opt = c.rfind_pattern(&self.inner.get_selected()?);
        if let Some(lc) = lc_opt {
            self.inner.go(lc);
        }
        Ok(())
    }

    /// Replaces the character under the cursor with the given character.
    ///
    /// # Arguments
    /// * `c` - The character to replace with.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn replace(&mut self, c: char) -> Result<()> {
        self.inner.replace_under_cursor(c)
    }

    /// Inserts the given character at the cursor position.
    ///
    /// # Arguments
    /// * `c` - The character to insert.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn insert_at_cursor(&mut self, c: char) -> Result<()> {
        let new_lc = self.inner.buffer.insert(self.inner.pos(), c)?;
        self.inner.go(new_lc);
        Ok(())
    }

    /// Deletes the character before the cursor.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn delete_before(&mut self) -> Result<()> {
        let mut lc = self.inner.pos();
        lc.col -= 1;
        let new_lc = self.inner.buffer.delete(lc)?;
        self.inner.go(new_lc);
        Ok(())
    }

    /// Deletes the character under the cursor.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn delete_under(&mut self) -> Result<()> {
        let new_lc = self.inner.buffer.delete(self.inner.pos())?;
        self.inner.go(new_lc);
        Ok(())
    }

    /// Yanks (copies) the current selection.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn yank(&mut self) -> Result<()> {
        self.inner.yank()?;
        Ok(())
    }

    /// Pastes the content of the specified register at the cursor position.
    ///
    /// # Arguments
    /// * `register` - The register to paste from.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn paste(&mut self, register: char) -> Result<()> {
        self.inner.paste_register_content(Some(register), false)
    }

    /// Pastes the content of the specified register on a new line.
    ///
    /// # Arguments
    /// * `register` - The register to paste from.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn paste_newline(&mut self, register: char) -> Result<()> {
        self.inner.paste_register_content(Some(register), true)
    }

    /// Pastes the content of the specified register above the current line.
    ///
    /// # Arguments
    /// * `register` - The register to paste from.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn paste_above(&mut self, register: char) -> Result<()> {
        unimplemented!();
    }

    /// Fetches an entry from the appropriate history based on the current mode.
    ///
    /// # Arguments
    /// * `nth` - The index of the history entry to fetch.
    ///
    /// # Returns
    /// A `Result` containing a reference to the fetched history entry, or an error.
    pub fn fetch_from_history(&mut self, nth: u8) -> Result<&String> {
        match self.inner.mode {
            Modal::Find(modals::FindMode::Forwards) => self
                .inner
                .forwards_history
                .get(nth.into())
                .ok_or(Error::NoHistoryContent),
            Modal::Find(modals::FindMode::Backwards) => self
                .inner
                .backwards_history
                .get(nth.into())
                .ok_or(Error::NoHistoryContent),
            Modal::Command => self
                .inner
                .command_history
                .get(nth.into())
                .ok_or(Error::NoHistoryContent),
            _ => Err(Error::InvalidMode),
        }
    }

    /// Executes a command.
    ///
    /// # Arguments
    /// * `command` - The command to execute.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    pub fn execute_command(&mut self, command: Command) -> Result<()> {
        unimplemented!()
    }

    /// Undoes the last action.
    pub fn undo(&mut self) {
        unimplemented!()
    }

    /// Redoes the last undone action.
    pub fn redo(&mut self) {
        unimplemented!()
    }
    pub fn get_under_cursor(&self) -> Option<char> {
        self.inner.get_under_cursor()
    }
}

pub enum Command {}

#[cfg(test)]
mod tests {
    use crate::modals::FindMode;
    use crate::Editor;
    use crate::Modal;
    use crate::TextBuffer;
    use crate::buffer::VecBuffer;

    fn create_test_editor() -> Editor<VecBuffer> {
        let content = vec![
            "The quick brown fox jumps over the lazy dog.".to_string(),
            "Pack my box with five dozen liquor jugs.".to_string(),
            "How vexingly quick daft zebras jump!".to_string(),
            "The five boxing wizards jump quickly.".to_string(),
        ];
        let buffer = VecBuffer::new(content);
        Editor::new(buffer, true)
    }

    #[test]
    fn test_bump_down() {
        let mut editor = create_test_editor();
        editor.bump_down();
        assert_eq!(
            editor.inner.pos().line,
            1,
            "Cursor should move down one line"
        );
    }

    #[test]
    fn test_bump_right() {
        let mut editor = create_test_editor();
        editor.bump_right();
        assert_eq!(
            editor.inner.pos().col,
            1,
            "Cursor should move right one column"
        );
    }

    #[test]
    fn test_bump_up() {
        let mut editor = create_test_editor();
        editor.bump_down();
        editor.bump_up();
        assert_eq!(editor.inner.pos().line, 0, "Cursor should move up one line");
    }

    #[test]
    fn test_bump_left() {
        let mut editor = create_test_editor();
        editor.bump_right();
        editor.bump_left();
        assert_eq!(
            editor.inner.pos().col,
            0,
            "Cursor should move left one column"
        );
    }

    #[test]
    fn test_jump_down() {
        let mut editor = create_test_editor();
        editor.jump_down();
        assert!(
            editor.inner.pos().line > 1,
            "Cursor should jump down multiple lines"
        );
    }

    #[test]
    fn test_jump_up() {
        let mut editor = create_test_editor();
        editor.jump_down();
        editor.jump_up();
        assert_eq!(
            editor.inner.pos().line,
            0,
            "Cursor should jump back to the first line"
        );
    }

    #[test]
    fn test_jump_eol() {
        let mut editor = create_test_editor();
        editor.jump_eol();
        assert!(
            editor.inner.pos().col > 0,
            "Cursor should be at the end of the line"
        );
    }

    #[test]
    fn test_jump_to_start_of_line() {
        let mut editor = create_test_editor();
        editor.jump_eol();
        editor.jump_to_start_of_line().unwrap();
        assert_eq!(
            editor.inner.pos().col,
            0,
            "Cursor should be at the start of the line"
        );
    }

    #[test]
    fn test_jump_eof() {
        let mut editor = create_test_editor();
        editor.jump_eof();
        assert!(
            editor.inner.pos().line > 0,
            "Cursor should be at the last line"
        );
    }

    #[test]
    fn test_jump_sof() {
        let mut editor = create_test_editor();
        editor.jump_eof();
        editor.jump_sof();
        assert_eq!(
            editor.inner.pos().line,
            0,
            "Cursor should be at the first line"
        );
    }

    #[test]
    fn test_find_char() {
        let mut editor = create_test_editor();
        editor.find_char('e').unwrap();
        assert!(
            editor.inner.pos().col == 2,
            "Cursor should move to second position, instead at col {}",
            editor.inner.pos().col
        );
    }

    #[test]
    fn test_rfind_char() {
        let mut editor = create_test_editor();
        editor.rfind_char('e').unwrap();
        assert!(
            editor.inner.pos().col < editor.inner.buffer.max_col(editor.inner.pos()),
            "Cursor should move to the last 'e'"
        );
    }

    #[test]
    fn test_jump_to_next_alnum_post_whitespace() {
        let mut editor = create_test_editor();
        editor.jump_to_next_alnum_post_whitespace().unwrap();
        assert!(
            editor.inner.pos().col > 0,
            "Cursor should move to the next word"
        );
    }

    #[test]
    fn test_jump_to_next_symbol_post_whitespace() {
        let mut editor = create_test_editor();
        editor.jump_to_next_symbol_post_whitespace().unwrap();
        assert!(
            editor.inner.pos().col > 0,
            "Cursor should move to the next symbol"
        );
    }

    #[test]
    fn test_replace() {
        let mut editor = create_test_editor();
        _ = editor.inner.get_under_cursor().unwrap();
        editor.replace('X').unwrap();
        assert_eq!(
            editor.inner.get_under_cursor().unwrap(),
            'X',
            "Character should be replaced with 'X'"
        );
    }

    #[test]
    fn test_delete_under() {
        let mut editor = create_test_editor();
        let original_char = editor.inner.get_under_cursor().unwrap();
        editor.delete_under().unwrap();
        assert_ne!(
            editor.inner.get_under_cursor().unwrap(),
            original_char,
            "Character should be deleted"
        );
    }

    #[test]
    fn test_insert_at_cursor() {
        let mut editor = create_test_editor();
        _ = editor.insert_at_cursor('Y');
        assert_eq!(
            editor.inner.get_under_cursor().unwrap(),
            'Y',
            "Character 'Y' should be inserted"
        );
    }

    #[test]
    fn test_yank_and_paste() {
        let mut editor = create_test_editor();
        editor.change_mode(Modal::Visual);
        editor.bump_right();
        editor.yank().unwrap();
        let original_pos = editor.inner.pos();
        editor.paste('0').unwrap();
        assert_ne!(
            editor.inner.pos(),
            original_pos,
            "Cursor should move after pasting"
        );
    }

    #[test]
    fn test_change_mode() {
        let mut editor = create_test_editor();
        editor.change_mode(Modal::Insert);
        assert_eq!(
            editor.inner.mode,
            Modal::Insert,
            "Editor should be in Insert mode"
        );
        editor.change_mode(Modal::Normal);
        assert_eq!(
            editor.inner.mode,
            Modal::Normal,
            "Editor should be in Normal mode"
        );
    }

    #[test]
    fn test_fetch_from_history() {
        let mut editor = create_test_editor();
        editor.change_mode(Modal::Find(FindMode::Forwards));
        let history_entry = editor.fetch_from_history(0);
        assert!(
            history_entry.is_ok(),
            "Should be able to fetch from history"
        );
    }
}

// TODO: Implement the following features:
// - Syntax Highlighting
// - Regex Command Processing
// - Undo and Redo
// - Terminal Mode
// - Macros
// - Marks
// - Improve buffer data structure
// - Telescope integration
// - Configuration parsing and configurable controller
// - LSP Integration
// - Programmable Extensions
// - Screen Splits
// - File Commands
// - Different cursor visuals

// TODO: Fix known bugs:
// - Constant crashing
//
