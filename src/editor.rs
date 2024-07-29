use crate::bars::{
    draw_bar, COMMAND_BAR, INFO_BAR, INFO_BAR_LINEWIDTH_INDICATOR_X_LOCATION_NEGATIVE,
    INFO_BAR_MODAL_INDICATOR_X_LOCATION, INFO_BAR_Y_LOCATION, NOTIFICATION_BAR,
    NOTIFICATION_BAR_Y_LOCATION,
};
use crate::buffer::{BufferError, TextBuffer};
use crate::cursor::{Cursor, LineCol};
use crate::modal::{FindMode, Modal};
use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
};

// use crate::modal::Modal;
use std::collections::VecDeque;
use std::io::stdout;
use std::process::exit;
use std::sync::{Mutex, OnceLock};

static DEBUG_MESSAGES: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

/// Retrieves or initializes the global debug message queue.
///
/// Returns a static reference to a `Mutex<VecDeque<String>>` which stores
/// debug messages used by the `bar_dbg!` macro. Initializes the queue
/// on first call.
pub fn get_debug_messages() -> &'static Mutex<VecDeque<String>> {
    DEBUG_MESSAGES.get_or_init(|| Mutex::new(VecDeque::new()))
}

/// A versatile debugging macro that logs expressions and their values to an info bar,
/// similar to the standard `dbg!` macro, with additional flexibility.
///
/// This macro captures the file name and line number where it's invoked,
/// evaluates the given expression(s), formats a debug message, and adds it
/// to a global debug message queue. It can either return the value of the expression
/// or not, depending on how it's used.
///
/// # Features
/// - Logs the file name and line number of the macro invocation
/// - Logs the expression as a string and its evaluated value
/// - Can handle multiple expressions
/// - Optionally returns the value of the expression, allowing inline use
/// - Maintains a queue of the last 10 debug messages
/// - Behavior changes based on the presence or absence of a trailing semicolon
///
/// # Usage
/// ```
/// let x = notif_bar!(5 + 3);  // Logs and returns 8
/// notif_bar!(5 + 3);  // Logs without returning
/// let (a, b) = notif_bar!(1, "two");  // Logs and returns (1, "two")
/// notif_bar!(1, "two");  // Logs multiple values without returning
/// ```
///
/// # Syntax
/// - `notif_bar!(expr)` - Logs and returns the value of `expr`
/// - `notif_bar!(expr);` - Logs the value of `expr` without returning
/// - `notif_bar!(expr1, expr2, ...)` - Logs and returns multiple values as a tuple
/// - `notif_bar!(expr1, expr2, ...);` - Logs multiple values without returning
///
/// # Notes
/// - The expression(s) must implement the `Debug` trait for proper formatting
/// - If the debug message queue exceeds 10 messages, the oldest message is removed
/// - The presence or absence of a trailing semicolon determines whether the macro returns a value
///
/// # Panics
/// This macro will not panic, but it may fail silently if it cannot acquire
/// the lock on the debug message queue.the debug message queue.
#[macro_export]
macro_rules! notif_bar {
    // Version that returns the value (no semicolon)
    ($val:expr) => {{
        let file = file!();
        let line = line!();
        let val = $val;
        let message = format!("[{}:{}] {} = {:?}", file, line, stringify!($val), &val);
        if let Ok(mut messages) = get_debug_messages().lock() {
            messages.push_back(message);
            if messages.len() > 10 {
                messages.pop_front();
            }
        }
        val
    }};

    // Version that doesn't return the value (with semicolon)
    ($val:expr;) => {{
        let file = file!();
        let line = line!();
        let message = format!("[{}:{}] {} = {:?}", file, line, stringify!($val), &$val);
        if let Ok(mut messages) = get_debug_messages().lock() {
            messages.push_back(message);
            if messages.len() > 10 {
                messages.pop_front();
            }
        }
    }};

    // Multiple arguments version (no semicolon)
    ($($val:expr),+ $(,)?) => {
        ($(notif_bar!($val)),+,)
    };

    // Multiple arguments version (with semicolon)
    ($($val:expr),+ $(,)?;) => {
        $(notif_bar!($val;))+
    };
}

/// The main editor is used as the main API for all commands
pub struct MainEditor<Buff: TextBuffer> {
    /// In the first implementation I will start with Vec, for simplicity, fairly early to the dev
    /// process a better data structure will have to be found and vec replaced;
    cursor: Cursor,
    buffer: Buff,
    mode: Modal,
}

impl<Buff: TextBuffer> MainEditor<Buff> {
    /// Applies a cursor movement if it results in a valid position within the buffer bounds.
    ///
    /// # Arguments
    /// * `movement` - A function that takes a mutable reference to a Cursor and moves it.
    ///
    /// # Behavior
    /// 1. Stores the original cursor position.
    /// 2. Applies the movement to the cursor.
    /// 3. If the new line exceeds the buffer's max line, reverts to the original position.
    /// 4. If the new column exceeds the max column for that line, adjusts to the max column.
    fn if_within_bounds<F>(&mut self, movement: F)
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
    fn pos(&self) -> LineCol {
        self.cursor.pos
    }
    fn last_normal_pos(&self) -> LineCol {
        self.cursor.last_text_mode_pos
    }
    fn set_mode(&mut self, modal: Modal) {
        self.cursor.mod_change(&modal);
        self.buffer.set_plane(&modal);
        self.mode = modal;
    }

    #[inline]
    fn go(&mut self, to: LineCol) {
        self.cursor.go(to);
    }
    fn delete(&mut self) {
        match self.buffer.delete(self.pos()) {
            Ok(new_pos) => self.go(new_pos),
            Err(BufferError::InvalidPosition) => panic!("Cursor found in a position it should never appear in: ({}), please contact the developers.", self.pos()),
            Err(BufferError::ImATeacup) => {}
            Err(_) => panic!("UnexpectedError, please contact the developers.")
        }
    }
    fn push(&mut self, c: char) {
        match self.buffer.insert(self.pos(), c) {
            Ok(new_pos) => self.go(new_pos),
            Err(BufferError::InvalidPosition) => panic!("Cursor found in a position it should never appear in: ({}), please contact the developers.", self.pos()),
            Err(BufferError::ImATeacup) => {}
            Err(_) => panic!("UnexpectedError, please contact the developers.")
        }
    }
    fn newline(&mut self) {
        self.cursor.pos = self.buffer.insert_newline(self.pos());
    }
}

impl<Buff: TextBuffer> MainEditor<Buff> {
    /// Creates a new instance of MainEditor.
    ///
    /// # Arguments
    /// * `buffer` - The text buffer to be edited.
    ///
    /// # Returns
    /// A new `MainEditor` instance initialized with the given buffer and default cursor position.
    pub fn new(buffer: Buff) -> Self {
        MainEditor {
            buffer,
            cursor: Cursor::default(),
            mode: Modal::default(),
        }
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
                Modal::Normal => self.run_normal(),
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
                            Err(BufferError::InvalidInput) => notif_bar!("Empty find query.";),
                            Err(BufferError::PatternNotFound) => notif_bar!("No matches found for your pattern";),
                            Err(_) => panic!("Unexpected error returned from find. Please contact the developers."),
                            Ok(linecol) => self.cursor.last_text_mode_pos = linecol
                        }
                        self.set_mode(Modal::Normal)
                    }
                    Ok(())
                }
                Modal::Insert => self.run_insert(),
                Modal::Visual => self.run_visual(),
                Modal::Command => {
                    if self.buffer.is_command_empty() {
                        self.push(':')
                    }
                    if self.run_command()? {
                        match self.buffer.get_command_text()[0].as_str() {
                            ":q" => break,
                            "/EXIT NOW" => exit(0),
                            _ => {}
                        };
                        self.set_mode(Modal::Normal)
                    }
                    Ok(())
                }
            };
        }
        Ok(())
    }

    fn run_visual(&mut self) -> Result<()> {
        unimplemented!()
    }
    fn run_insert(&mut self) -> Result<()> {
        self.draw_rows()?;
        draw_bar(&INFO_BAR, |term_width, _| {
            self.get_info_bar_content(term_width)
        })?;
        draw_bar(&NOTIFICATION_BAR, |_, _| self.get_notif_bar_content())?;
        self.move_cursor()?;

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
            self.get_info_bar_content(term_width)
        })?;
        draw_bar(&COMMAND_BAR, |_, _| {
            self.buffer.get_command_text()[0].to_string()
        })?;
        let (_, term_height) = terminal::size()?;
        self.move_command_cursor(term_height)?;

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
                    notif_bar!("nothing";)
                }
            }
        };
        Ok(false)
    }
    fn run_normal(&mut self) -> Result<()> {
        self.draw_rows()?;
        draw_bar(&INFO_BAR, |term_width, _| {
            self.get_info_bar_content(term_width)
        })?;
        draw_bar(&NOTIFICATION_BAR, |_, _| self.get_notif_bar_content())?;
        self.move_cursor()?;

        if let Event::Key(key_event) = event::read()? {
            if let KeyCode::Char(ch) = key_event.code {
                match ch {
                    'i' => self.set_mode(Modal::Insert),
                    'o' => {
                        self.set_mode(Modal::Insert);
                        self.newline()
                    }
                    ':' => self.set_mode(Modal::Command),
                    '/' => self.set_mode(Modal::Find(FindMode::Forwards)),
                    '?' => self.set_mode(Modal::Find(FindMode::Backwards)),
                    'h' => self.if_within_bounds(Cursor::bump_left),
                    'l' => self.if_within_bounds(Cursor::bump_right),
                    'k' => self.if_within_bounds(Cursor::bump_up),
                    'j' => self.if_within_bounds(Cursor::bump_down),
                    _ => {
                        notif_bar!("nothing");
                    }
                }
            } else {
                match key_event.code {
                    KeyCode::Esc => exit(0),
                    _ => {
                        notif_bar!("nothing");
                    }
                }
            }
        };
        Ok(())
    }

    //         terminal::disable_raw_mode()?;
    //         execute!(stdout, terminal::Clear(ClearType::All))?;
    //         Ok(())
    //     }

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
    fn draw_rows(&self) -> Result<()> {
        let mut stdout = stdout();
        let (_, term_height) = terminal::size()?;
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )?;

        let split_line_n =
            term_height as usize - 1 - (NOTIFICATION_BAR_Y_LOCATION).max(INFO_BAR_Y_LOCATION);

        for line in self.buffer.get_normal_text().iter().take(split_line_n) {
            execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
            println!("{}\r", line);
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
    fn move_cursor(&self) -> Result<()> {
        execute!(
            stdout(),
            crossterm::cursor::MoveTo(self.cursor.col() as u16, self.cursor.line() as u16)
        )
        .context("Failed moving cursor ")
    }

    fn move_command_cursor(&self, term_size: u16) -> Result<()> {
        execute!(
            stdout(),
            crossterm::cursor::MoveTo(
                self.cursor.col() as u16,
                term_size - NOTIFICATION_BAR_Y_LOCATION as u16
            )
        )
        .context("Failed moving cursor ")
    }

    /// Draws the notification bar at the bottom of the terminal.
    ///
    /// This function is responsible for rendering the debug notification bar, which displays
    /// the most recent message from the debug queue and potentially other editor status
    /// information. It performs the following operations:
    ///
    /// # Display Characteristics
    /// - Location: Positioned `NOTIFICATION_BAR_Y_LOCATION` lines from the bottom of the terminal.
    /// - Color: White text on the terminal's default background.
    /// - Padding: Starts `NOTIFICATION_BAR_TEXT_X_LOCATION` spaces from the left edge.
    /// - Width: Utilizes the full width of the terminal, truncating the message if necessary.
    ///
    /// # Message Handling
    /// - Messages exceeding the available width are truncated with an ellipsis ("...").
    /// - After displaying, the message is removed from the queue.
    ///
    /// # Errors
    /// Returns a `Result` which is:
    /// - `Ok(())` if all terminal operations succeed.
    /// - `Err(...)` if any terminal operation fails (e.g., writing to stdout, flushing).
    fn get_notif_bar_content(&self) -> String {
        get_debug_messages()
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or("".to_string())
    }

    /// Draws the information bar at the bottom of the editor.
    ///
    /// This function renders an information bar that displays the current cursor position
    /// and potentially other editor status information.
    ///
    /// # Display Characteristics
    /// - Location: Positioned `INFO_BAR_Y_LOCATION` lines from the bottom of the terminal.
    /// - Background: Dark grey
    /// - Text Color: White
    /// - Content: Displays the cursor position, starting at `INFO_BAR_LINEWIDTH_INDICATOR_X_LOCATION`
    ///
    /// # Returns
    /// `Ok(())` if the info bar is successfully drawn, or an error if any terminal operation fails.
    ///
    /// # Errors
    /// This function can return an error if:
    /// - Terminal size cannot be determined
    /// - Cursor movement fails
    /// - Writing to stdout fails
    /// - Color setting or resetting fails
    fn get_info_bar_content(&self, term_width: usize) -> String {
        let modal_string = format!("{}", self.mode);
        let pos_string = format!("{}", self.pos());

        let middle_space = term_width
            - INFO_BAR_MODAL_INDICATOR_X_LOCATION
            - modal_string.len()
            - pos_string.len()
            - INFO_BAR_LINEWIDTH_INDICATOR_X_LOCATION_NEGATIVE;

        format!(
            "{}{}{}{}",
            modal_string,
            " ".repeat(middle_space),
            pos_string,
            " ".repeat(INFO_BAR_LINEWIDTH_INDICATOR_X_LOCATION_NEGATIVE)
        )
    }
}

//     fn insert_char(&mut self, c: char) {
//         let current_line = &mut self.content[self.cursor_y];
//         current_line.insert(self.cursor_x, c);
//         self.cursor_x += 1;
//     }

//     fn insert_newline(&mut self) {
//         let current_line = &mut self.content[self.cursor_y];
//         let rest_of_line = current_line.split_off(self.cursor_x);
//         self.content.insert(self.cursor_y + 1, rest_of_line);
//         self.cursor_y += 1;
//         self.cursor_x = 0;
//     }

//     fn delete_char(&mut self) {
//         let current_line = &mut self.content[self.cursor_y];
//         if self.cursor_x > 0 {
//             current_line.remove(self.cursor_x - 1);
//             self.cursor_x -= 1;
//         } else if self.cursor_y > 0 {
//             let line = self.content.remove(self.cursor_y);
//             self.cursor_y -= 1;
//             self.cursor_x = self.content[self.cursor_y].len();
//             self.content[self.cursor_y].push_str(&line);
//         }
//     }

//     fn move_cursor_left(&mut self) {
//         if self.cursor_x > 0 {
//             self.cursor_x -= 1;
//         } else if self.cursor_y > 0 {
//             self.cursor_y -= 1;
//             self.cursor_x = self.content[self.cursor_y].len();
//         }
//     }

//     fn move_cursor_right(&mut self) {
//         if self.cursor_x < self.content[self.cursor_y].len() {
//             self.cursor_x += 1;
//         } else if self.cursor_y < self.content.len() - 1 {
//             self.cursor_y += 1;
//             self.cursor_x = 0;
//         }
//     }

//     fn move_cursor_up(&mut self) {
//         if self.cursor_y > 0 {
//             self.cursor_y -= 1;
//             self.cursor_x = self.cursor_x.min(self.content[self.cursor_y].len());
//         }
//     }

//     fn move_cursor_down(&mut self) {
//         if self.cursor_y < self.content.len() - 1 {
//             self.cursor_y += 1;
//             self.cursor_x = self.cursor_x.min(self.content[self.cursor_y].len());
//         }
//     }
// }
//
//
