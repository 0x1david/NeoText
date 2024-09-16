use crate::{get_debug_messages, modals::Modal, LineCol, Result};
use crossterm::{
    execute,
    style::{self, Color},
    terminal::{self, ClearType},
};
use std::io::{stdout, Write};

pub const INFO_BAR_Y_LOCATION: u16 = 1;
pub const NOTIFICATION_BAR_Y_LOCATION: u16 = 0;
pub const INFO_BAR_LINEWIDTH_INDICATOR_X_LOCATION_NEGATIVE: u16 = 1;
pub const INFO_BAR_MODAL_INDICATOR_X_LOCATION: u16 = 1;
pub const NOTIFICATION_BAR_TEXT_X_LOCATION: u16 = 2;

// pub struct Theme {
//     background: Color,
//     text: Color,
//     literals: Color,
//     idents: Color,
//     numerals: Color,
//     keywords: Color,
//     calls: Color,
//     comments: Color,
//     others: Color,
// }

pub const DEFAULT_FG: Color = Color::Reset;
pub const DEFAULT_BG: Color = Color::Reset;

pub const NOTIFICATION_BAR: BarInfo = BarInfo::new(
    NOTIFICATION_BAR_Y_LOCATION,
    NOTIFICATION_BAR_TEXT_X_LOCATION,
    DEFAULT_FG,
    DEFAULT_BG,
);

pub const INFO_BAR: BarInfo = BarInfo::new(
    INFO_BAR_Y_LOCATION,
    INFO_BAR_MODAL_INDICATOR_X_LOCATION,
    DEFAULT_FG,
    Color::DarkGrey,
);

pub const COMMAND_BAR: BarInfo =
    BarInfo::new(NOTIFICATION_BAR_Y_LOCATION, 0, DEFAULT_FG, DEFAULT_BG);

pub struct BarInfo {
    pub y_offset: u16,
    pub x_padding: u16,
    /// Foreground color
    pub fg_color: Color,
    /// Background color
    pub bg_color: Color,
}

impl BarInfo {
    const fn new(y_offset: u16, x_padding: u16, fg_color: Color, bg_color: Color) -> Self {
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
    print!("{}{}", " ".repeat(bar.x_padding as usize), content);

    let remaining_width = (term_width as usize)
        .saturating_sub(content.len())
        .saturating_sub(bar.x_padding as usize);
    print!("{}", " ".repeat(remaining_width));
    stdout.flush()?;
    execute!(stdout, style::ResetColor)?;

    Ok(())
}

/// Draws the notification bar at the bottom of the terminal.
///
/// This function is responsible for rendering the debug notification bar, which displays
/// the most recent message from the debug queue and potentially other editor status
/// information. It performs the following operations:
///
/// # Display Characteristics
/// - Location: Positioned `NOTIFICATION_BAR_Y_LOCATION` lines from the bottom of the terminal.
/// - Color: White text on the terminal's default background.
/// - Padding: Starts `NOTIFICATION_BAR_TEXT_X_LOCATION` spaces from the left edge.
/// - Width: Utilizes the full width of the terminal, truncating the message if necessary.
///
/// # Message Handling
/// - Messages exceeding the available width are truncated with an ellipsis ("...").
/// - After displaying, the message is removed from the queue.
///
/// # Errors
/// Returns a `Result` which is:
/// - `Ok(())` if all terminal operations succeed.
/// - `Err(...)` if any terminal operation fails (e.g., writing to stdout, flushing).
pub fn get_notif_bar_content() -> String {
    get_debug_messages()
        .lock()
        .unwrap()
        .pop_front()
        .unwrap_or_default()
}

/// Draws the information bar at the bottom of the editor.
///
/// This function renders an information bar that displays the current cursor position
/// and potentially other editor status information.
///
/// # Display Characteristics
/// - Location: Positioned `INFO_BAR_Y_LOCATION` lines from the bottom of the terminal.
/// - Background: Dark grey
/// - Text Color: White
/// - Content: Displays the cursor position, starting at `INFO_BAR_LINEWIDTH_INDICATOR_X_LOCATION`
///
/// # Returns
/// `Ok(())` if the info bar is successfully drawn, or an error if any terminal operation fails.
///
/// # Errors
/// This function can return an error if:
/// - Terminal size cannot be determined
/// - Cursor movement fails
/// - Writing to stdout fails
/// - Color setting or resetting fails
pub fn get_info_bar_content(term_width: usize, mode: &Modal, pos: LineCol) -> String {
    let modal_string = format!("{mode}");
    let mut pos = pos.clone();
    pos.line += 1;
    let pos_string = format!("{pos}");

    let middle_space = term_width
        - INFO_BAR_MODAL_INDICATOR_X_LOCATION as usize
        - modal_string.len()
        - pos_string.len()
        - INFO_BAR_LINEWIDTH_INDICATOR_X_LOCATION_NEGATIVE as usize;

    #[allow(clippy::repeat_once)]
    let loc_neg = " ".repeat(INFO_BAR_LINEWIDTH_INDICATOR_X_LOCATION_NEGATIVE as usize);
    format!(
        "{}{}{}{}",
        modal_string,
        " ".repeat(middle_space),
        pos_string,
        loc_neg
    )
}
