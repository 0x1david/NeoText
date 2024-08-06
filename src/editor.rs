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
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
};

use std::collections::VecDeque;
// use crate::modal::Modal;
use std::io::stdout;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::process::exit;

const MAX_HISTORY: usize = 50;
const WINDOW_MAX_CURSOR_PROXIMITY_TO_WINDOW_BOUNDS: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct ViewWindow {
    pub top: LineCol,
    pub bot: LineCol,
}

impl Default for ViewWindow {
    fn default() -> Self {
        let (_, term_height) =
            terminal::size().expect("Couldn't read information about terminal size");
        let normal_window_height = usize::from(term_height).saturating_sub(1).saturating_sub((NOTIFICATION_BAR_Y_LOCATION as usize).max(INFO_BAR_Y_LOCATION as usize));

        Self {
            top: Default::default(),
            bot: LineCol {
                line: normal_window_height,
                col: 0,
            },
        }
    }
}

impl ViewWindow {
    pub fn calculate_view_cursor(&self, main_cursor_pos: LineCol) -> LineCol {
        LineCol {
            line: main_cursor_pos.line - self.top.line,
            col: main_cursor_pos.col,
        }
    }
}

impl Add<isize> for ViewWindow {
    type Output = Self;

    fn add(self, rhs: isize) -> Self::Output {
        ViewWindow {
            top: LineCol {
                line: self.top.line + rhs as usize,
                col: 0,
            },
            bot: LineCol {
                line: self.bot.line + rhs as usize,
                col: 0,
            },
        }
    }
}

impl Sub<isize> for ViewWindow {
    type Output = Self;

    /// Moves the window down by one line
    fn sub(self, rhs: isize) -> Self::Output {
        ViewWindow {
            top: LineCol {
                line: self.top.line.saturating_sub(rhs as usize),
                col: 0,
            },
            bot: LineCol {
                line: self.bot.line - rhs as usize,
                col: 0,
            },
        }
    }
}

impl AddAssign<isize> for ViewWindow {
    fn add_assign(&mut self, rhs: isize) {
        self.top.line = self.top.line.saturating_sub(rhs as usize);
        self.bot.line = self.bot.line.saturating_sub(rhs as usize);
    }
}

impl SubAssign<isize> for ViewWindow {
    fn sub_assign(&mut self, rhs: isize) {
        self.top.line = self.top.line.saturating_add(rhs as usize);
        self.bot.line = self.bot.line.saturating_add(rhs as usize);
    }
}

/// The main editor is used as the main API for all commands
pub struct Editor<Buff: TextBuffer> {
    /// In the first implementation I will start with Vec, for simplicity, fairly early to the dev
    /// process a better data structure will have to be found and vec replaced;
    pub(crate) cursor: Cursor,
    pub(crate) prev_pos: LineCol,
    pub(crate) buffer: Buff,
    pub(crate) mode: Modal,
    pub(crate) command_history: VecDeque<String>,
    pub(crate) forwards_history: VecDeque<String>,
    pub(crate) backwards_history: VecDeque<String>,
    pub(crate) history_pointer: u8,
    pub(crate) view_window: ViewWindow,
}

impl<Buff: TextBuffer> Drop for Editor<Buff> {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(stdout(), terminal::Clear(ClearType::All));
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
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
            prev_pos: LineCol { line: 0, col: 0 },
            cursor: Cursor::default(),
            mode: Modal::default(),
            command_history: VecDeque::new(),
            forwards_history: VecDeque::new(),
            backwards_history: VecDeque::new(),
            history_pointer: 0,
            view_window: ViewWindow::default(),
        }
    }

    /// Stores a command in the search history
    fn add_to_search_history(&mut self, command: String) {
        self.forwards_history.push_front(command);
        if self.forwards_history.len() > MAX_HISTORY {
            self.forwards_history.pop_back();
        }
    }
    fn get_from_search_history(&self, nth: u8, find_mode: FindMode) -> Option<String> {
        if nth == 0 {
            return Some(String::new());
        }
        match find_mode {
            FindMode::Forwards => self.forwards_history.get((nth - 1) as usize).cloned(),
            FindMode::Backwards => self.backwards_history.get((nth - 1) as usize).cloned(),
        }
    }
    fn replay_from_search_history(&self) -> Result<()> {
        let pat = self
            .forwards_history
            .front()
            .map(String::as_str)
            .ok_or(Error::NoCommandAvailable)?;
        let (flag, pat) = pat.split_at(1);
        match flag {
            "/" => self.buffer.find(pat, self.last_normal_pos())?,
            "?" => self.buffer.rfind(pat, self.last_normal_pos())?,
            otherwise => Err(Error::ProgrammingBug {
                descr: format!(
                    "Only commands starting with `?` or `/` should be found {otherwise}"
                ),
            })?,
        };
        Ok(())
    }

    /// If the cursor is in an invalid position, applies a cursor movement that results in a valid position within the buffer bounds.
    pub fn force_within_bounds(&mut self) {
        let original_pos = self.pos();
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

    #[inline]
    pub(crate) const fn prev_pos(&self) -> LineCol {
        self.prev_pos
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
    pub fn push(&mut self, c: char) {
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
            self.force_within_bounds();
            self.control_view_window();
            match self.mode {
                Modal::Command | Modal::Find(_) => {}
                _ => self.buffer.clear_command(),
            }
            match self.mode {
                Modal::Normal => self.run_normal(None, None)?,
                Modal::Find(find_mode) => self.run_find(find_mode)?,
                Modal::Insert => self.run_insert()?,
                Modal::Visual => self.run_visual()?,
                Modal::Command => self.run_command_mode()?,
            };
        }
    }

    fn run_find(&mut self, find_mode: FindMode) -> Result<()> {
        if self.buffer.is_command_empty() {
            match find_mode {
                FindMode::Forwards => self.push('/'),
                FindMode::Backwards => self.push('?'),
            }
        }
        if self.run_command()? {
            let pat = &self.buffer.get_command_text()[0][1..];
            let (history_pat, result) = match find_mode {
                FindMode::Forwards => (
                    format!("/{pat}"),
                    self.buffer.find(pat, self.last_normal_pos()),
                ),
                FindMode::Backwards => (
                    format!("?{pat}"),
                    self.buffer.rfind(pat, self.last_normal_pos()),
                ),
            };
            self.add_to_search_history(history_pat);
            match result {
                Err(Error::InvalidInput) => notif_bar!("Empty find query.";),
                Err(Error::PatternNotFound) => notif_bar!("No matches found for your pattern";),
                Err(_) => {
                    panic!("Unexpected error returned from find. Please contact the developers.")
                }
                Ok(linecol) => self.cursor.last_text_mode_pos = linecol,
            }
            self.set_mode(Modal::Normal);
        }
        Ok(())
    }

    fn run_command_mode(&mut self) -> Result<()> {
        if self.buffer.is_command_empty() {
            self.push(':');
        }
        if self.run_command()? {
            match self.buffer.get_command_text()[0].as_str() {
                ":q" => return Err(Error::ExitCall),
                "/EXIT NOW" => exit(0),
                _ => {}
            };
            self.set_mode(Modal::Normal);
        }
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

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char(c) => self.push(c),
                KeyCode::Enter => self.newline(),
                KeyCode::Esc => self.set_mode(Modal::Normal),
                KeyCode::Backspace => self.delete(),
                KeyCode::Left => self.cursor.bump_left(),
                KeyCode::Right => self.cursor.bump_right(),
                KeyCode::Up => self.cursor.bump_up(),
                KeyCode::Down => self.cursor.bump_down(),
                _ => {
                    notif_bar!("nothing");
                }
            }
        };
        Ok(())
    }
    /// Checks if the history pointer can move further in the current mode.
    ///
    /// This function determines whether there are more historical entries
    /// available in the direction the pointer is moving, based on the current mode.
    fn can_move_history_pointer(&self) -> bool {
        let history_len = match &self.mode {
            Modal::Command => self.command_history.len(),
            Modal::Find(FindMode::Forwards) => self.forwards_history.len(),
            Modal::Find(FindMode::Backwards) => self.backwards_history.len(),
            otherwise => {
                notif_bar!(format!("Invalid mode `{otherwise}` asking for history pointer specs"););
                return false;
            }
        };
        // There is no -1 from history_len because history is looked up by n - 1 of the pointer
        // To accomodate having an empty string always be the 0th element
        history_len >= self.history_pointer as usize
    }
    fn navigate_history_backwards(&mut self) -> Result<()> {
        self.history_pointer += 1;
        if self.can_move_history_pointer() {
            match &self.mode {
                            Modal::Find(find_mode) => {
                                let hist = self.get_from_search_history(self.history_pointer, *find_mode);
                                if let Some(h) = hist {
                                    if !h.is_empty() {
                                        self.buffer.replace_command_text(h)
                                    }
                                }
                            }
                            Modal::Command => unimplemented!(),
                            otherwise => Err(Error::ProgrammingBug {descr: format!("A different mode than Find or Command set as editor modal while working in the command bar `{otherwise}`")})?
                        }
        } else {
            self.history_pointer = self.history_pointer.saturating_sub(1);
        }
        Ok(())
    }

    fn navigate_history_forwards(&mut self) -> Result<()> {
        if self.history_pointer > 0 {
            self.history_pointer -= 1;
            match &self.mode {
                            Modal::Find(find_mode) => {
                                let hist = self.get_from_search_history(self.history_pointer, *find_mode);
                                if let Some(h) = hist {
                                    self.buffer.replace_command_text(h);
                                }
                         }
                            Modal::Command => unimplemented!(),
                            otherwise => Err(Error::ProgrammingBug {descr: format!("A different mode than Find or Command set as editor modal while working in the command bar `{otherwise}`")})?
                        }
        }
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
            if key_event.code != KeyCode::Up && key_event.code != KeyCode::Down {
                self.history_pointer = 0
            }
            match key_event.code {
                KeyCode::Enter => return Ok(true),
                KeyCode::Char(c) => self.push(c),
                KeyCode::Up => self.navigate_history_backwards()?,
                KeyCode::Down => self.navigate_history_forwards()?,
                KeyCode::Backspace => self.delete(),
                KeyCode::Left => self.cursor.bump_left(),
                KeyCode::Right => self.cursor.bump_right(),
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
        // let (_, term_height) = terminal::size()?;
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )?;

        for line in self
            .buffer
            .get_buffer_window(Some(self.view_window.top), Some(self.view_window.bot))?
            .iter()
        {
            execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
            println!("{line}\r");
        }

        Ok(())
    }

    pub(crate) fn center_view_window(&mut self) {
        let (_, term_height) = terminal::size().expect("Terminal detection is corrupted.");
        let bottom_half = term_height / 2;
        let top_half = if term_height % 2 != 0 {
            bottom_half + 1
        } else {
            bottom_half
        };

        let current_pos = self.pos();
        let top_border = current_pos.line - top_half as usize;
        let bottom_border = current_pos.line + bottom_half as usize;
        self.view_window = {
            ViewWindow {
                top: LineCol {
                    line: top_border,
                    col: self.buffer.max_col(LineCol {
                        line: top_border,
                        col: 0,
                    }),
                },
                bot: LineCol {
                    line: bottom_border,
                    col: self.buffer.max_col(LineCol {
                        line: bottom_border,
                        col: 0,
                    }),
                },
            }
        }
    }
    /// Makes sure the cursor is in bounds of the view window, if it isnt' follow the cursor with
    /// the bounds
    pub(crate) fn control_view_window(&mut self) {
        let cursor_out_of_bounds = self.pos().line < self.view_window.top.line
            || self.pos().line > self.view_window.bot.line;
        let cursor_less_than_proximity_from_top = self.pos().line
            < self.view_window.top.line + WINDOW_MAX_CURSOR_PROXIMITY_TO_WINDOW_BOUNDS;
        let main_cursor_more_than_4 =
            self.pos().line > WINDOW_MAX_CURSOR_PROXIMITY_TO_WINDOW_BOUNDS;
        let cursor_less_than_proximity_from_bot = self.pos().line
            > self.view_window.bot.line - WINDOW_MAX_CURSOR_PROXIMITY_TO_WINDOW_BOUNDS;

        if cursor_out_of_bounds {
            self.center_view_window();
        } else if cursor_less_than_proximity_from_top && main_cursor_more_than_4 {
            self.view_window += 1;
        } else if cursor_less_than_proximity_from_bot {
            self.view_window -= 1;
        }
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
        let cursor = self.view_window.calculate_view_cursor(self.pos());
        notif_bar!(cursor);
        let _ = execute!(
            stdout(),
            crossterm::cursor::MoveTo(cursor.col as u16, cursor.line as u16,)
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
