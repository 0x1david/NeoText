use std::{cmp::Ordering, fmt::Display};

use crate::{modals::Modal, repeat};

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


#[derive(Debug, Clone, Copy)]
pub struct Selection {
    pub start: LineCol,
    pub end: LineCol,
}

impl Selection {
    pub const fn line_is_in_selection(&self, line: usize) -> bool {
        self.start.line < line && self.end.line > line 
    }
    pub fn normalized(mut self) -> Self {
        if self.end < self.start {
            std::mem::swap(&mut self.end, &mut self.start);
        };
        self
    }
}

impl From<&Cursor> for Selection {
    fn from(value: &Cursor) -> Self {
        Self {
            start: value.last_text_mode_pos,
            end: value.pos,
        }
    }
}

/// The overarching cursor struct
#[derive(Clone, Debug)]
pub struct Cursor {
    pub pos: LineCol,
    pub previous_pos: LineCol,
    pos_initial: LineCol,
    col_max: usize,
    line_max: usize,
    plane: CursorPlane,
    pub last_text_mode_pos: LineCol,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            pos: LineCol::default(),
            previous_pos: LineCol::default(),
            pos_initial: LineCol::default(),
            col_max: 0,
            line_max: 0,
            plane: CursorPlane::Text,
            last_text_mode_pos: LineCol::default(),
        }
    }
}

impl Cursor {
    #[inline]
    pub fn go(&mut self, to: LineCol) {
        self.previous_pos = self.pos;
        self.pos = to;
    }
    #[inline]
    pub const fn line(&self) -> usize {
        self.pos.line
    }

    #[inline]
    pub fn set_line(&mut self, new: usize) {
        self.previous_pos = self.pos;
        self.pos.line = new;
    }

    #[inline]
    pub const fn col(&self) -> usize {
        self.pos.col
    }

    #[inline]
    pub fn set_col(&mut self, new: usize) {
        self.previous_pos = self.pos;
        self.pos.col = new;
    }

    /// Moves the cursor one position to the left, if there's left to go to, otherwise remains in
    /// place.
    #[inline]
    pub fn bump_left(&mut self) {
        self.previous_pos = self.pos;
        if self.col() != 0 {
            self.pos.col -= 1;
        }
    }

    /// Moves the cursor one position to the right, if there's right to go to, otherwise remains in
    /// place.
    #[inline]
    pub fn bump_right(&mut self) {
        self.previous_pos = self.pos;
        self.pos.col += 1;
    }

    /// Moves the cursor one position up, if there's upper line to go to, otherwise remains in
    /// place.
    #[inline]
    pub fn bump_up(&mut self) {
        self.previous_pos = self.pos;
        if self.line() != 0 {
            self.pos.line -= 1;
        }
    }

    /// Moves the cursor one position down, if there's lower line to go to, otherwise remains in
    /// place.
    #[inline]
    pub fn bump_down(&mut self) {
        self.previous_pos = self.pos;
        self.pos.line += 1;
    }

    /// Moves the cursor left by the specified distance, clamping at zero.
    #[inline]
    pub fn jump_left(&mut self, dist: usize) {
        self.previous_pos = self.pos;
        self.pos.col = self.col().saturating_sub(dist);
    }

    /// Moves the cursor right by the specified distance, clamping at the end of a row.
    #[inline]
    pub fn jump_right(&mut self, dist: usize) {
        self.previous_pos = self.pos;
        self.pos.col = self.col_max.min(self.col() + dist);
    }

    /// Moves the cursor up by the specified distance, clamping at the top.
    #[inline]
    pub fn jump_up(&mut self, dist: usize) {
        self.previous_pos = self.pos;
        repeat!(self.bump_up(); Some(dist));
    }

    /// Moves the cursor down by the specified distance, clamping at the bottom.
    #[inline]
    pub fn jump_down(&mut self, dist: usize) {
        self.previous_pos = self.pos;
        self.pos.line = self.line() + dist;
        repeat!(self.bump_down(); Some(dist));
    }

    /// Updates the location the cursor points at depending on the current active modal state.
    pub fn mod_change(&mut self, modal: &Modal) {
        if self.plane.text() {
            if modal.is_visual_line() {
                self.last_text_mode_pos = LineCol {
                    line: self.pos.line,
                    col: 0,
                }
            } else {
                self.last_text_mode_pos = self.pos;
            }
            self.previous_pos = self.pos;
        }

        match modal {
            Modal::Command | Modal::Find(_) => {
                self.plane = CursorPlane::CommandBar;
                self.pos = LineCol { line: 0, col: 0 };
            }
            Modal::Normal | Modal::Insert | Modal::Visual | Modal::VisualLine => {
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
#[derive(Clone, Debug)]
enum CursorPlane {
    Text,
    CommandBar,
    Terminal,
}
impl CursorPlane {
    const fn text(&self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match &self {
            Self::Text => true,
            _ => false,
        }
    }
}
