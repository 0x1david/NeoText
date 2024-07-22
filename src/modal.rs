use anyhow::Result;
/// Contains the main modal variants of the editor.
#[derive(Default)]
pub enum Modal {
    #[default]
    Normal,
    Insert,
    Visual,
    Find,
    Command,
}

impl Modal {
    /// The null command is for implementation on bindings that have no associated commands for a
    /// given modal.
    fn null(&self) -> Result<()> {
        Ok(())
    }
}
