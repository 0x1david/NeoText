#![allow(dead_code)]
use std::{io::stdout, path::PathBuf};

use buffer::VecBuffer;
use crossterm::{
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use editor::MainEditor;

mod bars;
mod buffer;
mod cli;
mod cursor;
mod editor;
mod modal;
mod searcher;

fn main() {
    let _ = execute!(stdout(), EnterAlternateScreen);
    // let mut editor = MainEditor::new(VecBuffer::default());
    let p = "/home/flxvs/repositories/rust/text-editor/src/buffer.rs".into();
    let mut editor = new_from_file(p);
    let _ = editor.run();
    let _ = execute!(stdout(), LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}

pub fn new_from_file(p: PathBuf) -> MainEditor<VecBuffer> {
    let content = match std::fs::read(p) {
        Err(e) => panic!("{}", e),
        Ok(content) => content,
    };
    MainEditor::new(VecBuffer::new(
        String::from_utf8(content)
            .expect("Invalid utf8 file")
            .lines()
            .map(String::from)
            .collect(),
    ))
}
