use crate::{
    bars::{INFO_BAR_Y_LOCATION, NOTIFICATION_BAR_Y_LOCATION},
    editor::{LINE_NUMBER_RESERVED_COLUMNS, LINE_NUMBER_SEPARATOR_EMPTY_COLUMNS},
};
use crossterm::terminal;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::cursor::LineCol;

#[derive(Clone, Copy, Debug)]
pub struct ViewWindow {
    pub top: LineCol,
    pub bot: LineCol,
}

impl Default for ViewWindow {
    fn default() -> Self {
        let (_, term_height) =
            terminal::size().expect("Couldn't read information about terminal size");
        let normal_window_height = usize::from(term_height).saturating_sub(1).saturating_sub(
            (NOTIFICATION_BAR_Y_LOCATION as usize).max(INFO_BAR_Y_LOCATION as usize),
        );

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
            col: main_cursor_pos.col
                + LINE_NUMBER_RESERVED_COLUMNS
                + LINE_NUMBER_SEPARATOR_EMPTY_COLUMNS,
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
