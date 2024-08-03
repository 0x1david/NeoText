#![allow(clippy::match_wild_err_arm)]
use crate::bars::{
    draw_bar, get_debug_messages, get_info_bar_content, get_notif_bar_content, COMMAND_BAR,
    INFO_BAR, INFO_BAR_Y_LOCATION, NOTIFICATION_BAR, NOTIFICATION_BAR_Y_LOCATION,
};
use crate::buffer::TextBuffer;
use crate::cursor::{Cursor, LineCol};
use crate::modals::{FindMode, Modal};
use crate::notif_bar;
use crate::{Error, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
};

// use crate::modal::Modal;
use std::io::stdout;
use std::process::exit;

/// The main editor is used as the main API for all commands
pub struct Editor<Buff: TextBuffer> {
    /// In the first implementation I will start with Vec, for simplicity, fairly early to the dev
    /// process a better data structure will have to be found and vec replaced;
    pub(crate) cursor: Cursor,
    pub(crate) buffer: Buff,
    pub(crate) mode: Modal,
}

impl<Buff: TextBuffer> Editor<Buff> {
    /// Creates a new instance of `MainEditor`.
    ///
    /// # Arguments
    /// * `buffer` - The text buffer to be edited.
    ///
    /// # Returns
    /// A new `MainEditor` instance initialized with the given buffer and default cursor position.
    pub fn new(buffer: Buff) -> Self {
        Self {
            buffer,
            cursor: Cursor::default(),
            mode: Modal::default(),
        }
    }

    /// Applies a cursor movement if it results in a valid position within the buffer bounds.
    ///
    /// # Arguments
    /// * `movement` - A function that takes a mutable reference to a Cursor and moves it.
    ///
    /// # Behavior
    /// 1. Stores the original cursor position.
    /// 3. Applies the movement to the cursor.
    /// 3. If the new line exceeds the buffer's max line, reverts to the original position.
    /// 4. If the new column exceeds the max column for that line, adjusts to the max column.
    pub fn if_within_bounds<F>(&mut self, movement: F)
    where
        F: FnOnce(&mut Cursor),
    {
        let original_pos = self.pos();
        movement(&mut self.cursor);
        if self.pos().line > self.buffer.max_line() {
            self.cursor.pos = original_pos;
            return;
        }
        let new_pos = self.pos();
        let max_col = self.buffer.max_col(new_pos);
        if new_pos.col > max_col {
            self.cursor.set_col(max_col);
        }
    }

    #[inline]
    pub(crate) const fn pos(&self) -> LineCol {
        self.cursor.pos
    }
    const fn last_normal_pos(&self) -> LineCol {
        self.cursor.last_text_mode_pos
    }
    pub(crate) fn set_mode(&mut self, modal: Modal) {
        self.cursor.mod_change(&modal);
        self.buffer.set_plane(&modal);
        self.mode = modal;
    }

    #[inline]
    pub(crate) fn go(&mut self, to: LineCol) {
        self.cursor.go(to);
    }
    fn delete(&mut self) {
        match self.buffer.delete(self.pos()) {
            Ok(new_pos) => self.go(new_pos),
            Err(Error::InvalidPosition) => panic!("Cursor found in a position it should never appear in: ({}), please contact the developers.", self.pos()),
            Err(Error::ImATeacup) => {}
            Err(_) => panic!("UnexpectedError, please contact the developers.")
        }
    }
    fn push(&mut self, c: char) {
        match self.buffer.insert(self.pos(), c) {
            Ok(new_pos) => self.go(new_pos),
            Err(Error::InvalidPosition) => panic!("Cursor found in a position it should never appear in: ({}), please contact the developers.", self.pos()),
            Err(Error::ImATeacup) => {}
            Err(_) => panic!("UnexpectedError, please contact the developers.")
        }
    }
    pub fn newline(&mut self) {
        self.cursor.pos = self.buffer.insert_newline(self.pos());
    }

    /// Runs the main editor loop.
    ///
    /// This function:
    /// 1. Enables raw mode for the terminal.
    /// 2. Continuously draws the editor content and handles user input.
    /// 3. Exits when the user presses the Esc key.
    ///
    /// # Returns
    /// `Ok(())` if the editor runs and exits successfully, or an error if any operation fails.
    ///
    /// # Errors
    /// This function can return an error if:
    /// - Terminal operations fail (e.g., enabling raw mode, reading events)
    /// - Drawing operations fail
    pub fn run(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;

        loop {
            match self.mode {
                Modal::Command | Modal::Find(_) => {}
                _ => self.buffer.clear_command(),
            }
            let _ = match self.mode {
                Modal::Normal => self.run_normal(None),
                Modal::Find(find_mode) => {
                    if self.buffer.is_command_empty() {
                        match find_mode {
                            FindMode::Forwards => self.push('/'),
                            FindMode::Backwards => self.push('?'),
                        }
                    }
                    if self.run_command()? {
                        let pattern = &self.buffer.get_command_text()[0][1..];
                        let result = match find_mode {
                            FindMode::Forwards => self.buffer.find(pattern, self.last_normal_pos()),
                            FindMode::Backwards => {
                                self.buffer.rfind(pattern, self.last_normal_pos())
                            }
                        };
                        match result {
                            Err(Error::InvalidInput) => notif_bar!("Empty find query.";),
                            Err(Error::PatternNotFound) => notif_bar!("No matches found for your pattern";),
                            Err(_) => panic!("Unexpected error returned from find. Please contact the developers."),
                            Ok(linecol) => self.cursor.last_text_mode_pos = linecol
                        }
                        self.set_mode(Modal::Normal);
                    }
                    Ok(())
                }
                Modal::Insert => self.run_insert(),
                Modal::Visual => self.run_visual(),
                Modal::Command => {
                    if self.buffer.is_command_empty() {
                        self.push(':');
                    }
                    if self.run_command()? {
                        match self.buffer.get_command_text()[0].as_str() {
                            ":q" => break,
                            "/EXIT NOW" => exit(0),
                            _ => {}
                        };
                        self.set_mode(Modal::Normal);
                    }
                    Ok(())
                }
            };
        }
        terminal::disable_raw_mode()?;
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        Ok(())
    }

    #[allow(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    fn run_visual(&mut self) -> Result<()> {
        unimplemented!()
    }
    fn run_insert(&mut self) -> Result<()> {
        self.draw_rows()?;
        draw_bar(&INFO_BAR, |term_width, _| {
            get_info_bar_content(term_width, &self.mode, &self.pos())
        })?;
        draw_bar(&NOTIFICATION_BAR, |_, _| get_notif_bar_content())?;
        self.move_cursor();

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char(c) => self.push(c),
                KeyCode::Enter => self.newline(),
                KeyCode::Esc => self.set_mode(Modal::Normal),
                KeyCode::Backspace => self.delete(),
                KeyCode::Left => self.if_within_bounds(Cursor::bump_left),
                KeyCode::Right => self.if_within_bounds(Cursor::bump_right),
                KeyCode::Up => self.if_within_bounds(Cursor::bump_up),
                KeyCode::Down => self.if_within_bounds(Cursor::bump_down),
                _ => {
                    notif_bar!("nothing");
                }
            }
        };
        Ok(())
    }
    fn run_command(&mut self) -> Result<bool> {
        self.draw_rows()?;
        draw_bar(&INFO_BAR, |term_width, _| {
            get_info_bar_content(term_width, &self.mode, &self.pos())
        })?;
        draw_bar(&COMMAND_BAR, |_, _| {
            self.buffer.get_command_text()[0].to_string()
        })?;
        let (_, term_height) = terminal::size()?;
        self.move_command_cursor(term_height);

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Enter => return Ok(true),
                KeyCode::Char(c) => self.push(c),
                KeyCode::Backspace => self.delete(),
                KeyCode::Left => self.if_within_bounds(Cursor::bump_left),
                KeyCode::Right => self.if_within_bounds(Cursor::bump_right),
                KeyCode::Esc => {
                    self.set_mode(Modal::Normal);
                }
                _ => {
                    notif_bar!("nothing";);
                }
            }
        };
        Ok(false)
    }

    /// Draws the main content of the editor.
    ///
    /// This function:
    /// 1. Clears the screen.
    /// 2. Draws each line of the buffer content.
    /// 3. Stops drawing if it reaches the bottom of the terminal or the notification/info bar.
    ///
    /// # Returns
    /// `Ok(())` if drawing succeeds, or an error if any terminal operation fails.
    ///
    /// # Errors
    /// This function can return an error if terminal operations (e.g., clearing, moving cursor, writing) fail.
    pub(crate) fn draw_rows(&self) -> Result<()> {
        let mut stdout = stdout();
        let (_, term_height) = terminal::size()?;
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )?;

        let split_line_n = term_height as usize
            - 1
            - (NOTIFICATION_BAR_Y_LOCATION as usize).max(INFO_BAR_Y_LOCATION as usize);

        for line in self.buffer.get_normal_text().iter().take(split_line_n) {
            execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
            println!("{line}\r");
        }
        Ok(())
    }

    /// Moves the cursor graphics to its current position in the editor.
    ///
    /// This function updates the terminal cursor position to match the editor's internal cursor state.
    ///
    /// # Returns
    /// `Ok(())` if the cursor is successfully moved, or an error if the operation fails.
    ///
    /// # Errors
    /// This function can return an error if the terminal cursor movement operation fails.
    pub fn move_cursor(&self) {
        let col =
            u16::try_from(self.cursor.col()).expect("Column location higher than 65356 is invalid");
        let _ = execute!(
            stdout(),
            crossterm::cursor::MoveTo(
                col,
                u16::try_from(self.cursor.line())
                    .expect("More than 65356 lines in a single file are currently unsupported.")
            )
        );
    }

    fn move_command_cursor(&self, term_size: u16) {
        let _ = execute!(
            stdout(),
            crossterm::cursor::MoveTo(
                u16::try_from(self.cursor.col())
                    .expect("Column location lower than 0 or higher than 65356 is invalid"),
                term_size - (NOTIFICATION_BAR_Y_LOCATION)
            )
        );
    }
}
