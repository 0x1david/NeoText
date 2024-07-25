use crate::buffer::{BufferError, TextBuffer, VecBuffer};
use crate::cursor::{Cursor, LineCol};
use anyhow::{Result, Context};
use crossterm::style::{self, style, Color};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
};


use crate::modal::Modal;
use std::io::{stdout, Write};

const INFO_BAR_Y_LOCATION: u16 = 1;
const INFO_BAR_LINEWIDTH_INDICATOR: usize = 4;

/// The main editor is used as the main API for all commands
pub struct MainEditor<Buff: TextBuffer> {
    /// In the first implementation I will start with Vec, for simplicity, fairly early to the dev
    /// process a better data structure will have to be found and vec replaced;
    cursor: Cursor,
    buffer: Buff,
}

impl<Buff: TextBuffer> MainEditor<Buff> {
    fn if_within_bounds<F>(&mut self, movement: F) 
    where F: FnOnce(&mut Cursor){
        let original_pos = self.pos();
        movement(&mut self.cursor);
        if dbg!(self.pos().line > self.buffer.max_line()) {
            self.cursor.pos = original_pos;
            return
        }
        let new_pos = self.pos();
        let max_col = self.buffer.max_col(new_pos);
        if new_pos.col > max_col {
            self.cursor.set_col(max_col);
        }
    }

    #[inline]
    fn pos(&self) -> LineCol {
        self.cursor.pos
    }

    #[inline]
    fn go(&mut self, to: LineCol) {
        self.cursor.go(to);
    }
    fn delete(&mut self) {
        match self.buffer.delete(self.pos()) {
            Ok(new_pos) => self.go(new_pos),
            Err(BufferError::InvalidPosition) => panic!("Cursor found in a position it should never appear in, please contact the developers."),
            Err(BufferError::ImATeacup) => {}
            Err(_) => panic!("UnexpectedError, please contact the developers.")
        }
    }
    fn push(&mut self, c: char) {
        match self.buffer.insert(self.pos(), c) {
            Ok(new_pos) => self.go(new_pos),
            Err(BufferError::InvalidPosition) => panic!("Cursor found in a position it should never appear in, please contact the developers."),
            Err(BufferError::ImATeacup) => {}
            Err(_) => panic!("UnexpectedError, please contact the developers.")
        }
    }
    fn newline(&mut self) {
        self.cursor.pos = self.buffer.insert_newline(self.pos());
    }
}

impl<Buff: TextBuffer> MainEditor<Buff> {
    pub fn new(buffer: Buff) -> Self {
        MainEditor {
            buffer,
            cursor: Cursor::default(),
            // mode: Modal::default(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        // let stdout = stdout();
        // execute!(stdout, terminal::Clear(ClearType::All))?;

        loop {
            self.draw_rows()?;
            self.draw_location_bar()?;
            self.move_cursor()?;

            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char(c) => self.push(c),
                    KeyCode::Enter => self.newline(),
                    KeyCode::Backspace => self.delete(),
                    KeyCode::Left => self.if_within_bounds(Cursor::bump_left),
                    KeyCode::Right => self.if_within_bounds(Cursor::bump_right),
                    KeyCode::Up => self.if_within_bounds(Cursor::bump_up),
                    KeyCode::Down => self.if_within_bounds(Cursor::bump_down),
                    KeyCode::Esc => break,
                    _ => {println!("nothing")}
                }
            }
        }
        
        Ok(())
        }

    //         terminal::disable_raw_mode()?;
    //         execute!(stdout, terminal::Clear(ClearType::All))?;
    //         Ok(())
    //     }

    fn draw_rows(&self) -> Result<()> {
        let mut stdout = stdout();
        let (_, term_height) = terminal::size()?;
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )?;
        for (i, line) in self.buffer.get_entire_text().iter().enumerate() {
            if i >= term_height as usize - 1 {
                break;
            }
            execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
            println!("{}\r", line);
        }
        Ok(())
    }

    fn move_cursor(&self) -> Result<()> {
        execute!(
            stdout(),
            crossterm::cursor::MoveTo(self.cursor.col() as u16, self.cursor.line() as u16)
        )
        .context("Failed moving cursor ")
    }
    fn draw_location_bar(&self) -> Result<()> {
        let mut stdout = stdout();
        let (term_width, term_height) = terminal::size()?;
        execute!(
            stdout,
            crossterm::cursor::MoveTo(0, term_height - 1 - INFO_BAR_Y_LOCATION),
            terminal::Clear(ClearType::CurrentLine),
            style::SetBackgroundColor(Color::DarkGrey),
            style::SetForegroundColor(Color::White),
        )?;

        let pos_string = format!("{}", self.pos());
        print!("{}{}", " ".repeat(INFO_BAR_LINEWIDTH_INDICATOR),pos_string);
        
        // Fill the rest of the line with spaces
        let remaining_width = term_width as usize - pos_string.len() - INFO_BAR_LINEWIDTH_INDICATOR;
        print!("{:width$}", "", width = remaining_width);
        
        stdout.flush()?;
        
        execute!(
            stdout,
            style::ResetColor
        )?;
        
        Ok(())
    }
}

//     fn insert_char(&mut self, c: char) {
//         let current_line = &mut self.content[self.cursor_y];
//         current_line.insert(self.cursor_x, c);
//         self.cursor_x += 1;
//     }

//     fn insert_newline(&mut self) {
//         let current_line = &mut self.content[self.cursor_y];
//         let rest_of_line = current_line.split_off(self.cursor_x);
//         self.content.insert(self.cursor_y + 1, rest_of_line);
//         self.cursor_y += 1;
//         self.cursor_x = 0;
//     }

//     fn delete_char(&mut self) {
//         let current_line = &mut self.content[self.cursor_y];
//         if self.cursor_x > 0 {
//             current_line.remove(self.cursor_x - 1);
//             self.cursor_x -= 1;
//         } else if self.cursor_y > 0 {
//             let line = self.content.remove(self.cursor_y);
//             self.cursor_y -= 1;
//             self.cursor_x = self.content[self.cursor_y].len();
//             self.content[self.cursor_y].push_str(&line);
//         }
//     }

//     fn move_cursor_left(&mut self) {
//         if self.cursor_x > 0 {
//             self.cursor_x -= 1;
//         } else if self.cursor_y > 0 {
//             self.cursor_y -= 1;
//             self.cursor_x = self.content[self.cursor_y].len();
//         }
//     }

//     fn move_cursor_right(&mut self) {
//         if self.cursor_x < self.content[self.cursor_y].len() {
//             self.cursor_x += 1;
//         } else if self.cursor_y < self.content.len() - 1 {
//             self.cursor_y += 1;
//             self.cursor_x = 0;
//         }
//     }

//     fn move_cursor_up(&mut self) {
//         if self.cursor_y > 0 {
//             self.cursor_y -= 1;
//             self.cursor_x = self.cursor_x.min(self.content[self.cursor_y].len());
//         }
//     }

//     fn move_cursor_down(&mut self) {
//         if self.cursor_y < self.content.len() - 1 {
//             self.cursor_y += 1;
//             self.cursor_x = self.cursor_x.min(self.content[self.cursor_y].len());
//         }
//     }
// }
