use std::process::exit;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::{
    bars::{draw_bar, get_info_bar_content, get_notif_bar_content, INFO_BAR, NOTIFICATION_BAR},
    buffer::TextBuffer,
    cursor::Selection,
    editor::Editor,
    error::Error,
    notif_bar, repeat, LineCol, Result,
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
        let pos = self.pos();
        draw_bar(&mut self.viewport.terminal, &INFO_BAR, |term_width, _| {
            get_info_bar_content(term_width, &self.mode, pos)
        })?;
        draw_bar(&mut self.viewport.terminal, &NOTIFICATION_BAR, |_, _| {
            get_notif_bar_content()
        })?;
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
                        self.handle_modifiers(ch, carry_over, mods);
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
    pub fn handle_combination_input(
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
    pub fn handle_modifiers(&mut self, ch: char, carry_over: Option<i32>, modifiers: KeyModifiers) {
        if modifiers.contains(KeyModifiers::CONTROL) {
            match ch {
                'd' => {
                    repeat! {{
                        self.cursor.jump_down(SCROLL_JUMP_DISTANCE, self.buffer.max_line());
                        self.viewport.center(self.pos());
                    }; carry_over
                    }
                }
                'u' => {
                    repeat! {{
                        self.cursor.jump_up(SCROLL_JUMP_DISTANCE);
                        self.viewport.center(self.pos());
                    }; carry_over
                    }
                }
                _ => (),
            }
        }
    }
    pub fn handle_char_input(&mut self, ch: char, carry_over: Option<i32>) -> Result<()> {
        match ch {
            combination @ ('r' | 't' | 'd' | 'z' | 'f' | 'g' | 'F' | 'T') => {
                if combination == 'd' && self.mode.is_any_visual() {
                    let sel = Selection::from(&self.cursor).normalized();

                    let dest = self.buffer.delete_selection(sel.start, sel.end)?;
                    self.cursor.pos = dest;
                    self.set_mode(Modal::Normal)
                }
                self.run_normal(carry_over, Some(combination))?;
            }
            'y' => {
                if self.mode.is_any_visual() {
                    let sel = self.buffer.get_buffer_window(
                        Some(self.cursor.last_text_mode_pos),
                        Some(self.pos()),
                    )?;
                    let sel = if self.mode.is_visual_line() {
                        format!("\n{}", sel.join("\n"))
                    } else {
                        sel.join("\n").to_string()
                    };
                    let chars: Vec<char> = sel.chars().collect();
                    self.copy_register.yank(chars, None)?;
                    self.set_mode(Modal::Normal)
                }
            }
            'i' => {
                if !self.mode.is_any_visual() {
                    self.set_mode(Modal::Insert)
                }
            }
            'p' => self.paste_register_content(None, false)?,
            'P' => self.paste_register_content(None, true)?,
            'o' => {
                self.set_mode(Modal::Insert);
                self.newline();
            }
            ':' => self.set_mode(Modal::Command),
            'v' => self.set_mode(Modal::Visual),
            'V' => self.set_mode(Modal::VisualLine),
            '/' => self.set_mode(Modal::Find(FindMode::Forwards)),
            '?' => self.set_mode(Modal::Find(FindMode::Backwards)),
            'h' => repeat!(self.cursor.bump_left(); carry_over),
            'l' => repeat!(self.cursor.bump_right(); carry_over),
            'k' => repeat!(self.cursor.bump_up(); carry_over),
            'j' => repeat!(self.cursor.bump_down(); carry_over),
            'J' => {
                if self.mode.is_any_visual() {
                    // Add Join Lines
                }
            }
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
    fn paste_register_content(&mut self, register: Option<char>, newline: bool) -> Result<()> {
        let register_content = self.copy_register.get_from_register(register)?;
        let mut pos = self.pos();
        pos.line -= 1;
        let dest =
            self.buffer
                .insert_text(self.pos(), String::from_iter(register_content), newline);
        let dest = match dest {
            Err(Error::InvalidInput) => {
                notif_bar!("Register empty.");
                self.pos()
            }
            otherwise => otherwise?,
        };
        self.go(dest);
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
    pub fn move_to_end_of_line(&mut self) {
        let mut pos = self.pos();
        let dest = self.buffer.max_col(pos);
        pos.col = dest;
        self.go(pos);
    }
    pub fn move_to_first_col(&mut self) {
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
