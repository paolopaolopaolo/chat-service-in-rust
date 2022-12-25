
use std::{
    sync::{
        mpsc::Sender,
    },
    net::{TcpStream},
    io::{
        Write,
        Error
    }
};
use crossterm::{
    event::{
        read,
        Event,
    }
};

use crate::{
    window::{
        helpers::*,
        constants::*,
        handlers::{handle_key_codes, handle_modified_keys},
    },
    request::request::{ChatRequest, ChatRequestStatus, ChatRequestVerb},
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
    pub fn capture_events(&mut self, socket: &str, tx: Sender<WindowActions>) -> Result<(), Error> {
        let start_at_column = 0;
        let mut stream = TcpStream::connect(socket)?;
        let request = ChatRequest {
            subject: Some(self.name.clone()),
            verb: ChatRequestVerb::INIT,
            object: Some(String::from("")),
            status: ChatRequestStatus::Valid
        };
        let target_string = request.to_string_opt().unwrap();
        stream.write(target_string.as_bytes())?;
        while let Ok(ev) = read() {
            match ev {
                Event::Key(event) => {
                    handle_modified_keys(
                    self,
                        event.modifiers,
                        event.code,
                        &mut stream,
                        self.dimensions.height as u16,
                        start_at_column,
                        self.dimensions.clone()
                    );
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
        }
        Ok(())
    }
}
