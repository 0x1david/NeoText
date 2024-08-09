use std::io::stdout;

use crossterm::{cursor, execute, style, terminal};

use crate::error::Result;

#[macro_export]
macro_rules! repeat {
    ($statement:expr; $count:expr) => {{
        let count = match $count {
            Some(n) => n,
            None => 1,
        };
        for _ in 0..count {
            $statement
        }
    }};
}

pub fn draw_ascii_art() -> Result<()> {
    let mut stdout = stdout();
    let (term_width, term_height) = terminal::size()?;
    let art_lines: Vec<&str> = ASCII_INTRODUCTION_SCREEN2.lines().collect();

    let visible_length = |s: &str| s.chars().filter(|c| !c.is_control()).count();
    let art_width = art_lines
        .iter()
        .map(|line| visible_length(line))
        .max()
        .unwrap_or(0);

    let art_height = art_lines.len();
    let start_y = (term_height as usize - art_height) / 2;
    let start_x = (term_width as usize - art_width) / 2;

    for (i, line) in art_lines.iter().enumerate() {
        let visible_line_length = visible_length(line);
        let padding = " ".repeat(art_width - visible_line_length);
        #[allow(clippy::cast_possible_truncation)]
        execute!(
            stdout,
            cursor::MoveTo(start_x as u16, (start_y + i) as u16),
            style::SetForegroundColor(style::Color::Cyan),
            style::Print(line),
            style::Print(padding),
            style::ResetColor
        )?;
    }
    Ok(())
}

#[allow(dead_code)]
const ASCII_INTRODUCTION_SCREEN: &str = "
░▒▓███████▓▒░  ░▒▓████████▓▒░  ░▒▓██████▓▒░  ░▒▓████████▓▒░ ░▒▓████████▓▒░ ░▒▓█▓▒░░▒▓█▓▒░ ░▒▓████████▓▒░
░▒▓█▓▒░░▒▓█▓▒░ ░▒▓█▓▒░        ░▒▓█▓▒░░▒▓█▓▒░    ░▒▓█▓▒░     ░▒▓█▓▒░        ░▒▓█▓▒░░▒▓█▓▒░    ░▒▓█▓▒░    
░▒▓█▓▒░░▒▓█▓▒░ ░▒▓█▓▒░        ░▒▓█▓▒░░▒▓█▓▒░    ░▒▓█▓▒░     ░▒▓█▓▒░        ░▒▓█▓▒░░▒▓█▓▒░    ░▒▓█▓▒░    
░▒▓█▓▒░░▒▓█▓▒░ ░▒▓██████▓▒░   ░▒▓█▓▒░░▒▓█▓▒░    ░▒▓█▓▒░     ░▒▓██████▓▒░    ░▒▓██████▓▒░     ░▒▓█▓▒░    
░▒▓█▓▒░░▒▓█▓▒░ ░▒▓█▓▒░        ░▒▓█▓▒░░▒▓█▓▒░    ░▒▓█▓▒░     ░▒▓█▓▒░        ░▒▓█▓▒░░▒▓█▓▒░    ░▒▓█▓▒░    
░▒▓█▓▒░░▒▓█▓▒░ ░▒▓█▓▒░        ░▒▓█▓▒░░▒▓█▓▒░    ░▒▓█▓▒░     ░▒▓█▓▒░        ░▒▓█▓▒░░▒▓█▓▒░    ░▒▓█▓▒░    
░▒▓█▓▒░░▒▓█▓▒░ ░▒▓████████▓▒░  ░▒▓██████▓▒░     ░▒▓█▓▒░     ░▒▓████████▓▒░ ░▒▓█▓▒░░▒▓█▓▒░    ░▒▓█▓▒░    
";

const ASCII_INTRODUCTION_SCREEN2: &str = "
███╗   ██╗███████╗ ██████╗ ████████╗███████╗██╗  ██╗████████╗
████╗  ██║██╔════╝██╔═══██╗╚══██╔══╝██╔════╝╚██╗██╔╝╚══██╔══╝
██╔██╗ ██║█████╗  ██║   ██║   ██║   █████╗   ╚███╔╝    ██║   
██║╚██╗██║██╔══╝  ██║   ██║   ██║   ██╔══╝   ██╔██╗    ██║   
██║ ╚████║███████╗╚██████╔╝   ██║   ███████╗██╔╝ ██╗   ██║   
╚═╝  ╚═══╝╚══════╝ ╚═════╝    ╚═╝   ╚══════╝╚═╝  ╚═╝   ╚═╝   ";
