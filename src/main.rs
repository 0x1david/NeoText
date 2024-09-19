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
use std::{fs::OpenOptions, io::Read, panic, path::PathBuf};

mod error;
use buffer::VecBuffer;
use editor::Editor;
use error::{Error, Result};

mod bars;
mod buffer;
mod copy_register;
mod cursor;
mod editor;
mod highlighter;
mod modals;
mod theme;
mod utils;
mod viewport;
use clap::Parser;
mod common;
pub use common::*;
pub use tracing::{error, info, span, warn, Instrument};
pub use tracing_subscriber::{filter::EnvFilter, fmt::Subscriber, prelude::*, Layer};
pub use tracing_tree::HierarchicalLayer;

#[derive(Parser, Debug)]
#[command(name = "neotext")]
struct Cli {
    #[arg(short, long)]
    debug: bool,

    // Open neotext on the the dedcicated testfile
    #[arg(short = 't', long)]
    test: bool,

    // Read File on given path, this argument is the default argument being passed
    #[arg(default_value = "")]
    file: String,
}
fn main() {
    setup_panic();
    let cli = Cli::parse();
    setup_tracing(cli.debug);

    let mut instance = initialize_editor(&cli);

    match instance.run_main_loop() {
        Err(Error::ExitCall) => (),
        Ok(()) => panic!("Editor should never return without an error"),
        otherwise => {
            info!("Err of type {otherwise:?} should be handled before reaching the main function.")
        }
    }
}

fn initialize_editor(cli: &Cli) -> Editor<VecBuffer> {
    if cli.test {
        return new_from_file(&"./test_file.ntxt".into());
    }

    if cli.file.is_empty() {
        editor::Editor::new(VecBuffer::new(vec![" ".to_string()]), true)
    } else {
        new_from_file(&cli.file.clone().into())
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
pub fn new_from_file(p: &PathBuf) -> Editor<VecBuffer> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(p)
        .expect("This should never fail.");

    let mut content = String::new();
    let _ = file.read_to_string(&mut content);

    let buf = VecBuffer::new(content.lines().map(String::from).collect());
    Editor::new(buf, false)
}

fn setup_tracing(debug: bool) {
    let filter = EnvFilter::try_new("info, neotext = trace, crossterm = off")
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let stderr_layer = HierarchicalLayer::new(2)
        .with_writer(std::io::stderr)
        .with_targets(true)
        .with_bracketed_fields(true);

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(stderr_layer);

    // Set debug to automatically output to a dbg file
    if debug {
        let file = std::fs::File::create("dbg").expect("Failed to create debug log file");
        let file_layer = HierarchicalLayer::new(2)
            .with_writer(file)
            .with_targets(true)
            .with_bracketed_fields(true)
            .with_ansi(false);

        subscriber.with(file_layer).init();
    } else {
        subscriber.init();
    }
}

fn setup_panic() {
    // Capture Panics
    panic::set_hook(Box::new(|panic_info| {
        let (filename, line) = panic_info
            .location()
            .map(|loc| (loc.file(), loc.line()))
            .unwrap_or(("<unknown>", 0));

        let cause = panic_info
            .payload()
            .downcast_ref::<String>()
            .map(|s| s.as_str())
            .or_else(|| panic_info.payload().downcast_ref::<&str>().copied())
            .unwrap_or("<cause unknown>");

        error!(
            "Panic occurred in file '{}' at line {}: {}",
            filename, line, cause
        );
    }));
}
