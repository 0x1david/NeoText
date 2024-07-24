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
use buffer::{BufferError, TextBuffer, VecBuffer};
use cursor::{Cursor, LineCol};

/// The main editor is used as the main API for all commands
struct MainEditor<Buff: TextBuffer> {
    /// In the first implementation I will start with Vec, for simplicity, fairly early to the dev
    /// process a better data structure will have to be found and vec replaced;
    cursor: Cursor,
    buffer: Buff,
}

impl<Buff: TextBuffer> MainEditor<Buff> {
    fn pos(&self) -> LineCol {
        self.cursor.pos
    }
    fn go(&mut self, to: LineCol){
        self.cursor.go(to)
    }
    fn delete(&mut self) {
        match self.buffer.delete(self.pos()) {
            Ok(new_pos) => self.go(new_pos),
            Err(BufferError::InvalidPosition) => panic!("Cursor found in a position it should never appear in, please contact the developers."),
            Err(_) => panic!("UnexpectedError, please contact the developers.")
        }
    }
    fn insert(&mut self, c: char) {
        println!("{:?}", self.cursor.pos);
        match self.buffer.insert(self.pos(), c) {
            Ok(new_pos) => self.go(new_pos),
            Err(BufferError::InvalidPosition) => panic!("Cursor found in a position it should never appear in, please contact the developers."),
            Err(_) => panic!("UnexpectedError, please contact the developers.")
        }
    }
}

impl<Buff: TextBuffer> MainEditor<Buff> {
    fn new(buffer: Buff) -> Self {
        MainEditor {
            buffer,
            cursor: Cursor::default(),
            // mode: Modal::default(),
        }
    }

    fn run(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, terminal::Clear(ClearType::All))?;

        loop {
            self.draw_rows()?;
            self.move_cursor()?;

            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char(c) => self.insert(c),
                    // KeyCode::Enter => self.buffer.insert_text(self.pos()),
                    KeyCode::Backspace => self.delete(),
                    KeyCode::Left => self.cursor.bump_left(),
                    KeyCode::Right => self.cursor.bump_right(),
                    KeyCode::Up => self.cursor.bump_up(),
                    KeyCode::Down => self.cursor.bump_down(),
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        };
        Ok(())
    }


//         terminal::disable_raw_mode()?;
//         execute!(stdout, terminal::Clear(ClearType::All))?;
//         Ok(())
//     }

    fn draw_rows(&self) -> Result<()> {
        let mut stdout = stdout();
        execute!(stdout, terminal::Clear(ClearType::All), crossterm::cursor::MoveTo(0, 0))?;
        for line in self.buffer.get_entire_text(){
            execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
            println!("{}\r", line);
        }
        Ok(())
    }

    fn move_cursor(&self) -> Result<()> {
        execute!(stdout(), crossterm::cursor::MoveTo(self.cursor.col() as u16 , self.cursor.line() as u16)).context("Failed moving cursor ")
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

fn main() -> Result<()> {
    let mut editor = MainEditor::new(VecBuffer::default());
    editor.run()
}
