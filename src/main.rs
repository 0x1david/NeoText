// Features to implement:
//      TEXT EDITING:
//          Copy && Paste
//          Regex Command Processing
//          Visual Mode
//          Undo and Redo
//
//          Macros
//          Marks
//
//      PERFORMANCE:
//          create a better DS for buffer
//
//      NEW CRATES:
//          Telescope
//          Configuration parsing and configurable controller
//          LSP Integration
//          Programmable Extensions
//
//      ADDONS:
//          Screen Splits
//          File Commands (After pressing :)  -- This is easy just inconvenient while development
//          Different cursors (Visuals)
//
// Bugs To Fix:
//      Jumps out  of bounds crash the program
#![allow(dead_code)]
use std::{io::stdout, path::PathBuf};

mod error;
use buffer::VecBuffer;
use crossterm::{execute, terminal::EnterAlternateScreen};
use editor::Editor;
use error::{Error, Result};

mod bars;
mod buffer;
mod cursor;
mod editor;
mod modals;
mod searcher;
mod utils;
mod view_window;

fn main() {
    let _ = execute!(stdout(), EnterAlternateScreen);
    // let buf = VecBuffer::default();
    // let mut editor = Editor::new(buf, true);
    let p = "/home/flxvs/repositories/rust/text-editor/src/buffer.rs".into();
    let mut editor = new_from_file(p);
    match editor.run() {
        Err(Error::ExitCall) => (),
        Ok(_) => panic!("Editor should never return without an error"),
        otherwise => panic!(
            "Err of type {:?} should be handled before reaching the main function.",
            otherwise
        ),
    }
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
    Editor::new(
        VecBuffer::new(
            String::from_utf8(content)
                .expect("Invalid utf8 file")
                .lines()
                .map(String::from)
                .collect(),
        ),
        false,
    )
}
