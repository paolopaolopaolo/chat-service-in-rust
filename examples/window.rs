
use std::io::{Write, stdout};
use crossterm::{
    execute, 
    terminal::{
        SetSize, ClearType, Clear, ScrollUp
    }
};

/**
 * Chat Window UI
 * 
 * ┌────────────────────────────────────────────────────────────────────────────────────┐
 * │  (name)> H_PADDING = 2 chars; V_PADDING = 1;                                       │                              
 * ├────────────────────────────────────────────────────────────────────────────────────┤
 * │  <text appears here>                                                               │
 * └────────────────────────────────────────────────────────────────────────────────────┘
 */



const MAX_WINDOW_WIDTH: u16 = 80;
const MAX_WINDOW_HEIGHT: u16 = 40;
const H_PADDING: u16 = 2; 
const V_PADDING: u16 = 1;
const TL_CORNER: char = '┌';
const TR_CORNER: char = '┐'; 
const BL_CORNER: char = '└';
const BR_CORNER: char = '┘';
const VERT_EDGE: char = '│';
const HORI_EDGE: char = '─';
const LVDIV_EDGE: char = '├';
const RVDIV_EDGE: char = '┤';
const MAX_HLINE_LENGTH: u16 = MAX_WINDOW_WIDTH - 2u16 * H_PADDING;

pub struct ChatWindow {
    text: Vec<String>,
}

impl ChatWindow {
    pub fn new () -> ChatWindow {
        ChatWindow {
            text: vec![
                String::from("Hello world!"),
                String::from("This is a test string"),
            ]
        }
    }

    pub fn print (&self) {
        execute!(
            stdout(),
            SetSize(MAX_WINDOW_WIDTH, MAX_WINDOW_HEIGHT), 
            Clear(ClearType::Purge),
            ScrollUp(0)
        );
        println!("nothing happened?");
        let top_bar: Vec<char> = [[TL_CORNER].to_vec(), [HORI_EDGE; MAX_HLINE_LENGTH as usize].to_vec(), [TR_CORNER].to_vec()].concat();
        println!("{}", top_bar.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(""));
        
        let get_text_by_line = |string_line: &String| {
            let left_padding: u16 = 2;
            let right_padding: u16 = MAX_HLINE_LENGTH - left_padding - (string_line.len() as u16);
            let text = [
                [VERT_EDGE].to_vec(),
                vec![' '; left_padding as usize],
                string_line.chars().collect::<Vec<char>>(),
                vec![' '; right_padding as usize],
                [VERT_EDGE].to_vec()
            ].concat();
            println!("{}", text.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(""))
        };

        self.text.iter().for_each(get_text_by_line);
    }

}

fn main() {
    let cw = ChatWindow::new();
    cw.print();
}