// Features to implement:
//      Regex Command Processing
//      create a better DS for buffer
//      Marks
//      Undo and Redo
//      Command History
//      Configuration parsing and configurable controller
//      Visual Mode
//      Macros
//      Scrolling
//      LSP Integration
//      Telescope
//      Copy && Paste
//      Programmable Extensions
#![allow(dead_code)]
use std::{io::stdout, path::PathBuf};

mod error;
use buffer::VecBuffer;
use crossterm::{
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use editor::Editor;
use error::{Error, Result};

mod bars;
mod buffer;
mod cli;
mod cursor;
mod editor;
mod modals;
mod searcher;
mod utils;

fn main() {
    let _ = execute!(stdout(), EnterAlternateScreen);
    // let mut editor = MainEditor::new(VecBuffer::default());
    let p = "/home/flxvs/repositories/rust/text-editor/src/buffer.rs".into();
    let mut editor = new_from_file(p);
    let _ = editor.run();
    let _ = execute!(stdout(), LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}

/// Creates a `MainEditor` instance from a file/
/// Reads the file at `p`, converts its content to a `VecBuffer`,
/// and initializes a `MainEditor` with this buffer.
///
/// # Arguments
/// * `p` - Path to the file to be read.
///
/// # Returns
/// A `MainEditor<VecBuffer>` with the file's content.
///
/// # Panics
/// - If the file can't be read.
/// - If the file content is not valid UTF-8.
pub fn new_from_file(p: PathBuf) -> Editor<VecBuffer> {
    let content = match std::fs::read(p) {
        Err(e) => panic!("{}", e),
        Ok(content) => content,
    };
    Editor::new(VecBuffer::new(
        String::from_utf8(content)
            .expect("Invalid utf8 file")
            .lines()
            .map(String::from)
            .collect(),
    ))
}
