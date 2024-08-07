use std::process::exit;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::{
    bars::{draw_bar, get_info_bar_content, get_notif_bar_content, INFO_BAR, NOTIFICATION_BAR},
    buffer::TextBuffer,
    cursor::LineCol,
    editor::Editor,
    notif_bar, repeat, Result,
};

const SCROLL_JUMP_DISTANCE: usize = 25;

use super::{FindMode, Modal};

impl<Buff: TextBuffer> Editor<Buff> {
    pub(crate) fn run_normal(
        &mut self,
        carry_over: Option<i32>,
        prev_char: Option<char>,
    ) -> Result<()> {
        self.draw_lines()?;
        draw_bar(&INFO_BAR, |term_width, _| {
            get_info_bar_content(term_width, &self.mode, &self.pos())
        })?;
        draw_bar(&NOTIFICATION_BAR, |_, _| get_notif_bar_content())?;
        self.move_cursor();
        self.force_within_bounds();

        if let Event::Key(key_event) = event::read()? {
            match (key_event.code, key_event.modifiers) {
                (KeyCode::Char(ch), mods) => {
                    if let Some(prev) = prev_char {
                        self.handle_combination_input(ch, carry_over, prev)?;
                    } else if !(key_event.modifiers.is_empty()
                        || (mods == KeyModifiers::SHIFT && ch.is_alphabetic()))
                    {
                        self.handle_modifiers(ch, carry_over, mods)?;
                    } else {
                        self.handle_char_input(ch, carry_over)?;
                    }
                }
                (KeyCode::End, _) => self.move_to_end_of_line(),
                (KeyCode::Home, _) => self.move_to_first_col(),
                (KeyCode::Esc, _) => exit(0),
                _ => {
                    notif_bar!("nothing");
                }
            }
        }

        Ok(())
    }
    fn handle_combination_input(
        &mut self,
        ch: char,
        carry_over: Option<i32>,
        prev: char,
    ) -> Result<()> {
        match (prev, ch) {
            ('d', 'd') => repeat!(self.buffer.delete_line(self.pos().line); carry_over),
            ('g', 'g') => {
                let col = self.pos().col;
                self.go(LineCol { line: 0, col });
            }
            ('t', pat) => self.move_to_char(pat)?,
            ('T', pat) => self.move_back_to_char(pat)?,
            ('f', pat) => self.find_next_char(pat, carry_over)?,
            ('F', pat) => self.find_previous_char(pat, carry_over)?,
            ('r', pat) => self.replace_under_cursor(pat)?,
            (_, _) => {
                notif_bar!("nothing");
            }
        }
        Ok(())
    }
    fn find_next_char(&mut self, pat: char, carry_over: Option<i32>) -> Result<()> {
        repeat! {{
            let mut pos = self.pos();
            if self.buffer.max_col(pos) > pos.col + 1 {
                pos.col += 1;
            };

            self.go(self.buffer.find(pat, pos)?);
        }; carry_over}
        Ok(())
    }

    fn find_previous_char(&mut self, pat: char, carry_over: Option<i32>) -> Result<()> {
        repeat! {{
            self.go(self.buffer.rfind(pat, self.pos())?);
        }; carry_over}
        Ok(())
    }
    fn move_to_char(&mut self, pat: char) -> Result<()> {
        let dest = self.buffer.find(pat, self.pos())?;
        self.go(dest);
        let mut dest = self.pos();
        dest.col -= 1;
        self.go(dest);
        Ok(())
    }

    fn move_back_to_char(&mut self, pat: char) -> Result<()> {
        let dest = self.buffer.rfind(pat, self.pos())?;
        self.go(dest);
        let mut dest = self.pos();
        dest.col += 1;
        self.go(dest);
        Ok(())
    }
    /// Unnecessary until redo and scrolling
    fn handle_modifiers(
        &mut self,
        ch: char,
        carry_over: Option<i32>,
        modifiers: KeyModifiers,
    ) -> Result<()> {
        if modifiers.contains(KeyModifiers::CONTROL) {
            match ch {
                'd' => {repeat!{{
                    self.cursor.jump_down(SCROLL_JUMP_DISTANCE);
                    self.center_view_window();
                }; carry_over
                }}
                'u' => {repeat!{{
                    self.cursor.jump_up(SCROLL_JUMP_DISTANCE);
                    self.center_view_window();
                }; carry_over
                }},
                _ => ()
            }
        };
        Ok(())
    }
    fn handle_char_input(&mut self, ch: char, carry_over: Option<i32>) -> Result<()> {
        match ch {
            combination @ ('r' | 't' | 'd' | 'y' | 'z' | 'f' | 'g' | 'F' | 'T') => {
                self.run_normal(carry_over, Some(combination))?;
            }
            'i' => self.set_mode(Modal::Insert),
            'o' => {
                self.set_mode(Modal::Insert);
                self.newline();
            }
            ':' => self.set_mode(Modal::Command),
            '/' => self.set_mode(Modal::Find(FindMode::Forwards)),
            '?' => self.set_mode(Modal::Find(FindMode::Backwards)),
            'h' => repeat!(self.cursor.bump_left(); carry_over),
            'l' => repeat!(self.cursor.bump_right(); carry_over),
            'k' => repeat!(self.cursor.bump_up(); carry_over),
            'j' => repeat!(self.cursor.bump_down(); carry_over),
            'W' => repeat!(self.move_to_next_word_after_whitespace()?; carry_over),
            'w' => repeat!(self.move_to_next_non_alphanumeric()?; carry_over),
            'G' => self.move_to_lowest_line(),
            'x' => self.delete_under_cursor()?,
            'X' => self.delete_before_cursor()?,
            'A' => self.move_to_end_of_line_and_insert(),
            '_' => self.move_to_first_non_whitespace_col()?,
            '$' => self.move_to_end_of_line(),
            '0'..='9' => self.handle_number_input(ch, carry_over),
            _ => {
                notif_bar!("nothing");
            }
        }
        Ok(())
    }
    fn replace_under_cursor(&mut self, ch: char) -> Result<()> {
        self.delete_under_cursor()?;
        self.push(ch);
        Ok(())
    }
    fn delete_under_cursor(&mut self) -> Result<()> {
        let mut delete_dest = self.pos();
        delete_dest.col += 1;
        let dest = self.buffer.delete(delete_dest)?;
        self.go(dest);
        Ok(())
    }
    fn delete_before_cursor(&mut self) -> Result<()> {
        let dest = self.buffer.delete(self.pos())?;
        self.go(dest);
        Ok(())
    }
    fn move_to_end_of_line_and_insert(&mut self) {
        self.move_to_end_of_line();
        self.set_mode(Modal::Insert);
    }
    fn move_to_lowest_line(&mut self) {
        let mut pos = self.pos();
        let dest = self.buffer.max_line();
        pos.line = dest;
        self.go(pos);
    }
    fn move_to_end_of_line(&mut self) {
        let mut pos = self.pos();
        let dest = self.buffer.max_col(pos);
        pos.col = dest;
        self.go(pos);
    }
    fn move_to_first_col(&mut self) {
        let mut pos = self.pos();
        pos.col = 0;
        self.go(pos);
    }
    fn move_to_first_non_whitespace_col(&mut self) -> Result<()> {
        let mut pos = self.pos();
        pos.col = 0;
        let dest = self.buffer.find(|ch| !char::is_whitespace(ch), pos)?;
        self.go(dest);
        Ok(())
    }
    fn move_to_next_word_after_whitespace(&mut self) -> Result<()> {
        let mut pos = self.pos();
        if self.buffer.max_col(pos) > pos.col {
            pos.col += 1;
        };

        let mut dest = self.buffer.find(char::is_whitespace, pos)?;
        dest = self.buffer.find(|ch| !char::is_whitespace(ch), dest)?;
        self.go(dest);
        Ok(())
    }

    fn move_to_next_non_alphanumeric(&mut self) -> Result<()> {
        let mut pos = self.pos();
        if self.buffer.max_col(pos) > pos.col {
            pos.col += 1;
        };

        let mut dest = self.buffer.find(|ch| !char::is_whitespace(ch), pos)?;
        dest = self.buffer.find(|ch| !char::is_alphanumeric(ch), dest)?;
        self.go(dest);
        Ok(())
    }
    fn handle_number_input(&mut self, num: char, carry_over: Option<i32>) {
        let digit = i32::from(num as u8 - b'0');
        let new_carry_over = carry_over.map_or(digit, |current_carry_over| {
            concatenate_ints(current_carry_over, digit)
        });
        let _ = self.run_normal(Some(new_carry_over), None);
    }
}

pub fn concatenate_ints(a: i32, b: i32) -> i32 {
    format!("{a}{b}").parse().unwrap_or(a)
}
