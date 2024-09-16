#![allow(clippy::match_wild_err_arm)]
use crate::bars::{
    draw_bar, get_info_bar_content, get_notif_bar_content, COMMAND_BAR, INFO_BAR, NOTIFICATION_BAR,
    NOTIFICATION_BAR_Y_LOCATION,
};
use crate::buffer::TextBuffer;
use crate::copy_register::CopyRegister;
use crate::cursor::{Cursor, Selection};
use crate::highlighter::{Highlighter, Style};
use crate::modals::{FindMode, Modal};
use crate::utils::draw_ascii_art;
use crate::viewport::Viewport;
use crate::{get_debug_messages, notif_bar, Error, LineCol, Result};
use crossterm::QueueableCommand;
use crossterm::{
    event::{self, Event, KeyCode},
    style::{self, Color, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use rangemap::RangeMap;
use std::{
    collections::VecDeque,
    io::{stdout, Stdout, Write},
};

const MAX_HISTORY: usize = 50;
const WINDOW_MAX_CURSOR_PROXIMITY_TO_WINDOW_BOUNDS: usize = 8;
pub const LINE_NUMBER_SEPARATOR_EMPTY_COLUMNS: usize = 4;
pub const LINE_NUMBER_RESERVED_COLUMNS: usize = 5;
pub const LEFT_RESERVED_COLUMNS: usize =
    LINE_NUMBER_RESERVED_COLUMNS + LINE_NUMBER_RESERVED_COLUMNS;

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
    pub(crate) view_window: Viewport,
    // Specifies whether a drawing of lines has happened before if the app was opened without a
    // target file
    pub(crate) is_initial_launch: bool,
    pub(crate) copy_register: CopyRegister,
    highlighter: Highlighter,
}

impl<Buff: TextBuffer> Editor<Buff> {
    /// Creates a new instance of `MainEditor`.
    ///
    /// # Arguments
    /// * `buffer` - The text buffer to be edited.
    ///
    /// # Returns
    /// A new `MainEditor` instance initialized with the given buffer and default cursor position.
    pub fn new(buffer: Buff, launch_without_target: bool) -> Self {
        Self {
            highlighter: Highlighter::new(buffer.get_coalesced_bytes())
                .expect("Tree sitter needs to parse."),
            buffer,
            prev_pos: LineCol { line: 0, col: 0 },
            cursor: Cursor::default(),
            mode: Modal::default(),
            command_history: VecDeque::new(),
            forwards_history: VecDeque::new(),
            backwards_history: VecDeque::new(),
            history_pointer: 0,
            view_window: Viewport::default(),
            is_initial_launch: launch_without_target,
            copy_register: CopyRegister::default(),
        }
    }

    /// Stores a command in the search history
    fn add_to_search_history(&mut self, command: impl Into<String>) {
        self.forwards_history.push_front(command.into());
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
            .ok_or(Error::NoCommandAvailable)?;
        let (flag, pat) = pat.split_at(1);
        match flag {
            "/" => self.buffer.find(pat, self.last_normal_pos())?,
            "?" => self.buffer.rfind(pat, self.last_normal_pos())?,
            otherwise => Err(Error::ProgrammingBug {
                descr: format!(
                    "Only commands starting with `?` or `/` should be found. Instead got ``{otherwise}"
                ),
            })?,
        };
        Ok(())
    }

    /// If the cursor is in an invalid position, applies a cursor movement that results in a valid position within the buffer bounds.
    pub fn force_within_bounds(&mut self) {
        let original_pos = self.cursor.previous_pos;
        notif_bar!(self.buffer.max_line(););
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
    pub fn run_main_loop(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;

        loop {
            let empty_buffer = self.buffer.is_empty()
                || self.buffer.line(0).is_err()
                || self.buffer.line(0).unwrap().is_empty();
            if !empty_buffer {
                self.force_within_bounds();
                self.control_view_window();
            }
            match self.mode {
                Modal::Command | Modal::Find(_) => {}
                _ => self.buffer.clear_command(),
            }
            match self.mode {
                Modal::Normal => self.run_normal(None, None)?,
                Modal::Find(find_mode) => self.run_find(find_mode)?,
                Modal::Insert => self.run_insert()?,
                Modal::Visual => self.run_normal(None, None)?,
                Modal::VisualLine => self.run_normal(None, None)?,
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
                "/EXIT NOW" => std::process::exit(0),
                _ => {}
            };
            self.set_mode(Modal::Normal);
        }
        Ok(())
    }

    fn run_insert(&mut self) -> Result<()> {
        self.draw_lines()?;
        draw_bar(&INFO_BAR, |term_width, _| {
            get_info_bar_content(term_width, &self.mode, self.pos())
        })?;
        draw_bar(&NOTIFICATION_BAR, |_, _| get_notif_bar_content())?;
        self.move_cursor();
        self.force_within_bounds();

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
                                        self.buffer.replace_command_text(h);
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
        self.draw_lines()?;
        draw_bar(&INFO_BAR, |term_width, _| {
            get_info_bar_content(term_width, &self.mode, self.pos())
        })?;
        draw_bar(&COMMAND_BAR, |_, _| {
            self.buffer.get_command_text()[0].to_string()
        })?;
        let (_, term_height) = terminal::size()?;
        self.move_command_cursor(term_height);

        if let Event::Key(key_event) = event::read()? {
            if key_event.code != KeyCode::Up && key_event.code != KeyCode::Down {
                self.history_pointer = 0;
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
    pub(crate) fn draw_lines(&mut self) -> Result<()> {
        let mut stdout = stdout();
        // let (_, term_height) = terminal::size()?;
        crossterm::execute!(
            stdout,
            terminal::Clear(ClearType::All),
            crossterm::cursor::MoveTo(0, 0),
        )?;

        if self.is_initial_launch {
            draw_ascii_art()?;
            self.is_initial_launch = false;
            return Ok(());
        }

        let mut byte_index = self.buffer.get_preceding_byte_len(self.view_window.topleft);
        let style_map = self.highlighter.highlight(self.buffer.get_entire_text())?;

        for (i, line) in self
            .buffer
            .get_full_lines_buffer_window(
                Some(self.view_window.topleft),
                Some(self.view_window.bottomright()),
            )?
            .iter()
            .enumerate()
        {
            let line_number = self.view_window.topleft.line + i;

            crossterm::execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;

            self.create_line_numbers(&mut stdout, line_number + 1)?;
            // self.draw_line(line, line_number, &mut byte_index)?;
            self.draw_line_new(line, line_number, &mut byte_index, &style_map)?;
        }

        Ok(())
    }
    /// Currently parsing through the tree and printing char by char, a more efficient version
    /// would go over a token representation by token representation. Whitespace or other symbol
    /// delimited
    fn draw_line_new(
        &self,
        line: impl AsRef<str>,
        absolute_ln: usize,
        byte_offset: &mut usize,
        style_map: &RangeMap<usize, Style>,
    ) -> Result<()> {
        let line = line.as_ref();
        let mut stdout = stdout();
        let selection = Selection::from(&self.cursor).normalized();
        let default_style = &Style::default();

        for ch in line.chars() {
            let style = style_map.get(byte_offset).unwrap_or(default_style);
            crossterm::execute!(
                stdout,
                SetBackgroundColor(Color::Reset),
                SetForegroundColor(style.fg),
                style::Print(ch)
            )?;
        }
        Ok(())
    }

    fn draw_line(
        &self,
        line: impl AsRef<str>,
        absolute_ln: usize,
        byte_offset: &mut usize,
    ) -> Result<()> {
        let line = line.as_ref();
        let selection = Selection::from(&self.cursor).normalized();
        let mut stdout = stdout();

        let line_in_highlight_bounds =
            absolute_ln >= selection.start.line && absolute_ln < selection.end.line;
        let highlight_whole_line = (self.mode.is_visual_line() && line_in_highlight_bounds)
            || absolute_ln > selection.start.line
                && (absolute_ln < selection.end.line.saturating_sub(1) && self.mode.is_visual());

        if highlight_whole_line {
            crossterm::execute!(
                stdout,
                SetBackgroundColor(Color::White),
                SetForegroundColor(Color::Black)
            )?;
            write!(stdout, "{}\r", line)?;
            crossterm::execute!(stdout, ResetColor)?;
        } else if self.mode.is_visual() && line_in_highlight_bounds {
            let start_col = if absolute_ln == selection.start.line {
                selection.start.col
            } else {
                0
            };
            let end_col = if absolute_ln == selection.end.line {
                selection.end.col
            } else {
                line.len()
            };

            write!(stdout, "{}", &line[..start_col])?;

            crossterm::execute!(
                stdout,
                SetBackgroundColor(Color::White),
                SetForegroundColor(Color::Black)
            )?;
            write!(stdout, "{}", &line[start_col..end_col])?;
            crossterm::execute!(stdout, ResetColor)?;

            // Print part after selection
            write!(stdout, "{}\r", &line[end_col..])?;
        } else {
            write!(stdout, "{}\r", line)?;
        }

        writeln!(stdout)?;
        Ok(())
    }

    fn create_line_numbers(&self, stdout: &mut Stdout, line_number: usize) -> Result<()> {
        crossterm::execute!(stdout, style::SetForegroundColor(style::Color::Green))?;
        let rel_line_number = (line_number as i64 - self.pos().line as i64 - 1).abs();
        let line_number = if rel_line_number == 0 {
            line_number as i64
        } else {
            rel_line_number
        };

        print!(
            "{line_number:>width$}{separator}",
            line_number = line_number,
            width = LINE_NUMBER_RESERVED_COLUMNS,
            separator = " ".repeat(LINE_NUMBER_SEPARATOR_EMPTY_COLUMNS)
        );
        crossterm::execute!(stdout, ResetColor)?;
        Ok(())
    }

    pub(crate) fn center_view_window(&mut self) {
        self.view_window.center(self.cursor.pos)
    }

    /// Makes sure the cursor is in bounds of the view window, if it isnt' follow the cursor with
    /// the bounds
    pub(crate) fn control_view_window(&mut self) {
        let current_line = self.pos().line;
        let top_line = self.view_window.topleft.line;
        let bot_line = self.view_window.bottomright().line;

        // Adjusting by one done to prevent centering on cursor bumps
        let cursor_out_of_bounds =
            current_line < top_line.saturating_sub(1) || current_line > bot_line + 1;

        let cursor_less_than_proximity_from_top = current_line < top_line;
        let main_cursor_more_than_proximity =
            current_line > WINDOW_MAX_CURSOR_PROXIMITY_TO_WINDOW_BOUNDS;
        let cursor_less_than_proximity_from_bot = current_line > bot_line;

        if cursor_out_of_bounds {
            self.center_view_window();
        } else if cursor_less_than_proximity_from_top && main_cursor_more_than_proximity {
            self.view_window.move_up(1)
        } else if cursor_less_than_proximity_from_bot {
            self.view_window.move_down(1)
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
        let cursor = self.view_window.view_cursor(self.pos());
        #[allow(clippy::cast_possible_truncation)]
        let _ = crossterm::execute!(
            stdout(),
            crossterm::cursor::MoveTo(cursor.col as u16, cursor.line as u16,)
        );
    }

    fn move_command_cursor(&self, term_size: u16) {
        let _ = crossterm::execute!(
            stdout(),
            crossterm::cursor::MoveTo(
                u16::try_from(self.cursor.col())
                    .expect("Column location lower than 0 or higher than 65356 is invalid"),
                term_size - (NOTIFICATION_BAR_Y_LOCATION)
            )
        );
    }
}

impl<Buff: TextBuffer> Drop for Editor<Buff> {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            stdout(),
            terminal::Clear(ClearType::All),
            crossterm::terminal::LeaveAlternateScreen
        );
    }
}
