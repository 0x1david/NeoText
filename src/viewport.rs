use crate::{
    bars::{INFO_BAR_Y_LOCATION, NOTIFICATION_BAR_Y_LOCATION},
    editor::{LINE_NUMBER_RESERVED_COLUMNS, LINE_NUMBER_SEPARATOR_EMPTY_COLUMNS},
};

use crossterm::terminal;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::cursor::LineCol;

const WINDOW_MAX_CURSOR_PROXIMITY_TO_WINDOW_BOUNDS: usize = 8;
#[derive(Clone, Copy, Debug)]
pub struct ViewPort {
    pub top: LineCol,
    pub bot: LineCol,
}

impl ViewPort {
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
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            crossterm::cursor::MoveTo(0, 0),
        )?;

        if self.is_initial_launch {
            draw_ascii_art()?;
            self.is_initial_launch = false;
            return Ok(());
        }

        for (i, line) in self
            .buffer
            .get_full_lines_buffer_window(Some(self.view_window.top), Some(self.view_window.bot))?
            .iter()
            .enumerate()
        {
            let line_number = self.view_window.top.line + i;

            execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;

            self.create_line_numbers(&mut stdout, line_number + 1)?;
            self.draw_line(line, line_number)?;
        }

        Ok(())
    }
    fn draw_line(&self, line: impl AsRef<str>, absolute_ln: usize) -> Result<()> {
        let line = line.as_ref();
        let selection = Selection::from(&self.cursor).normalized();
        let mut stdout = stdout();
        let line_in_highlight_bounds =
            absolute_ln >= selection.start.line && absolute_ln <= selection.end.line;
        let highlight_whole_line = (self.mode.is_visual_line() && line_in_highlight_bounds)
            || absolute_ln > selection.start.line
                && (absolute_ln < selection.end.line.saturating_sub(1) && self.mode.is_visual());

        if highlight_whole_line {
            execute!(
                stdout,
                SetBackgroundColor(Color::White),
                SetForegroundColor(Color::Black)
            )?;
            write!(stdout, "{}\r", line)?;
            execute!(stdout, ResetColor)?;
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

            execute!(
                stdout,
                SetBackgroundColor(Color::White),
                SetForegroundColor(Color::Black)
            )?;
            write!(stdout, "{}", &line[start_col..end_col])?;
            execute!(stdout, ResetColor)?;

            // Print part after selection
            write!(stdout, "{}\r", &line[end_col..])?;
        } else {
            write!(stdout, "{}\r", line)?;
        }

        writeln!(stdout)?;
        Ok(())
    }
    // fn draw_line(&self, line: impl AsRef<str>, absolute_ln: usize) -> Result<()> {
    //     let line = line.as_ref();
    //     let selection = Selection::from(&self.cursor).normalized();
    //     let line_in_highlight_bounds = absolute_ln >= selection.start.line && absolute_ln <= selection.end.line;
    //     let highlight = self.mode.is_visual_line() && line_in_highlight_bounds;
    //     if highlight { execute!(stdout(), SetBackgroundColor(Color::White), SetForegroundColor(Color::Black))?; }

    //     println!("{line}\r");

    //     if highlight {
    //         execute!(stdout(), ResetColor)?;
    //     };
    //     Ok(())
    // }

    fn create_line_numbers(&self, stdout: &mut Stdout, line_number: usize) -> Result<()> {
        execute!(stdout, style::SetForegroundColor(style::Color::Green))?;
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
        execute!(stdout, ResetColor)?;
        Ok(())
    }

    pub(crate) fn center_view_window(&mut self) {
        let (_, term_height) = terminal::size().expect("Terminal detection is corrupted.");
        let term_height = term_height as usize;
        let bottom_half = term_height / 2;
        let top_half = if term_height % 2 != 0 {
            bottom_half + 1
        } else {
            bottom_half
        };

        let current_pos = self.pos();
        let top_border = current_pos.line.saturating_sub(top_half);
        let bottom_border = min(current_pos.line + bottom_half, self.buffer.max_line());
        self.view_window = {
            ViewPort {
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
        let current_line = self.pos().line;
        let top_line = self.view_window.top.line + WINDOW_MAX_CURSOR_PROXIMITY_TO_WINDOW_BOUNDS;
        let bot_line = self
            .view_window
            .bot
            .line
            .saturating_sub(WINDOW_MAX_CURSOR_PROXIMITY_TO_WINDOW_BOUNDS);

        // Adjusting by one done to prevent centering on cursor bumps
        let cursor_out_of_bounds = current_line < top_line - 1 || current_line > bot_line + 1;

        let cursor_less_than_proximity_from_top = current_line < top_line;
        let main_cursor_more_than_proximity =
            current_line > WINDOW_MAX_CURSOR_PROXIMITY_TO_WINDOW_BOUNDS;
        let cursor_less_than_proximity_from_bot = current_line > bot_line;

        if cursor_out_of_bounds {
            self.center_view_window();
        } else if cursor_less_than_proximity_from_top && main_cursor_more_than_proximity {
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
        #[allow(clippy::cast_possible_truncation)]
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

impl Default for ViewPort {
    fn default() -> Self {
        let (_, term_height) =
            terminal::size().expect("Couldn't read information about terminal size");
        let normal_window_height = usize::from(term_height).saturating_sub(1).saturating_sub(
            (NOTIFICATION_BAR_Y_LOCATION as usize).max(INFO_BAR_Y_LOCATION as usize),
        );

        Self {
            top: LineCol::default(),
            bot: LineCol {
                line: normal_window_height,
                col: 0,
            },
        }
    }
}

impl ViewPort {
    pub const fn calculate_view_cursor(&self, main_cursor_pos: LineCol) -> LineCol {
        LineCol {
            line: main_cursor_pos.line - self.top.line,
            col: main_cursor_pos.col
                + LINE_NUMBER_RESERVED_COLUMNS
                + LINE_NUMBER_SEPARATOR_EMPTY_COLUMNS,
        }
    }
}

impl Add<isize> for ViewPort {
    type Output = Self;

    fn add(self, rhs: isize) -> Self::Output {
        Self {
            top: LineCol {
                line: self.top.line + rhs.unsigned_abs(),
                col: 0,
            },
            bot: LineCol {
                line: self.bot.line + rhs.unsigned_abs(),
                col: 0,
            },
        }
    }
}

impl Sub<isize> for ViewPort {
    type Output = Self;

    /// Moves the window down by one line
    fn sub(self, rhs: isize) -> Self::Output {
        Self {
            top: LineCol {
                line: self.top.line.saturating_sub(rhs.unsigned_abs()),
                col: 0,
            },
            bot: LineCol {
                line: self.bot.line - rhs.unsigned_abs(),
                col: 0,
            },
        }
    }
}

impl AddAssign<isize> for ViewPort {
    fn add_assign(&mut self, rhs: isize) {
        self.top.line = self.top.line.saturating_sub(rhs.unsigned_abs());
        self.bot.line = self.bot.line.saturating_sub(rhs.unsigned_abs());
    }
}

impl SubAssign<isize> for ViewPort {
    fn sub_assign(&mut self, rhs: isize) {
        self.top.line = self.top.line.saturating_add(rhs.unsigned_abs());
        self.bot.line = self.bot.line.saturating_add(rhs.unsigned_abs());
    }
}
