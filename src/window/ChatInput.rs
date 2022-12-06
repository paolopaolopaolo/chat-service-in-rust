
use std::{
    sync::{
        mpsc::Sender,
    },
    net::{TcpStream},
    io::{
        stdout,
        Write,
    }
};
use crossterm::{
    event::{
        read,
        Event,
    },
    terminal::{
        disable_raw_mode,
    }
};

use crate::{
    window::{
        helpers::*,
        constants::*,
        handlers::{handle_key_codes, handle_modified_keys},
    },
    request::request::{ChatRequest, ChatRequestStatus},
};

/**
 * ChatInput component
 */
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
        let mut start_at_column = 0;
        let stream = TcpStream::connect(socket);

        match stream {
            Ok(mut stream) => {
                let request = ChatRequest {
                    subject: Some(self.name.clone()),
                    // TODO: make "verb" an enum
                    verb: Some(String::from("init")),
                    object: Some(String::from("")),
                    status: ChatRequestStatus::Valid
                };
                let target_string = request.to_string_opt().unwrap();
                stream.write(target_string.as_bytes()).unwrap();
                loop {
                    match read() {
                        Ok(ev) => {
                            match ev {
                                Event::Key(event) => {
                                    handle_modified_keys(event.modifiers, event.code, self.dimensions.height as u16, start_at_column, self.dimensions.clone());
                                    handle_key_codes(
                                        self,
                                        event.modifiers,
                                        event.code,
                                        &mut stream,
                                        tx.clone(),
                                        self.dimensions.height as u16 + 1,
                                        0,
                                        self.dimensions.clone()
                                    );
                                },
                                Event::Resize(x, y) => {
                                    tx.clone().send(WindowActions::Resize(x as usize, y as usize - 2)).expect("didn't send resize event");
                                    self.dimensions.width = x as usize;
                                    self.dimensions.height = y as usize - 4;
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
                    self.dimensions.height as u16 + 12,
                    start_at_column,
                    self.dimensions.clone()
                );
            },
        }
        disable_raw_mode().expect("disable raw mode failed");
    }
}
