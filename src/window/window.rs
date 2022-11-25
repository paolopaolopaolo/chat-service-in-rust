
use std::{
    cmp,
    io::{Write, stdout, Stdout},
    net::TcpStream, vec,
    sync::{mpsc::Sender, MutexGuard, Arc, Mutex}
};

use crossterm::{
    execute,
    queue,
    style::{
        Print,
    },
    cursor::{
        Hide,
        MoveTo,
        MoveToNextLine,
    },
    event::{
        Event,
        read,
        KeyCode,
    },
    terminal::{
        SetSize, ClearType, Clear, enable_raw_mode, disable_raw_mode,
    }
};

use super::handlers::{handle_modified_keys, handle_key_codes};

/**
 * Chat Window UI
 * 
 * ┌────────────────────────────────────────────────────────────────────────────────────┐
 * │  (name)> H_PADDING = 2 chars; V_PADDING = 1;                                       │                              
 * ├────────────────────────────────────────────────────────────────────────────────────┤
 * │  <user-input text appears here>                                                    │
 * └────────────────────────────────────────────────────────────────────────────────────┘
 */

const MAX_WINDOW_WIDTH: u16 = 65;
const MAX_WINDOW_HEIGHT: u16 = 10;
const H_PADDING: u16 = 2; 
const TL_CORNER: char = '┌';
const TR_CORNER: char = '┐'; 
const BL_CORNER: char = '└';
const BR_CORNER: char = '┘';
const VERT_EDGE: char = '│';
const HORI_EDGE: char = '─';
const DOBLE_HORI_EDGE: char = '═';
const DOBLE_VERT_EDGE: char = '║';
const LVDIV_EDGE: char = '├';
const RVDIV_EDGE: char = '┤';
const MAX_HLINE_LENGTH: u16 = MAX_WINDOW_WIDTH - 2u16 * H_PADDING;
const MAX_VLINE_LENGTH: u16 = MAX_WINDOW_HEIGHT - 2u16;

/**
 * Types
 */

pub type SharedChatWindow = Arc<Mutex<ChatWindow>>;

/**
 * An Index Slice that runs an on-change function when it changes
 **/
#[derive(Copy, Clone)]
struct SliceIndex {
    from: usize,
    to: usize,
    on_change: fn(&Vec<String>, usize, usize),
}

impl SliceIndex {
    // Instantiates new SliceIndex
    pub fn new(from: usize, to: usize, on_change: fn(&Vec<String>, usize, usize)) -> SliceIndex {
        SliceIndex {
            from,
            to,
            on_change,
        }
    }

    // Change the index with this method to trigger on_change
    pub fn change(&mut self, text: &Vec<String>, from: usize, to: usize) {
        self.from = from;
        self.to = to;
        (self.on_change)(text, from, to);
    }
}

#[derive(Copy, Clone)]
struct Dimensions {
    width: usize,
    height: usize,
}


/**
 * Chat Feed UI
 **/
#[derive(Clone)]
pub struct ChatWindow {
    name: String,
    pub text: Vec<String>,
    dimensions: Dimensions,
    current_slice: SliceIndex,
}

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
        SetSize(MAX_WINDOW_WIDTH, MAX_WINDOW_HEIGHT),
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
pub fn println_starting_at(stdout: &mut Stdout, string: String, start_at: u16, start_at_col: u16) {
    execute!(
        stdout,
        MoveTo(start_at_col, start_at),
        Clear(ClearType::CurrentLine),
        Print(vec_char_to_string([
            vec![VERT_EDGE],
            vec![' '; 2],
            string.chars().collect(),
            vec![' '; MAX_HLINE_LENGTH as usize - 2 - string.len()],
            vec![VERT_EDGE]
        ].concat())),
        MoveToNextLine(1)
    ).unwrap();
}

// Print multiple lines (from within a chat-feed)
pub fn printlns(stdout: &mut Stdout, strings: Vec<String>, start_printidx: &mut u16) {
    strings.iter().for_each(|string| {
        queue!(
            stdout,
            MoveTo(0, *start_printidx),
            Print(vec_char_to_string(
                [
                    vec![VERT_EDGE],
                    vec![' '; H_PADDING as usize],
                    string.chars().collect(),
                    vec![' '; MAX_HLINE_LENGTH as usize - 2 - string.len()],
                    vec![VERT_EDGE],
                ].concat()
            )),
            MoveToNextLine(1),
        ).expect("Error queueing terminal command.");
        *start_printidx += 1;
    });
    
}

// Print the top line of the chat-feed
fn top_line(stdout: &mut Stdout) {
    let top_bar: String = vec_char_to_string([
        vec![TL_CORNER],
        vec![HORI_EDGE; MAX_HLINE_LENGTH as usize],
        vec![TR_CORNER],
    ].concat());
    println(stdout, top_bar);
}

// Print the bottom line of the chat-input
fn bottom_line(stdout: &mut Stdout) {
    println(stdout, vec_char_to_string([
        vec![BL_CORNER],
        vec![HORI_EDGE; MAX_HLINE_LENGTH as usize],
        vec![BR_CORNER],
    ].concat()));
}

// Print an empty line within the chat-feed
fn empty_line(stdout: &mut Stdout) {
    println(stdout, vec_char_to_string([
        vec![VERT_EDGE],
        vec![' '; MAX_HLINE_LENGTH as usize],
        vec![VERT_EDGE],
    ].concat()));
}

// Split a string that is long into multiple strings
fn split_long_line(text: &String, prefix: &str) -> Vec<String> {
    let max_length = MAX_HLINE_LENGTH as usize - 2 - prefix.len();
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

// Print within a chatfeed with assumption that all text 
// inputs are less than the MAX length
fn print_slice(text: &Vec<String>, start: usize, end: usize) {
    let mut actual_end = text.len();
    if end < actual_end {
        actual_end = end;
    }
    let text_slice = &text[start..actual_end];
    let mut stdout = stdout();
    let mut print_index = 2u16;
    text_slice.iter().for_each(|string| {
        printlns(&mut stdout, vec![string.clone()], &mut print_index);
    });
    stdout.flush().unwrap();
}

// Implement simple overflow. Return latest visible 
// slice of string if the string is too long.
pub fn adjust_text_for_overflow(copy: String) -> String {
    let max_input_length = MAX_HLINE_LENGTH as usize - 4;
    let mut text_to_print = copy.clone();
    if text_to_print.len() > max_input_length {
        let start_index = text_to_print.len() - max_input_length;
        text_to_print = text_to_print[start_index..text_to_print.len()].to_string();
    }
    text_to_print
}


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
        let right_trim = vec!['_'; 10 - &self.input_text.len()];
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
        loop {
            match read() {
                Ok(ev) => {
                    match ev {
                        Event::Key(ev) => {
                            match ev.code {
                                KeyCode::Char(character) => {
                                    if self.input_text.len() < 10 {
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
                _ => {}
            }
        }
        self.input_text.clone()
    }

}

pub fn name_enter_capture() {

}


// Blocking call to get a Mutex-locked Chat-Feed struct
pub fn lock_chat_window(chat_window: &SharedChatWindow) -> MutexGuard<ChatWindow> {
    let mut result = None;
    let mut count = 0;
    while result.is_none() {
        match chat_window.try_lock() {
            Ok(cw) => { result = Some(cw); },
            _ => {
                println_starting_at(&mut stdout(), format!("looping: {}", count), 25, 0);
                count += 1;
            },
        }
    }
    // This works because the loop above will repeat otherwise
    result.unwrap()
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
    pub fn new(name: String) -> ChatWindow {
        execute!(stdout(), Hide).expect("bad things happened");
        // TODO: setup way for us to listen to changes on current_slice
        ChatWindow {
            name: name.clone(),
            text: vec![],
            current_slice: SliceIndex::new(
                0,
                MAX_HLINE_LENGTH as usize,
                print_slice
            ),
            dimensions: Dimensions { width: MAX_WINDOW_WIDTH as usize, height: MAX_WINDOW_HEIGHT as usize }
        }
    }

    /**
     * Window actions
     */

    pub fn scroll_up (&mut self) {
        if self.current_slice.from - 1 > usize::MIN {
            self.current_slice.change(&self.text, self.current_slice.from - 1, self.current_slice.to - 1);
        }
    }

    pub fn scroll_down (&mut self) {
        if self.current_slice.to + 1 < self.text.len() {
            self.current_slice.change(&self.text, self.current_slice.from + 1, self.current_slice.to + 1);
        }
    }

    // TODO: Remove this in favor of pulling text from the ChatBuffer
    pub fn add_chat_line(&mut self, string: String) {
        let lines = split_long_line(&string, "  ");
        lines.iter().for_each(|line| {
            self.text.push(line.clone());
        });
        if self.text.len() < MAX_VLINE_LENGTH as usize {
            self.current_slice.change(&self.text, 0, MAX_VLINE_LENGTH as usize);
        } else {
            self.current_slice.change(&self.text, self.text.len() - MAX_VLINE_LENGTH as usize, self.text.len());
        }
    }

    pub fn print (&self) {
        enable_raw_mode().expect("raw mode swap failed");
        let mut stdout = stdout();
        reset_screen(&mut stdout);
        println(&mut stdout, format!(">> You are {}!", self.name.clone()));
        top_line(&mut stdout);
  
        let get_text_by_line = |string_line: &String| {
            let left_padding: u16 = 2;
            let right_padding: u16 = MAX_HLINE_LENGTH - left_padding - (string_line.len() as u16);
            let text: String = vec_char_to_string([
                vec![VERT_EDGE],
                vec![' '; left_padding as usize],
                string_line.chars().collect::<Vec<char>>(),
                vec![' '; right_padding as usize],
                vec![VERT_EDGE],
            ].concat());
            println(&mut stdout, text);
        };
        self.text.iter().for_each(get_text_by_line);
        let mut lines_left = MAX_VLINE_LENGTH;
        while lines_left > u16::MIN {
            empty_line(&mut stdout);
            lines_left -= 1;
        }
        println(
            &mut stdout,
            vec_char_to_string([
                vec![LVDIV_EDGE],
                vec![HORI_EDGE; MAX_HLINE_LENGTH as usize],
                vec![RVDIV_EDGE],
            ].concat())
        );
        empty_line(&mut stdout);
        bottom_line(&mut stdout);
        stdout.flush().unwrap_or_else(|_| { println!("stout flush failed"); });
        disable_raw_mode().expect("raw mode swap failed!");
    }

}

pub struct ChatInput {
    pub text: String,
    pub name: String
}

impl ChatInput {
    pub fn new(name: String) -> ChatInput {
        ChatInput {
            text: String::new(),
            name: name.clone(),
        }
    }

    // BLOCKING
    pub fn capture_events(&mut self, socket: &str, tx: Sender<WindowActions>) {
        let start_at_row = MAX_VLINE_LENGTH + 3;
        let start_at_column = 0;
        enable_raw_mode().expect("enable raw mode failed");
        let stream = TcpStream::connect(socket);

        match stream {
            Ok(mut stream) => {
                loop {
                    match read() {
                        Ok(ev) => {
                            match ev {
                                Event::Key(event) => {
                                    handle_modified_keys(event.modifiers, event.code, start_at_row, start_at_column);
                                    handle_key_codes(
                                        self,
                                        event.modifiers,
                                        event.code,
                                        &mut stream,
                                        tx.clone(),
                                        start_at_row,
                                        start_at_column
                                    );
                                },
                                Event::Resize(x, y) => {
                                    tx.clone().send(WindowActions::Resize(x as usize, y as usize)).expect("didn't send resize event");
                                },
                                _ => { },
                            }
                        },
                        _ => {},
                    }
                }
            },
            Err(err) => {  
                println_starting_at(
                    &mut stdout(),
                    format!("socket_failed: {:?}", err),
                    start_at_row + 10,
                    start_at_column
                );
            },
        }
        disable_raw_mode().expect("disable raw mode failed");
    }
}
