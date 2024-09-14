use crate::{cursor::LineCol, editor::LEFT_RESERVED_COLUMNS};

const BAR_GAP: u16 = 2;

#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    pub topleft: LineCol,
    pub terminal_dimensions: LineCol,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            topleft: LineCol { line: 0, col: 0 },
            terminal_dimensions: Self::get_new_dimensions(),
        }
    }
}

impl Viewport {
    fn get_new_dimensions() -> LineCol {
        let xy = crossterm::terminal::size().expect("Need terminal information");
        LineCol {
            line: xy.1 as usize,
            col: xy.0 as usize,
        }
    }
    pub fn move_left(&mut self, by: u16) {
        self.topleft.col = self.topleft.col.saturating_sub(by as usize);
    }
    pub fn move_right(&mut self, by: u16) {
        self.topleft.col += by as usize;
    }
    pub fn move_up(&mut self, by: u16) {
        self.topleft.line = self.topleft.line.saturating_sub(by as usize);
    }
    pub fn move_down(&mut self, by: u16) {
        self.topleft.line += by as usize;
    }
    pub fn center(&mut self, cursor: LineCol) {
        let half_height = self.terminal_dimensions.line / 2;
        let half_width = self.terminal_dimensions.col / 2;

        self.topleft.line = cursor.line.saturating_sub(half_height);
        self.topleft.col = cursor.col.saturating_sub(half_width);
    }
    pub fn view_cursor(&self, cursor: LineCol) -> LineCol {
        let mut c = cursor - self.topleft;
        c.col += LEFT_RESERVED_COLUMNS - 1;
        c
    }
    pub fn update_dimensions(&mut self) {
        self.terminal_dimensions = Self::get_new_dimensions()
    }

    pub fn bottomright(&self) -> LineCol {
        let mut lc = self.topleft + self.terminal_dimensions;
        lc.line -= BAR_GAP as usize;
        lc
    }
}
