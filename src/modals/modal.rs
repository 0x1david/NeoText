use std::fmt::Display;

/// Contains the main modal variants of the editor.
#[derive(Default, Debug, PartialEq, Eq)]
pub enum Modal {
    #[default]
    Normal,
    Insert,
    Visual,
    VisualLine,
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
    pub const fn is_normal(&self) -> bool {
        matches!(self, Self::Normal)
    }

    pub const fn is_insert(&self) -> bool {
        matches!(self, Self::Insert)
    }

    pub const fn is_visual(&self) -> bool {
        matches!(self, Self::Visual)
    }

    pub const fn is_visual_line(&self) -> bool {
        matches!(self, Self::VisualLine)
    }
    pub const fn is_any_visual(&self) -> bool {
        matches!(self, Self::VisualLine) || matches!(self, Self::Visual)
    }

    pub const fn is_find(&self) -> bool {
        matches!(self, Self::Find(_))
    }

    pub const fn is_command(&self) -> bool {
        matches!(self, Self::Command)
    }
}

impl Display for Modal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let disp = match &self {
            Self::Find(_) => "FIND",
            Self::Normal => "NORMAL",
            Self::Command => "COMMAND",
            Self::Insert => "INSERT",
            Self::Visual => "VISUAL",
            Self::VisualLine => "VISUAL LINE",
        };
        write!(f, "{disp}")
    }
}
