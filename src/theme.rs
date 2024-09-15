use crossterm::style::Color;

pub trait Theme {
    fn background() -> Color;
    fn text() -> Color;
    fn literal() -> Color;
    fn ident() -> Color;
    fn numeral() -> Color;
    fn keyword() -> Color;
    fn call() -> Color;
    fn comment() -> Color;
    fn other() -> Color;
}

pub struct DefaultTheme {}

impl Theme for DefaultTheme {
    fn background() -> Color {
        Color::Reset
    }
    fn text() -> Color {
        Color::Reset
    }
    fn literal() -> Color {
        Color::Reset
    }
    fn ident() -> Color {
        Color::Reset
    }
    fn numeral() -> Color {
        Color::Reset
    }
    fn keyword() -> Color {
        Color::Reset
    }
    fn call() -> Color {
        Color::Reset
    }
    fn comment() -> Color {
        Color::Reset
    }
    fn other() -> Color {
        Color::Reset
    }
}

// All credits for this theme go to sainnhe - `https://github.com/sainnhe/sonokai`
pub struct Sonokai {}
