use anyhow::Result;
use crate::modal::Modal;

/// The overarching cursor struct
pub struct Cursor {
    x: usize,
    y: usize,
    x_first: usize,
    y_first: usize,
    x_max: usize,
    y_max: usize,
    loc: CursorLoc
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            x: 0,
            y: 0,
            x_max: 0,
            y_max: 0,
            x_first: 0,
            y_first: 0,
            loc: CursorLoc::Text
        }
    }
}


impl Cursor {
    pub fn bump_left(&mut self) {
        if self.x != 0 {
            self.x -= 1;
        }
    }
    pub fn bump_right(&mut self) {
        if self.x != self.x_max  {
            self.x += 1;
        }
    }
    pub fn bump_up(&mut self) {
        if self.y != 0 {
            self.y -= 1;
        }
    }
    pub fn bump_down(&mut self) {
        if self.y != self.y_max {
            self.y += 1;
        }
    }
    pub fn jump_left(&mut self, dist: usize) {
        self.x = self.x.saturating_sub(dist)
    }
    pub fn jump_right(&mut self, dist: usize) {
        self.x = self.x_max.min(self.x + dist);
    }
    pub fn jump_up(&mut self, dist: usize) {
        self.y = self.y.saturating_sub(dist)
    }
    pub fn jump_down(&mut self, dist: usize) {
        self.y = self.y_max.min(self.y + dist);
    }
    pub fn mod_change(&mut self, modal: Modal) {
        match modal {
            Modal::Command => self.loc = CursorLoc::CommandBar,
            _ => self.loc = CursorLoc::Text,
        }
        self.x_first = self.x;
        self.y_first = self.y;
    }
}

/// Specifies at which location the cursor is currently located
enum CursorLoc {
    Text,
    CommandBar,
    Terminal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_movements() {
        // Initialize cursor with a 10x10 grid
        let mut cursor = Cursor {
            x_first: 5,
            x: 5,
            y: 5,
            y_first: 5,
            x_max: 9,
            y_max: 9,
            loc: CursorLoc::Text
        };

        // Test bump movements
        cursor.bump_left();
        assert_eq!(cursor.x, 4);
        assert_eq!(cursor.y, 5);

        cursor.bump_right();
        assert_eq!(cursor.x, 5);
        assert_eq!(cursor.y, 5);

        cursor.bump_up();
        assert_eq!(cursor.x, 5);
        assert_eq!(cursor.y, 4);

        cursor.bump_down();
        assert_eq!(cursor.x, 5);
        assert_eq!(cursor.y, 5);

        // Test bump at edges
        for _ in 0..10 { cursor.bump_left(); }
        assert_eq!(cursor.x, 0);

        for _ in 0..10 { cursor.bump_right(); }
        assert_eq!(cursor.x, 9);

        for _ in 0..10 { cursor.bump_up(); }
        assert_eq!(cursor.y, 0);

        for _ in 0..10 { cursor.bump_down(); }
        assert_eq!(cursor.y, 9);

        // Reset cursor position
        cursor.x = 5;
        cursor.y = 5;

        // Test jump movements
        cursor.jump_left(3);
        assert_eq!(cursor.x, 2);
        assert_eq!(cursor.y, 5);

        cursor.jump_right(4);
        assert_eq!(cursor.x, 6);
        assert_eq!(cursor.y, 5);

        cursor.jump_up(2);
        assert_eq!(cursor.x, 6);
        assert_eq!(cursor.y, 3);

        cursor.jump_down(3);
        assert_eq!(cursor.x, 6);
        assert_eq!(cursor.y, 6);

        // Test jump at edges
        cursor.jump_left(10);
        assert_eq!(cursor.x, 0);

        cursor.jump_right(15);
        assert_eq!(cursor.x, 9);

        cursor.jump_up(10);
        assert_eq!(cursor.y, 0);

        cursor.jump_down(15);
        assert_eq!(cursor.y, 9);

        // Test jumps that don't reach the edge
        cursor.x = 5;
        cursor.y = 5;

        cursor.jump_left(2);
        assert_eq!(cursor.x, 3);

        cursor.jump_right(3);
        assert_eq!(cursor.x, 6);

        cursor.jump_up(3);
        assert_eq!(cursor.y, 2);

        cursor.jump_down(4);
        assert_eq!(cursor.y, 6);
    }
}
