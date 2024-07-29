use std::{cmp::Ordering, fmt::Display};

use crate::modal::Modal;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}

impl Display for LineCol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

impl PartialOrd for LineCol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.line.cmp(&other.line) {
            Ordering::Equal => self.col.cmp(&other.col).into(),
            otherwise => Some(otherwise),
        }
    }
}

/// The overarching cursor struct
pub struct Cursor {
    pub pos: LineCol,
    pos_initial: LineCol,
    col_max: usize,
    line_max: usize,
    plane: CursorPlane,
    pub last_text_mode_pos: LineCol,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            pos: LineCol { line: 0, col: 0 },
            pos_initial: LineCol { line: 0, col: 0 },
            col_max: 0,
            line_max: 0,
            plane: CursorPlane::Text,
            last_text_mode_pos: LineCol { line: 0, col: 0 },
        }
    }
}

impl Cursor {
    #[inline]
    pub fn go(&mut self, to: LineCol) {
        self.pos = to;
    }

    #[inline]
    pub fn line(&self) -> usize {
        self.pos.line
    }

    #[inline]
    pub fn set_line(&mut self, new: usize) {
        self.pos.line = new
    }

    #[inline]
    pub fn col(&self) -> usize {
        self.pos.col
    }

    #[inline]
    pub fn set_col(&mut self, new: usize) {
        self.pos.col = new
    }
    /// Moves the cursor one position to the left, if there's left to go to, otherwise remains in
    /// place.
    #[inline]
    pub fn bump_left(&mut self) {
        if self.col() != 0 {
            self.set_col(self.col() - 1);
        }
    }
    /// Moves the cursor one position to the right, if there's right to go to, otherwise remains in
    /// place.
    #[inline]
    pub fn bump_right(&mut self) {
        self.set_col(self.col() + 1);
    }
    /// Moves the cursor one position up, if there's upper line to go to, otherwise remains in
    /// place.
    #[inline]
    pub fn bump_up(&mut self) {
        if self.line() != 0 {
            self.set_line(self.line() - 1);
        }
    }
    /// Moves the cursor one position down, if there's lower line to go to, otherwise remains in
    /// place.
    #[inline]
    pub fn bump_down(&mut self) {
        self.set_line(self.line() + 1);
    }
    /// Moves the cursor left by the specified distance, clamping at zero.
    /// TODO: Check whether Y is in the allowed boundaries for the new row, if it isnt, update the
    /// value
    #[inline]
    pub fn jump_left(&mut self, dist: usize) {
        self.set_col(self.col().saturating_sub(dist))
    }
    /// Moves the cursor right by the specified distance, clamping at the end of a row.
    /// TODO: Check whether Y is in the allowed boundaries for the new row, if it isnt, update the
    /// value
    #[inline]
    pub fn jump_right(&mut self, dist: usize) {
        self.set_col(self.col_max.min(self.col() + dist));
    }
    /// Moves the cursor up by the specified distance, clamping at the top.
    /// TODO: Check whether X is in the allowed boundaries for the new row, if it isnt, update the
    /// value
    #[inline]
    pub fn jump_up(&mut self, dist: usize) {
        self.set_line(self.line().saturating_sub(dist))
    }
    /// Moves the cursor down by the specified distance, clamping at the bottom of the file.
    /// TODO: Check whether X is in the allowed boundaries for the new row, if it isnt, update the
    /// value
    #[inline]
    pub fn jump_down(&mut self, dist: usize) {
        self.set_line(self.line_max.min(self.line() + dist));
    }

    /// Updates the location the cursor points at depending on the current active modal state.
    pub fn mod_change(&mut self, modal: &Modal) {
        if self.plane.text() {
            self.last_text_mode_pos = self.pos
        }
        match modal {
            Modal::Command => {
                self.plane = CursorPlane::CommandBar;
                self.pos = LineCol { line: 0, col: 0 };
            }
            Modal::Find => {
                self.plane = CursorPlane::CommandBar;
                self.pos = LineCol { line: 0, col: 0 };
            }
            Modal::Normal | Modal::Insert | Modal::Visual => {
                self.plane = CursorPlane::Text;
                self.pos = self.last_text_mode_pos;
            }
        }
        self.pos_initial = LineCol {
            line: self.line(),
            col: self.col(),
        };
    }
}

/// Specifies at which plane the cursor is currently located.
enum CursorPlane {
    Text,
    CommandBar,
    Terminal,
}
impl CursorPlane {
    fn text(&self) -> bool {
        match &self {
            Self::Text => true,
            _ => false,
        }
    }
}
