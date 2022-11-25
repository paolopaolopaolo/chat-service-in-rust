
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


/**
 * BOX CHARS
 */

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

/**
 * SIZES (Replace these or use as default sizes)
 */
const MAX_WINDOW_WIDTH: u16 = 65;
const MAX_WINDOW_HEIGHT: u16 = 10;
const H_PADDING: u16 = 2; 

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
    on_change: fn(&Vec<String>, usize, usize, Dimensions),
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
 * Chat Feed UI
 **/
#[derive(Clone)]
pub struct ChatWindow {
    name: String,
    pub text: Vec<String>,
    pub dimensions: Dimensions,
    current_slice: SliceIndex,
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
        enable_raw_mode().expect("raw mode swap failed");
        let mut stdout = stdout();
        reset_screen(&mut stdout, self.dimensions.clone());
        println(&mut stdout, format!(">> You are {}!", self.name.clone()));
        top_line(&mut stdout, self.dimensions.clone());
          self.text
            .iter()
            .enumerate()
            .filter(|(idx, _)| {
                *idx >= self.current_slice.from && *idx < self.current_slice.to
            })
            .for_each(|(_, string_line)| {
            let max_hline_length = self.dimensions.clone().width as u16 - 4;
            let left_padding: u16 = 2;
            let right_padding: u16 = max_hline_length - left_padding - (string_line.len() as u16);
            let text: String = vec_char_to_string([
                vec![VERT_EDGE],
                vec![' '; left_padding as usize],
                string_line.chars().collect::<Vec<char>>(),
                vec![' '; right_padding as usize],
                vec![VERT_EDGE],
            ].concat());
            println(&mut stdout, text);
        });
        let mut lines_left = self.dimensions.clone().height as u16 - 2;
        while lines_left > u16::MIN {
            empty_line(&mut stdout, self.dimensions.clone());
            lines_left -= 1;
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
        disable_raw_mode().expect("raw mode swap failed!");
    }

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
                _ => {}
            }
        }
        self.input_text.clone()
    }

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
pub fn reset_screen(stdout: &mut Stdout, dimensions: Dimensions) {
    execute!(
        stdout,
        Clear(ClearType::All),
        Clear(ClearType::Purge),
        MoveTo(0, 0),
        SetSize(dimensions.width as u16, dimensions.height as u16),
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
        let max_length = dimensions.width - 6 - string.len();
        queue!(
            stdout,
            MoveTo(0, *start_printidx),
            Print(vec_char_to_string(
                [
                    vec![VERT_EDGE],
                    vec![' '; H_PADDING as usize],
                    string.chars().collect(),
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
fn top_line(stdout: &mut Stdout, dimensions: Dimensions) {
    let top_bar: String = vec_char_to_string([
        vec![TL_CORNER],
        vec![HORI_EDGE; dimensions.width - 4 as usize],
        vec![TR_CORNER],
    ].concat());
    println(stdout, top_bar);
}

// Print the bottom line of the chat-input
fn bottom_line(stdout: &mut Stdout, dimensions: Dimensions) {
    println(stdout, vec_char_to_string([
        vec![BL_CORNER],
        vec![HORI_EDGE; dimensions.width - 4],
        vec![BR_CORNER],
    ].concat()));
}

// Print an empty line within the chat-feed
fn empty_line(stdout: &mut Stdout, dimensions: Dimensions) {
    println(stdout, vec_char_to_string([
        vec![VERT_EDGE],
        vec![' '; dimensions.width - 4],
        vec![VERT_EDGE],
    ].concat()));
}

// Split a string that is long into multiple strings
fn split_long_line(text: &String, prefix: &str, dimensions: Dimensions) -> Vec<String> {
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
    text_slice.iter().for_each(|string| {
        printlns(&mut stdout, vec![string.clone()], &mut print_index, dimensions.clone());
    });
    stdout.flush().unwrap();
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
 * Enums
 */

pub enum WindowActions {
    ScrollUp,
    ScrollDown,
    CursorLeft,
    CursorRight,
    Resize(usize, usize),
}

pub struct ChatInput {
    pub text: String,
    pub name: String,
    dimensions: Dimensions
}

impl ChatInput {
    pub fn new(name: String, width: Option<usize>, height: Option<usize>) -> ChatInput {
        let actual_width = match width {
            Some(w) => w,
            _ => MAX_WINDOW_WIDTH as usize
        };
        let actual_height = match height {
            Some(h) => h,
            _ => MAX_WINDOW_HEIGHT as usize
        };
        
        ChatInput {
            text: String::new(),
            name: name.clone(),
            dimensions: Dimensions { width: actual_width, height: actual_height },
        }
    }

    // BLOCKING
    pub fn capture_events(&mut self, socket: &str, tx: Sender<WindowActions>) {
        let mut start_at_row = self.dimensions.height as u16 + 1;
        let mut start_at_column = 0;
        enable_raw_mode().expect("enable raw mode failed");
        let stream = TcpStream::connect(socket);

        match stream {
            Ok(mut stream) => {
                loop {
                    match read() {
                        Ok(ev) => {
                            match ev {
                                Event::Key(event) => {
                                    handle_modified_keys(event.modifiers, event.code, start_at_row, start_at_column, self.dimensions.clone());
                                    handle_key_codes(
                                        self,
                                        event.modifiers,
                                        event.code,
                                        &mut stream,
                                        tx.clone(),
                                        start_at_row,
                                        start_at_column,
                                        self.dimensions.clone()
                                    );
                                },
                                Event::Resize(x, y) => {
                                    tx.clone().send(WindowActions::Resize(x as usize, y as usize - 15)).expect("didn't send resize event");
                                    self.dimensions.width = x as usize;
                                    self.dimensions.height = y as usize;
                                    start_at_row = x;
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
                    start_at_column,
                    self.dimensions.clone()
                );
            },
        }
        disable_raw_mode().expect("disable raw mode failed");
    }
}
