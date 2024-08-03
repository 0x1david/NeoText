use std::fmt::Display;

/// Contains the main modal variants of the editor.
#[derive(Default, Debug, PartialEq, Eq)]
pub enum Modal {
    #[default]
    Normal,
    Insert,
    Visual,
    Find(FindMode),
    Command,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum FindMode {
    #[default]
    Forwards,
    Backwards,
}

impl Modal {
}

impl Display for Modal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let disp = match &self {
            Self::Find(_) => "FIND",
            Self::Normal => "NORMAL",
            Self::Command => "COMMAND",
            Self::Insert => "INSERT",
            Self::Visual => "VISUAL",
        };
        write!(f, "{disp}")
    }
}
