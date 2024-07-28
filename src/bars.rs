use std::io::{stdout, Write};

use anyhow::Result;
use crossterm::{
    execute,
    style::{self, Color},
    terminal::{self, ClearType},
};

pub const INFO_BAR_Y_LOCATION: usize = 1;
pub const NOTIFICATION_BAR_Y_LOCATION: usize = 0;
pub const INFO_BAR_LINEWIDTH_INDICATOR_X_LOCATION_NEGATIVE: usize = 1;
pub const INFO_BAR_MODAL_INDICATOR_X_LOCATION: usize = 1;
pub const NOTIFICATION_BAR_TEXT_X_LOCATION: usize = 2;
pub const DEFAULT_FG: Color = Color::White;
pub const DEFAULT_BG: Color = Color::Reset;

pub const NOTIFICATION_BAR: BarInfo = BarInfo::new(
    NOTIFICATION_BAR_Y_LOCATION as u16,
    NOTIFICATION_BAR_TEXT_X_LOCATION,
    DEFAULT_FG,
    DEFAULT_BG,
);

pub const INFO_BAR: BarInfo = BarInfo::new(
    INFO_BAR_Y_LOCATION as u16,
    INFO_BAR_MODAL_INDICATOR_X_LOCATION,
    DEFAULT_FG,
    Color::DarkGrey,
);

pub const COMMAND_BAR: BarInfo = BarInfo::new(
    NOTIFICATION_BAR_Y_LOCATION as u16,
    0,
    DEFAULT_FG,
    DEFAULT_BG,
);

pub struct BarInfo {
    pub y_offset: u16,
    /// Foreground color
    pub fg_color: Color,
    /// Background color
    pub bg_color: Color,
    pub x_padding: usize,
}

impl BarInfo {
    const fn new(y_offset: u16, x_padding: usize, fg_color: Color, bg_color: Color) -> Self {
        Self {
            y_offset,
            x_padding,
            fg_color,
            bg_color,
        }
    }
}

pub fn draw_bar<F>(bar: &BarInfo, content_generator: F) -> Result<()>
where
    F: FnOnce(usize, usize) -> String,
{
    let mut stdout = stdout();
    let (term_width, term_height) = terminal::size()?;
    let y_position = term_height - 1 - bar.y_offset;

    execute!(
        stdout,
        crossterm::cursor::MoveTo(0, y_position),
        terminal::Clear(ClearType::CurrentLine),
        style::SetForegroundColor(bar.fg_color),
        style::SetBackgroundColor(bar.bg_color),
    )?;
    let content = content_generator(term_width as usize, term_height as usize);
    print!("{}{}", " ".repeat(bar.x_padding), content);

    let remaining_width = term_width as usize - content.len() - bar.x_padding;
    print!("{}", " ".repeat(remaining_width));
    stdout.flush()?;
    execute!(stdout, style::ResetColor)?;

    Ok(())
}
