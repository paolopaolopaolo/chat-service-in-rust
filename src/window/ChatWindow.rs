
use std::{
    io::{Write, stdout},
    vec,
};

use crate::window::{constants::*, helpers::*};

use crossterm::{
    execute,
    cursor::{
        Hide,
    },
};


/**
 * Chat Window UI
 * 
 * ┌────────────────────────────────────────────────────────────────────────────────────┐
 * │  (name)> H_PADDING = 2 chars; V_PADDING = 1;                                       │                              
 * ├────────────────────────────────────────────────────────────────────────────────────┤
 * │  <user-input text appears here>                                                    │
 * └────────────────────────────────────────────────────────────────────────────────────┘
 */


/**
 * Chat Feed UI
 **/
#[derive(Clone)]
pub struct ChatWindow {
    name: String,
    pub text: Vec<String>,
    pub dimensions: Dimensions,
    pub current_slice: SliceIndex,
}

/**
 * Chat feed with the following
 * 1. Creates a new window (hides everything and instantiates basics)
 * 2. Adds a new line of text
 *  a. If text buffer is full, adds the line and "scrolls down" by shifting the current slice downwards
 *  b. If text buffer is not full, the current slice is maintained.
 * 3. Handle up/down key interactions
 *  a. If up, move start index up one and end index up one
 *  b. If down, move start index down one and end index down one
 *  c. Disable if we're at the top or bottom (start index = 0 or end index = text.len() - 1)
 */
impl ChatWindow {
    pub fn new(name: String, width: Option<usize>, height: Option<usize>) -> ChatWindow {
        execute!(stdout(), Hide).expect("bad things happened");
        let window_width = match width {
            Some(w) => w,
            _ => MAX_WINDOW_WIDTH as usize,
        };
        let window_height: usize = match height {
            Some(h) => h,
            _ => MAX_WINDOW_HEIGHT as usize,
        };
        ChatWindow {
            name: name.clone(),
            text: vec![],
            current_slice: SliceIndex::new(
                0,
                window_width - 2,
                print_slice
            ),
            dimensions: Dimensions { width: window_width, height: window_height }
        }
    }

    /**
     * Window actions
     */

    pub fn scroll_up (&mut self) {
        if self.current_slice.from - 1 > usize::MIN {
            self.current_slice.change(&self.text, self.current_slice.from - 1, self.current_slice.to - 1, self.dimensions.clone());
        }
    }

    pub fn scroll_down (&mut self) {
        if self.current_slice.to + 1 < self.text.len() {
            self.current_slice.change(&self.text, self.current_slice.from + 1, self.current_slice.to + 1, self.dimensions.clone());
        }
    }

    /**
     * Chat Feed Actions
     */

     pub fn add_chat_line(&mut self, string: String) {
        let lines = split_long_line(&string, "  ", self.dimensions.clone());
        lines.iter().for_each(|line| {
            self.text.push(line.clone());
        });
        let max_height = self.dimensions.height - 2;
        if self.text.len() < self.dimensions.height {
            self.current_slice.change(&self.text, 0, max_height, self.dimensions.clone());
        } else {
            self.current_slice.change(&self.text, self.text.len() - max_height, self.text.len(), self.dimensions.clone());
        }
    }

    pub fn print (&self) {
        let mut stdout = stdout();
        reset_screen(&mut stdout);
        println(&mut stdout, format!(">> You are {}!", self.name.clone()));
        top_line(&mut stdout, self.dimensions.clone());
        for _ in  0..self.dimensions.clone().height - 2 {
            empty_line(&mut stdout, self.dimensions.clone());
        }
        println(
            &mut stdout,
            vec_char_to_string([
                vec![LVDIV_EDGE],
                vec![HORI_EDGE; self.dimensions.width - 4],
                vec![RVDIV_EDGE],
            ].concat())
        );
        empty_line(&mut stdout, self.dimensions.clone());
        bottom_line(&mut stdout, self.dimensions.clone());
        stdout.flush().unwrap_or_else(|_| { println!("stout flush failed"); });
    }

}

// Print within a chatfeed with assumption that all text 
// inputs are less than the MAX length
fn print_slice(text: &Vec<String>, start: usize, end: usize, dimensions: Dimensions) {
    let mut actual_end = text.len();
    if end < actual_end {
        actual_end = end;
    }
    let text_slice = &text[start..actual_end];
    let mut stdout = stdout();
    let mut print_index = 2u16;
    printlns(&mut stdout, text_slice.to_vec(), &mut print_index, dimensions.clone());
    stdout.flush().unwrap();
}
