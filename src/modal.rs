use std::fmt::Display;

use anyhow::Result;
/// Contains the main modal variants of the editor.
#[derive(Default, Debug, PartialEq)]
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

impl Display for Modal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let disp = match &self {
            Self::Find => "FIND",
            Self::Normal => "NORMAL",
            Self::Command => "COMMAND",
            Self::Insert => "INSERT",
            Self::Visual => "VISUAL",
        };
        write!(f, "{}", disp)
    }
}
