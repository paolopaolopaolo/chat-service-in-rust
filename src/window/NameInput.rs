use std::{
    io::{
        stdout,
        Write,
    }
};
extern crate unicode_width;

use unicode_width::UnicodeWidthStr;

use crossterm::{
    queue,
    cursor::{
        MoveTo,
    },
    terminal::{
        Clear,
        ClearType,
        enable_raw_mode,
        disable_raw_mode
    },
    event::{
        KeyCode,
        read,
        Event,
    }
};
use crate::window::{
    constants::*,
    helpers::*
};

pub struct BasicInputPanel {
    input_text: String
}

impl BasicInputPanel {
    pub fn new() -> BasicInputPanel {
        BasicInputPanel { input_text: String::from("") }
    }

    pub fn print(&mut self) {
        let width: usize = 25;
        let mut stdout = stdout();
        let q = "What is your name?";
        let right_trim = vec!['_'; 10 - UnicodeWidthStr::width(self.input_text.as_str())];
        let a = self.input_text.clone();
        let start_q_at = (width - q.len()) / 2;
        let start_a_at = (width - 10) / 2;
        queue!(
            stdout,
            MoveTo(0, 0),
            Clear(ClearType::All),
        ).unwrap();
        println(&mut stdout, vec_char_to_string([
            vec![TL_CORNER],
            vec![DOBLE_HORI_EDGE; width],
            vec![TR_CORNER],
        ].concat()));
        println(&mut stdout, vec_char_to_string([
            vec![DOBLE_VERT_EDGE],
            vec![' '; width],
            vec![DOBLE_VERT_EDGE],
        ].concat()));
        println(&mut stdout, vec_char_to_string([
            vec![DOBLE_VERT_EDGE],
            vec![' '; width],
            vec![DOBLE_VERT_EDGE],
        ].concat()));
        println(&mut stdout, vec_char_to_string([
            vec![DOBLE_VERT_EDGE],
            vec![' '; start_q_at],
            String::from(q).chars().collect::<Vec<char>>(),
            vec![' '; start_q_at + 1],
            vec![DOBLE_VERT_EDGE],
        ].concat()));
        println(&mut stdout, vec_char_to_string([
            vec![DOBLE_VERT_EDGE],
            vec![' '; width],
            vec![DOBLE_VERT_EDGE],
        ].concat()));
        println(&mut stdout, vec_char_to_string([
            vec![DOBLE_VERT_EDGE],
            vec![' '; start_a_at],
            a.chars().collect::<Vec<char>>(),
            right_trim,
            vec![' '; start_a_at + 1],
            vec![DOBLE_VERT_EDGE],
        ].concat()));
        println(&mut stdout, vec_char_to_string([
            vec![DOBLE_VERT_EDGE],
            vec![' '; width],
            vec![DOBLE_VERT_EDGE],
        ].concat()));
        println(&mut stdout, vec_char_to_string([
            vec![DOBLE_VERT_EDGE],
            vec![' '; width],
            vec![DOBLE_VERT_EDGE],
        ].concat()));
        println(&mut stdout, vec_char_to_string([
            vec![BL_CORNER],
            vec![DOBLE_HORI_EDGE; width],
            vec![BR_CORNER],
        ].concat()));
        stdout.flush().expect("fail");
    }

    pub fn enable_raw(&self) {
        enable_raw_mode().expect("fail");
    }

    pub fn disable_raw(&self) {
        disable_raw_mode().expect("fail");
    }

    pub fn capture_input(&mut self) -> String {
        let max_char_count: usize = 10;
        while let Ok(ev) = read() {
            match ev {
                Event::Key(ev) => {
                    match ev.code {
                        KeyCode::Char(character) => {
                            if self.input_text.len() < max_char_count {
                                self.input_text = format!("{}{}", self.input_text, character);
                            }
                            self.print();
                        },
                        KeyCode::Backspace => {
                            if self.input_text.len() > 0 {
                                self.input_text = self.input_text[0..self.input_text.len() - 1].to_string();
                                self.print();
                            }
                        },
                        KeyCode::Enter => {
                            break;
                        },
                        _ => {}
                    }
                },
                _ => {},
            }
        }
        self.input_text.clone()
    }

}