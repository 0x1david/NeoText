// Features to implement:
//      TEXT EDITING:
//          Syntax Highlighting
//          Regex Command Processing
//          Undo and Redo
//          Terminal Mode
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
//      Constant crashing
#![allow(dead_code, clippy::cast_possible_wrap)]
use std::{io::stdout, path::PathBuf};

mod error;
use buffer::VecBuffer;
use crossterm::{
    execute,
    terminal::{DisableLineWrap, EnterAlternateScreen},
};
use editor::Editor;
use error::{Error, Result};

mod bars;
mod buffer;
mod copy_register;
mod cursor;
mod editor;
mod modals;
mod searcher;
mod utils;
mod view_window;

fn main() {
    let _ = execute!(stdout(), EnterAlternateScreen, DisableLineWrap);

    let args: Vec<String> = std::env::args().collect();
    let mut editor = if args.len() > 1 && (args[1] == "--empty" || args[1] == "-e") {
        let buf = VecBuffer::default();
        Editor::new(buf, true)
    } else {
        let p = "/home/flxvs/repositories/rust/text-editor/src/buffer.rs".into();
        new_from_file(p)
    };

    match editor.run() {
        Err(Error::ExitCall) => (),
        Ok(()) => panic!("Editor should never return without an error"),
        otherwise => {
            panic!("Err of type {otherwise:?} should be handled before reaching the main function.",)
        }
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
