#![allow(dead_code)]
use anyhow::Result;
use buffer::VecBuffer;
use editor::MainEditor;

mod buffer;
mod cursor;
mod editor;
mod modal;

fn main() -> Result<()> {
    let mut editor = MainEditor::new(VecBuffer::default());
    editor.run()
}
