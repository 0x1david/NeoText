use crate::modal::Modal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}
/// The overarching cursor struct
pub struct Cursor {
    pub pos: LineCol,
    pos_initial: LineCol,
    col_max: usize,
    line_max: usize,
    plane: CursorPlane,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            pos: LineCol { line: 0, col: 0 },
            pos_initial: LineCol { line: 0, col: 0 },
            col_max: 0,
            line_max: 0,
            plane: CursorPlane::Text,
        }
    }
}

impl Cursor {
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
        if self.col() != self.col_max {
            self.set_col(self.col() + 1);
        }
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
        if self.line() != self.line_max {
            self.set_line(self.line() + 1);
        }
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
        match modal {
            Modal::Command => self.plane = CursorPlane::CommandBar,
            _ => self.plane = CursorPlane::Text,
        }
        self.pos_initial = LineCol {
            line: self.line(),
            col: self.col(),
        };
    }
}

/// Specifies at which plane the cursor is currently located
enum CursorPlane {
    Text,
    CommandBar,
    Terminal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_movements() {
        // Initialize cursor with a 10x10 grid
        let mut cursor = Cursor {
            pos: LineCol { line: 5, col: 5 },
            pos_initial: LineCol { line: 5, col: 5 },
            col_max: 9,
            line_max: 9,
            plane: CursorPlane::Text,
        };

        // Test bump movements
        cursor.bump_left();
        assert_eq!(cursor.col(), 4);
        assert_eq!(cursor.line(), 5);

        cursor.bump_right();
        assert_eq!(cursor.col(), 5);
        assert_eq!(cursor.line(), 5);

        cursor.bump_up();
        assert_eq!(cursor.col(), 5);
        assert_eq!(cursor.line(), 4);

        cursor.bump_down();
        assert_eq!(cursor.col(), 5);
        assert_eq!(cursor.line(), 5);

        // Test bump at edges
        for _ in 0..10 {
            cursor.bump_left();
        }
        assert_eq!(cursor.col(), 0);

        for _ in 0..10 {
            cursor.bump_right();
        }
        assert_eq!(cursor.col(), 9);

        for _ in 0..10 {
            cursor.bump_up();
        }
        assert_eq!(cursor.line(), 0);

        for _ in 0..10 {
            cursor.bump_down();
        }
        assert_eq!(cursor.line(), 9);

        // Reset cursor position
        cursor.set_col(5);
        cursor.set_line(5);

        // Test jump movements
        cursor.jump_left(3);
        assert_eq!(cursor.col(), 2);
        assert_eq!(cursor.line(), 5);

        cursor.jump_right(4);
        assert_eq!(cursor.col(), 6);
        assert_eq!(cursor.line(), 5);

        cursor.jump_up(2);
        assert_eq!(cursor.col(), 6);
        assert_eq!(cursor.line(), 3);

        cursor.jump_down(3);
        assert_eq!(cursor.col(), 6);
        assert_eq!(cursor.line(), 6);

        // Test jump at edges
        cursor.jump_left(10);
        assert_eq!(cursor.col(), 0);

        cursor.jump_right(15);
        assert_eq!(cursor.col(), 9);

        cursor.jump_up(10);
        assert_eq!(cursor.line(), 0);

        cursor.jump_down(15);
        assert_eq!(cursor.line(), 9);

        // Test jumps that don't reach the edge
        cursor.set_col(5);
        cursor.set_line(5);

        cursor.jump_left(2);
        assert_eq!(cursor.col(), 3);

        cursor.jump_right(3);
        assert_eq!(cursor.col(), 6);

        cursor.jump_up(3);
        assert_eq!(cursor.line(), 2);

        cursor.jump_down(4);
        assert_eq!(cursor.line(), 6);
    }
}
