#![allow(dead_code)]
use std::io::stdout;

use buffer::VecBuffer;
use crossterm::{
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use editor::MainEditor;

mod bars;
mod buffer;
mod cursor;
mod editor;
mod modal;

fn main() {
    let _ = execute!(stdout(), EnterAlternateScreen);
    let mut editor = MainEditor::new(VecBuffer::default());
    let _ = editor.run();
    let _ = execute!(stdout(), LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}
