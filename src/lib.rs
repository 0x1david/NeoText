#![allow(dead_code, clippy::cast_possible_wrap)]

use std::path::PathBuf;
use crossterm::{
    execute,
    terminal::{DisableLineWrap, EnterAlternateScreen},
};

pub mod editor;
pub(crate) mod error;
pub(crate) mod buffer;
pub(crate) mod copy_register;
pub(crate) mod cursor;
pub(crate) mod modals;
pub(crate) mod searcher;
pub(crate) mod utils;
pub(crate) mod view_window;
#[macro_use]
pub mod bars;

pub use error::{Error, Result};
pub use buffer::VecBuffer;
pub use editor::Editor;

/// Initializes the terminal for the editor.
pub fn initialize_terminal() -> std::io::Result<()> {
    execute!(std::io::stdout(), EnterAlternateScreen, DisableLineWrap)
}

/// Creates a new `Editor` instance with an empty buffer.
pub fn new_empty_editor() -> Editor<VecBuffer> {
    Editor::new(VecBuffer::default(), true)
}

/// Creates an `Editor` instance from a file.
///
/// Reads the file at `p`, converts its content to a `VecBuffer`,
/// and initializes an `Editor` with this buffer.
///
/// # Arguments
/// * `p` - Path to the file to be read.
///
/// # Returns
/// An `Editor<VecBuffer>` with the file's content.
///
/// # Errors
/// Returns an `Error` if the file can't be read or if the content is not valid UTF-8.
pub fn new_editor_from_file(p: PathBuf) -> Result<Editor<VecBuffer>> {
    let content = std::fs::read(&p)?;
    let buffer = VecBuffer::new(
        String::from_utf8(content)
            .map_err(|_| Error::InvalidUtf8)?
            .lines()
            .map(String::from)
            .collect(),
    );
    Ok(Editor::new(buffer, false))
}

/// Runs the editor and handles its result.
pub fn run_editor(editor: &mut Editor<VecBuffer>) -> Result<()> {
    match editor.run() {
        Err(Error::ExitCall) => Ok(()),
        Ok(()) => Err(Error::UnexpectedReturn),
        Err(e) => Err(e),
    }
}

// TODO: Implement the following features:
// - Syntax Highlighting
// - Regex Command Processing
// - Undo and Redo
// - Terminal Mode
// - Macros
// - Marks
// - Improve buffer data structure
// - Telescope integration
// - Configuration parsing and configurable controller
// - LSP Integration
// - Programmable Extensions
// - Screen Splits
// - File Commands
// - Different cursor visuals

// TODO: Fix known bugs:
// - Constant crashing
