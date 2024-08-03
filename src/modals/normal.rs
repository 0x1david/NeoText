use std::process::exit;

use crossterm::event::{self, Event, KeyCode};

use crate::{bars::{draw_bar, get_notif_bar_content, INFO_BAR, NOTIFICATION_BAR}, buffer::TextBuffer, cursor::Cursor, editor:: Editor, notif_bar, repeat, Result};

use super::{FindMode, Modal};

impl<Buff: TextBuffer> Editor<Buff> {
    pub(crate) fn run_normal(&mut self, carry_over: Option<i32>) -> Result<()> {
        self.draw_rows()?;
        draw_bar(&INFO_BAR, |term_width, _| {
            self.get_info_bar_content(term_width)
        })?;
        draw_bar(&NOTIFICATION_BAR, |_, _| get_notif_bar_content())?;
        self.move_cursor();


        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char(ch) => self.handle_char_input(ch, carry_over)?,
                KeyCode::Esc => exit(0),
                _ => {notif_bar!("nothing");},
            }
        }

        Ok(())
    }
    fn handle_char_input(&mut self, ch: char, carry_over: Option<i32>) -> Result<()> {
        match ch {
            'i' => self.set_mode(Modal::Insert),
            'o' => {
                self.set_mode(Modal::Insert);
                self.newline();
            }
            ':' => self.set_mode(Modal::Command),
            '/' => self.set_mode(Modal::Find(FindMode::Forwards)),
            '?' => self.set_mode(Modal::Find(FindMode::Backwards)),
            'h' => repeat!(self.if_within_bounds(Cursor::bump_left); carry_over),
            'l' => repeat!(self.if_within_bounds(Cursor::bump_right); carry_over),
            'k' => repeat!(self.if_within_bounds(Cursor::bump_up); carry_over),
            'j' => repeat!(self.if_within_bounds(Cursor::bump_down); carry_over),
            'W' => repeat!(self.move_to_next_word_after_whitespace()?; carry_over),
            'w' => repeat!(self.move_to_next_non_alphanumeric()?; carry_over),
            '0'..='9' => self.handle_number_input(ch, carry_over),
            _ => {notif_bar!("nothing");},
        }
        Ok(())
    }
    fn move_to_next_word_after_whitespace(&mut self) -> Result<()> {
        let mut pos = self.pos();
        pos.col += 1;
        let mut dest = self.buffer.find(char::is_whitespace, pos)?;
        dest = self.buffer.find(|ch| !char::is_whitespace(ch), dest)?;
        notif_bar!(dest);
        self.go(dest);
        Ok(())
    }

    fn move_to_next_non_alphanumeric(&mut self) -> Result<()> {
        let mut pos = self.pos();
        pos.col += 1;
        let dest = self.buffer.find(|ch| !char::is_alphanumeric(ch), pos)?;
        self.go(dest);
        Ok(())
    }
    fn handle_number_input(&mut self, num: char, carry_over: Option<i32>) {
        let digit = i32::from(num as u8 - b'0');
        let new_carry_over = carry_over.map_or(
            digit,
            |current_carry_over| concatenate_ints(current_carry_over, digit)
        );
        let _ = self.run_normal(Some(new_carry_over));
    }
}


pub fn concatenate_ints(a: i32, b: i32) -> i32 {
    format!("{a}{b}").parse().unwrap()
}
