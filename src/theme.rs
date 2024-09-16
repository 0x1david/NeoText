use crossterm::style::Color;

pub trait Theme {
    fn from_str(&self, element: &str) -> Color;
}

pub struct DefaultTheme {}

impl Theme for DefaultTheme {
    fn from_str(&self, el: &str) -> Color {
        match el {
            "function" => Color::Yellow,
            "keyword" => Color::Red,
            "comment" => Color::Grey,
            "" => Color::Reset,
            _ => Color::Reset,
        }
    }
}

// All credits for this theme go to sainnhe - `https://github.com/sainnhe/sonokai`
pub struct Sonokai {}
