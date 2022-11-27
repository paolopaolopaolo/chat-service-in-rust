use::std::{
    io::{Stdout, stdout, Write},
    sync::{Arc, Mutex, MutexGuard},
    cmp,
};
use crossterm::{
    execute,
    queue,
    terminal::{
        Clear,
        ClearType,
    },
    cursor::{
        MoveTo,
        MoveToNextLine,
    },
    style::{
        Print,
    },
};

use crate::window::constants::*;
use crate::window::ChatWindow::ChatWindow;

pub type SharedChatWindow = Arc<Mutex<ChatWindow>>;

/**
 * Macro-like methods
 */

// Convert a vector of characters to a string 
pub fn vec_char_to_string(vec_char: Vec<char>) -> String {
    vec_char.iter()
    .map(|x| x.to_string())
    .collect::<String>()
}

// Clear the screen and reset everything
pub fn reset_screen(stdout: &mut Stdout) {
    execute!(
        stdout,
        Clear(ClearType::All),
        Clear(ClearType::Purge),
        MoveTo(0, 0),
    ).unwrap();
}

// Print in place and move down a line
pub fn println(stdout: &mut Stdout, string: String) {
    queue!(
        stdout,
        Print(string),
        MoveToNextLine(1),
    ).unwrap();
}

// Print at a given location, overwriting the line previously
pub fn println_starting_at(stdout: &mut Stdout, string: String, start_at: u16, start_at_col: u16, dimensions: Dimensions) {
    execute!(
        stdout,
        MoveTo(start_at_col, start_at),
        Clear(ClearType::CurrentLine),
        Print(vec_char_to_string([
            vec![VERT_EDGE],
            vec![' '; 2],
            string.chars().collect(),
            vec![' '; dimensions.width - 6 - string.len()],
            vec![VERT_EDGE]
        ].concat())),
        MoveToNextLine(1)
    ).unwrap();
}

// Print multiple lines (from within a chat-feed)
pub fn printlns(stdout: &mut Stdout, strings: Vec<String>, start_printidx: &mut u16, dimensions: Dimensions) {
    strings.iter().for_each(|string| {
        let max_length = match string.len() > dimensions.width - 6 {
            false => dimensions.width - 6 - string.len(),
            true => dimensions.width - 6
        };
        queue!(
            stdout,
            MoveTo(0, *start_printidx),
            Print(vec_char_to_string(
                [
                    vec![VERT_EDGE],
                    vec![' '; H_PADDING as usize],
                    string.chars().collect::<Vec<char>>(),
                    vec![' '; max_length],
                    vec![VERT_EDGE],
                ].concat()
            )),
            MoveToNextLine(1),
        ).expect("Error queueing terminal command.");
        *start_printidx += 1;
    });
    
}

// Print the top line of the chat-feed
pub fn top_line(stdout: &mut Stdout, dimensions: Dimensions) {
    let top_bar: String = vec_char_to_string([
        vec![TL_CORNER],
        vec![HORI_EDGE; dimensions.width - 4 as usize],
        vec![TR_CORNER],
    ].concat());
    println(stdout, top_bar);
}

// Print the bottom line of the chat-input
pub fn bottom_line(stdout: &mut Stdout, dimensions: Dimensions) {
    println(stdout, vec_char_to_string([
        vec![BL_CORNER],
        vec![HORI_EDGE; dimensions.width - 4],
        vec![BR_CORNER],
    ].concat()));
}

// Print an empty line within the chat-feed
pub fn empty_line(stdout: &mut Stdout, dimensions: Dimensions) {
    println(stdout, vec_char_to_string([
        vec![VERT_EDGE],
        vec![' '; dimensions.width - 4],
        vec![VERT_EDGE],
    ].concat()));
}

// Split a string that is long into multiple strings
pub fn split_long_line(text: &String, prefix: &str, dimensions: Dimensions) -> Vec<String> {
    let max_length = dimensions.width - 6 - prefix.len();
    let mut result: Vec<String> = vec![];
    let string_clone = text.clone();
    let mut start_idx: usize = 0;
    let mut end_idx: usize = cmp::min(string_clone.len(), max_length);
    let mut current_buffer: String = string_clone[start_idx..end_idx]
        .to_string();
    while current_buffer.len() > usize::MIN {
        if start_idx == 0 {
            result.push(current_buffer);
        } else {
            result.push(format!("{}{}", prefix, current_buffer));
        }
        start_idx = end_idx;
        if end_idx + max_length < string_clone.len() {
            end_idx += max_length;
        } else {
            end_idx = string_clone.len();
        }
        current_buffer = string_clone[start_idx..end_idx]
            .to_string();
    }
    result
}

// Implement simple overflow. Return latest visible 
// slice of string if the string is too long.
pub fn adjust_text_for_overflow(copy: String, dimensions: Dimensions) -> String {
    let max_input_length = dimensions.width - 6;
    let mut text_to_print = copy.clone();
    if text_to_print.len() > max_input_length {
        let start_index = text_to_print.len() - max_input_length;
        text_to_print = text_to_print[start_index..text_to_print.len()].to_string();
    }
    text_to_print
}


// Blocking call to get a Mutex-locked Chat-Feed struct
pub fn lock_chat_window(chat_window: &SharedChatWindow) -> MutexGuard<ChatWindow> {
    let mut result = None;
    while result.is_none() {
        match chat_window.try_lock() {
            Ok(cw) => { result = Some(cw); },
            _ => {},
        }
    }
    // This works because the loop above will repeat otherwise
    result.unwrap()
}

/**
 * An Index Slice that runs an on-change function when it changes
 **/
 #[derive(Copy, Clone)]
 pub struct SliceIndex {
     pub from: usize,
     pub to: usize,
     pub on_change: fn(&Vec<String>, usize, usize, Dimensions),
 }
 
 impl SliceIndex {   
      // Instantiates new SliceIndex
     pub fn new(from: usize, to: usize, on_change: fn(&Vec<String>, usize, usize, Dimensions)) -> SliceIndex {
         SliceIndex {
             from,
             to,
             on_change,
         }
     }
 
     // Change the index with this method to trigger on_change
     pub fn change(&mut self, text: &Vec<String>, from: usize, to: usize, dimensions: Dimensions) {
         self.from = from;
         self.to = to;
         (self.on_change)(text, from, to, dimensions);
     }
 }
 
 #[derive(Copy, Clone)]
 pub struct Dimensions {
     pub width: usize,
     pub height: usize,
 }
 
 /**
 * Enums
 */

pub enum WindowActions {
    ScrollUp,
    ScrollDown,
    CursorLeft,
    CursorRight,
    Resize(usize, usize),
}

