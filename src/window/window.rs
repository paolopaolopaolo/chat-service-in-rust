
use std::{
    cmp,
    io::{Write, stdout, Stdout},
    net::TcpStream, vec,
    sync::{mpsc::Sender, MutexGuard, Arc, Mutex},
    thread,
    time::Duration,
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
        KeyCode,
        read,
        KeyModifiers,
    },
    terminal::{
        SetSize, ClearType, Clear, enable_raw_mode, disable_raw_mode,
    }
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

const MAX_WINDOW_WIDTH: u16 = 85;
const MAX_WINDOW_HEIGHT: u16 = 10;
const H_PADDING: u16 = 2; 
const TL_CORNER: char = '┌';
const TR_CORNER: char = '┐'; 
const BL_CORNER: char = '└';
const BR_CORNER: char = '┘';
const VERT_EDGE: char = '│';
const HORI_EDGE: char = '─';
const LVDIV_EDGE: char = '├';
const RVDIV_EDGE: char = '┤';
const MAX_HLINE_LENGTH: u16 = MAX_WINDOW_WIDTH - 2u16 * H_PADDING;
const MAX_VLINE_LENGTH: u16 = MAX_WINDOW_HEIGHT - 2u16;

pub type SharedChatWindow = Arc<Mutex<ChatWindow>>;

/**
 * When SliceIndex changes values, we want it to "emit" an event with the changed value.
**/
#[derive(Copy, Clone)]
struct SliceIndex {
    from: usize,
    to: usize,
    on_change: fn(&Vec<String>, usize, usize),
}

impl SliceIndex {

    pub fn new(from: usize, to: usize, on_change: fn(&Vec<String>, usize, usize)) -> SliceIndex {
        SliceIndex {
            from,
            to,
            on_change,
        }
    }

    pub fn change(&mut self, text: &Vec<String>, from: usize, to: usize) {
        self.from = from;
        self.to = to;
        (self.on_change)(text, from, to);
    }
}

#[derive(Clone)]
pub struct ChatWindow {
    name: String,
    pub text: Vec<String>,
    current_slice: SliceIndex,
}

pub fn vec_char_to_string(vec_char: Vec<char>) -> String {
    vec_char.iter()
    .map(|x| x.to_string())
    .collect::<String>()
}

pub fn reset_screen(stdout: &mut Stdout) {
    execute!(
        stdout,
        Clear(ClearType::All),
        Clear(ClearType::Purge),
        MoveTo(0, 0),
        SetSize(MAX_WINDOW_WIDTH, MAX_WINDOW_HEIGHT),
    ).unwrap();
}

pub fn println(stdout: &mut Stdout, string: String) {
    queue!(
        stdout,
        Print(string),
        MoveToNextLine(1),
    ).unwrap();
}

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
    bottom_line(stdout);
}

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

fn top_line(stdout: &mut Stdout) {
    let top_bar: String = vec_char_to_string([
        vec![TL_CORNER],
        vec![HORI_EDGE; MAX_HLINE_LENGTH as usize],
        vec![TR_CORNER],
    ].concat());
    println(stdout, top_bar);
}

fn bottom_line(stdout: &mut Stdout) {
    println(stdout, vec_char_to_string([
        vec![BL_CORNER],
        vec![HORI_EDGE; MAX_HLINE_LENGTH as usize],
        vec![BR_CORNER],
    ].concat()));
}

fn empty_line(stdout: &mut Stdout) {
    println(stdout, vec_char_to_string([
        vec![VERT_EDGE],
        vec![' '; MAX_HLINE_LENGTH as usize],
        vec![VERT_EDGE],
    ].concat()));
}

fn split_long_line(text: &String, prefix: &str) -> Vec<String> {
    let MAX_LENGTH = MAX_HLINE_LENGTH as usize - 2 - prefix.len();
    let mut result: Vec<String> = vec![];
    let mut string_clone = text.clone();
    let mut start_idx: usize = 0;
    let mut end_idx: usize = cmp::min(string_clone.len(), MAX_LENGTH);
    let mut current_buffer: String = string_clone[start_idx..end_idx]
        .to_string();
    while current_buffer.len() > usize::MIN {
        if start_idx == 0 {
            result.push(current_buffer);
        } else {
            result.push(format!("{}{}", prefix, current_buffer));
        }
        start_idx = end_idx;
        if end_idx + MAX_LENGTH < string_clone.len() {
            end_idx += MAX_LENGTH;
        } else {
            end_idx = string_clone.len();
        }
        current_buffer = string_clone[start_idx..end_idx]
            .to_string();
    }
    result
}

// Has assumption that all text inputs are less than the MAX length
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

pub enum WindowActions {
    ScrollUp
}

// Blocking call to get a working chat_window
pub fn lock_chat_window<'a>(chat_window: &'a SharedChatWindow) -> MutexGuard<ChatWindow> {
    match chat_window.lock() {
        Ok(lock) => lock,
        _ => panic!("the world is on fire"),
    }
}

/**
 * Behaviors we want:
 * 1. Create a new window (hides everything and instantiates basics)
 * 2. Adding a new line of text
 *  a. If text buffer is full, add the line and "scroll down" a line (start index + 1, end index = end)
 *  b. If text buffer is empty, don't touch current slice.
 * 3. Handle up/down key interactions
 *  a. If up, move start index up one and end index up one
 *  b. If down, move start index down one and end index down one
 *  c. Disable if we're at the top or bottom (start index = 0 or end index = text.len() - 1)
 */
impl ChatWindow {
    pub fn new(name: String) -> ChatWindow {
        execute!(stdout(), Hide);
        // TODO: setup way for us to listen to changes on current_slice
        ChatWindow {
            name: name.clone(),
            text: vec![],
            current_slice: SliceIndex::new(
                0,
                MAX_HLINE_LENGTH as usize,
                print_slice
            ),
        }
    }

    pub fn scroll_up (&mut self) {
        self.current_slice.change(&self.text, self.current_slice.from - 1, self.current_slice.to - 1);
    }

    // TODO: Remove this in favor of pulling text from the ChatBuffer
    pub fn add_chat_line(&mut self, string: String) {
        let lines = split_long_line(&string, "  ");
        lines.iter().for_each(|line| {
            self.text.push(line.clone());
        });
        if self.text.len() < MAX_VLINE_LENGTH as usize {
            // self.print_slice(0, MAX_VLINE_LENGTH as usize);
            self.current_slice.change(&self.text, 0, MAX_VLINE_LENGTH as usize);
        } else {
            // self.print_slice(self.text.len() - MAX_VLINE_LENGTH as usize, self.text.len());
            self.current_slice.change(&self.text, self.text.len() - MAX_VLINE_LENGTH as usize, self.text.len());
        }
    }

    pub fn print (&self) {
        enable_raw_mode().expect("raw mode swap failed");
        let mut stdout = stdout();
        reset_screen(&mut stdout);
        println(&mut stdout, self.name.clone());
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
        stdout.flush();
        disable_raw_mode().expect("raw mode swap failed!");
    }

}

pub struct ChatInput {
    text: String,
    name: String
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
                                    match event.modifiers {
                                        KeyModifiers::CONTROL => {
                                            match event.code {
                                                KeyCode::Char(char) => {
                                                    match char {
                                                        'c' => {
                                                            execute!(stdout(), Clear(ClearType::All)).unwrap();
                                                            disable_raw_mode().expect("error with disable raw mode");
                                                            stream.shutdown(std::net::Shutdown::Both).expect("Shutdown failed");
                                                            println!("CTRL+C to exit");
                                                        },
                                                        _ => {}
                                                    }
                                                }
                                                _ => {},
                                            }
                                        },
                                        _ => {}
                                    }
                                    match event.code {
                                        KeyCode::Char(char) => {
                                            self.text = format!("{}{}", self.text, char);
                                            println_starting_at(
                                                &mut stdout(),
                                                self.text.clone(),
                                                start_at_row,
                                                start_at_column
                                            );
                                        },
                                        KeyCode::Left => {
                                            println_starting_at(&mut stdout(), 
                                                "Left Key Pressed!".to_string(), 
                                                start_at_row + 10, 
                                                start_at_column
                                            );
                                        },
                                        KeyCode::Right => {
                                            println_starting_at(&mut stdout(), 
                                                "Right Key Pressed!".to_string(), 
                                                start_at_row + 10, 
                                                start_at_column
                                            );
                                        },
                                        KeyCode::Up => {
                                            tx.send(WindowActions::ScrollUp).unwrap();
                                            println_starting_at(&mut stdout(), 
                                                "Up Key Pressed!".to_string(), 
                                                start_at_row + 10, 
                                                start_at_column
                                            );
                                        },
                                        KeyCode::Down => {
                                            println_starting_at(&mut stdout(), 
                                                "Down Key Pressed!".to_string(), 
                                                start_at_row + 10, 
                                                start_at_column
                                            );
                                        },
                                        KeyCode::Enter => {
                                            let target_string = format!("\r\n{}: {}\r\n", self.name.clone(), self.text.clone());
                                            stream.write(target_string.as_bytes()).expect("write failed");
                                            self.text = "".to_string();
        
                                            println_starting_at(
                                                &mut stdout(),
                                                self.text.clone(),
                                                start_at_row,
                                                start_at_column
                                            );
                                        },
                                        KeyCode::Backspace => {
                                            self.text = self.text[0..self.text.len() - 1].to_string();
        
                                            println_starting_at(
                                                &mut stdout(),
                                                self.text.clone(),
                                                start_at_row,
                                                start_at_column
                                            );
                                        }
                                        _ => {
                                            // println_starting_at(
                                            //     &mut stdout(),
                                            //     format!("event: {:?}", event),
                                            //     start_at_row,
                                            //     start_at_column
                                            // );
                                        },
                                    }
                                },
                                _ => { break; },
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
