use std::{
    process,
    sync::mpsc::Sender,
    net::TcpStream,
    io::{
        stdout,
        Write
    }
};
use crossterm::{
    execute,
    event::{
        KeyModifiers, KeyCode
    },
    terminal::{
        Clear,
        ClearType,
        disable_raw_mode
    },
};
use crate::window::{
    ChatInput::{
        ChatInput,
    },
    helpers::*,
};
use crate::request::request::{ChatRequest, ChatRequestStatus, ChatRequestVerb};

pub fn handle_modified_keys(cw: &mut ChatInput, modifiers: KeyModifiers, code: KeyCode, stream: &mut TcpStream, start_at_row: u16, start_at_column: u16, dimensions: Dimensions) {
    match modifiers {
        KeyModifiers::CONTROL => {
            match code {
                KeyCode::Char(char) => {
                    match char {
                        'c' => {
                            execute!(stdout(), Clear(ClearType::All)).unwrap();
                            disable_raw_mode().expect("error with disable raw mode");
                            let request = ChatRequest {
                                subject: Some(cw.name.clone()),
                                verb: ChatRequestVerb::END,
                                object: Some(String::new()),
                                status: ChatRequestStatus::Valid
                            };
                            stream.write(request.to_string_opt().unwrap().as_bytes()).expect("problem writing to stream");
                            process::exit(0x0100);
                        },
                        _ => {}
                    }
                }
                _ => {},
            }
        },
        _ => {}
    }
}

pub fn handle_key_codes(cw: &mut ChatInput, modifiers: KeyModifiers, code: KeyCode, stream: &mut TcpStream,  tx: Sender<WindowActions>, start_at_row: u16, start_at_column: u16, dimensions: Dimensions) {
    match code {
        KeyCode::Char(char) => {
            cw.text = format!("{}{}", cw.text, char);
            let text_to_print = adjust_text_for_overflow(cw.text.clone(), dimensions.clone());
            if modifiers != KeyModifiers::CONTROL {
                println_starting_at(
                    &mut stdout(),
                    text_to_print,
                    start_at_row,
                    start_at_column,
                    dimensions
                );
            }
        },
        KeyCode::Up => {
            tx.send(WindowActions::ScrollUp).unwrap_or_else(|err| {
                println_starting_at(&mut stdout(), 
                format!("Error! {err}"), 
                start_at_row + 10, 
                start_at_column,
                dimensions
            );
            });
        },
        KeyCode::Down => {
            tx.send(WindowActions::ScrollDown).unwrap_or_else(|err| {
                println_starting_at(&mut stdout(), 
                format!("Error! {err}"), 
                start_at_row + 10, 
                start_at_column,
                dimensions
            );
            });
        },
        KeyCode::Left => {
            tx.send(WindowActions::CursorLeft).unwrap_or_else(|err| {
                println_starting_at(&mut stdout(), 
                format!("Error! {err}"), 
                start_at_row + 10, 
                start_at_column,
                dimensions
            );
            });
        },
        KeyCode::Right => {
            tx.send(WindowActions::CursorRight).unwrap_or_else(|err| {
                println_starting_at(&mut stdout(), 
                format!("Error! {err}"), 
                start_at_row + 10, 
                start_at_column,
                dimensions
            );
            });
        },
        KeyCode::Enter => {
            let request = ChatRequest {
                subject: Some(cw.name.clone()),
                verb: ChatRequestVerb::TX,
                object: Some(cw.text.clone()),
                status: ChatRequestStatus::Valid
            };
            let target_string = request.to_string_opt().unwrap();
            stream.write(target_string.as_bytes()).expect("write failed");
            cw.text = "".to_string();

            println_starting_at(
                &mut stdout(),
                cw.text.clone(),
                start_at_row,
                start_at_column,
                dimensions
            );
        },
        KeyCode::Backspace => {
            if cw.text.len() > 0 {
                cw.text = cw.text[0..cw.text.len() - 1].to_string();
                let text_to_print = adjust_text_for_overflow(cw.text.clone(), dimensions.clone());
                println_starting_at(
                    &mut stdout(),
                    text_to_print,
                    start_at_row,
                    start_at_column,
                    dimensions
                );
            }
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
}
