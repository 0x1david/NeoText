use std::{collections::VecDeque, io::{stdout, Write}, sync::{Mutex, OnceLock}};

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


static DEBUG_MESSAGES: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

/// Retrieves or initializes the global debug message queue.
///
/// Returns a static reference to a `Mutex<VecDeque<String>>` which stores
/// debug messages used by the `bar_dbg!` macro. Initializes the queue
/// on first call.
pub fn get_debug_messages() -> &'static Mutex<VecDeque<String>> {
    DEBUG_MESSAGES.get_or_init(|| Mutex::new(VecDeque::new()))
}

/// A versatile debugging macro that logs expressions and their values to an info bar,
/// similar to the standard `dbg!` macro, with additional flexibility.
///
/// This macro captures the file name and line number where it's invoked,
/// evaluates the given expression(s), formats a debug message, and adds it
/// to a global debug message queue. It can either return the value of the expression
/// or not, depending on how it's used.
///
/// # Features
/// - Logs the file name and line number of the macro invocation
/// - Logs the expression as a string and its evaluated value
/// - Can handle multiple expressions
/// - Optionally returns the value of the expression, allowing inline use
/// - Maintains a queue of the last 10 debug messages
/// - Behavior changes based on the presence or absence of a trailing semicolon
///
/// # Usage
/// ```
/// let x = notif_bar!(5 + 3);  // Logs and returns 8
/// notif_bar!(5 + 3);  // Logs without returning
/// let (a, b) = notif_bar!(1, "two");  // Logs and returns (1, "two")
/// notif_bar!(1, "two");  // Logs multiple values without returning
/// ```
///
/// # Syntax
/// - `notif_bar!(expr)` - Logs and returns the value of `expr`
/// - `notif_bar!(expr);` - Logs the value of `expr` without returning
/// - `notif_bar!(expr1, expr2, ...)` - Logs and returns multiple values as a tuple
/// - `notif_bar!(expr1, expr2, ...);` - Logs multiple values without returning
///
/// # Notes
/// - The expression(s) must implement the `Debug` trait for proper formatting
/// - If the debug message queue exceeds 10 messages, the oldest message is removed
/// - The presence or absence of a trailing semicolon determines whether the macro returns a value
///
/// # Panics
/// This macro will not panic, but it may fail silently if it cannot acquire
/// the lock on the debug message queue.the debug message queue.
#[macro_export]
macro_rules! notif_bar {
    // Version that returns the value (no semicolon)
    ($val:expr) => {{
        let file = file!();
        let line = line!();
        let val = $val;
        let message = format!("[{}:{}] {} = {:?}", file, line, stringify!($val), &val);
        if let Ok(mut messages) = get_debug_messages().lock() {
            messages.push_back(message);
            if messages.len() > 10 {
                messages.pop_front();
            }
        }
        val
    }};

    // Version that doesn't return the value (with semicolon)
    ($val:expr;) => {{
        let file = file!();
        let line = line!();
        let message = format!("[{}:{}] {} = {:?}", file, line, stringify!($val), &$val);
        if let Ok(mut messages) = get_debug_messages().lock() {
            messages.push_back(message);
            if messages.len() > 10 {
                messages.pop_front();
            }
        }
    }};

    // Multiple arguments version (no semicolon)
    ($($val:expr),+ $(,)?) => {
        ($(notif_bar!($val)),+,)
    };

    // Multiple arguments version (with semicolon)
    ($($val:expr),+ $(,)?;) => {
        $(notif_bar!($val;))+
    };
}

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
