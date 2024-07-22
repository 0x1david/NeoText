use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
};
use modal::Modal;
use std::io::stdout;
mod buffer;
mod cursor;
mod modal;
use buffer::{TextBuffer, VecBuffer};
use cursor::Cursor;

/// The main editor is used as the main API for all commands
struct MainEditor<Buff: TextBuffer> {
    /// In the first implementation I will start with Vec, for simplicity, fairly early to the dev
    /// process a better data structure will have to be found and vec replaced;
    cursor: Cursor,
    content: Buff,
}

impl<Buff: TextBuffer> MainEditor<Buff> {
    fn new(buffer: Buff) -> Self {
        MainEditor {
            content: buffer,
            cursor: Cursor::default(),
            // mode: Modal::default(),
        }
    }
}

//     fn run(&mut self) -> Result<()> {
//         terminal::enable_raw_mode()?;
//         let mut stdout = stdout();
//         execute!(stdout, terminal::Clear(ClearType::All))?;

//         loop {
//             self.draw_rows()?;
//             self.move_cursor()?;

//             if let Event::Key(key_event) = event::read()? {
//                 match key_event.code {
//                     KeyCode::Char(c) => self.insert_char(c),
//                     KeyCode::Enter => self.insert_newline(),
//                     KeyCode::Backspace => self.delete_char(),
//                     KeyCode::Left => self.move_cursor_left(),
//                     KeyCode::Right => self.move_cursor_right(),
//                     KeyCode::Up => self.move_cursor_up(),
//                     KeyCode::Down => self.move_cursor_down(),
//                     KeyCode::Esc => break,
//                     _ => {}
//                 }
//             }
//         }

//         terminal::disable_raw_mode()?;
//         execute!(stdout, terminal::Clear(ClearType::All))?;
//         Ok(())
//     }

//     fn draw_rows(&self) -> Result<()> {
//         let mut stdout = stdout();
//         execute!(stdout, terminal::Clear(ClearType::All), crossterm::cursor::MoveTo(0, 0))?;
//         for line in &self.content {
//             execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
//             println!("{}\r", line);
//         }
//         Ok(())
//     }

//     fn move_cursor(&self) -> Result<()> {
//         execute!(stdout(), cursor::MoveTo(self.cursor_x as u16, self.cursor_y as u16)).context("Failed moving cursor ")
//     }

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

// fn main() -> Result<()> {
//     let mut editor = MainEditor::new(VecBuffer::default());
//     // editor.run()
// }
