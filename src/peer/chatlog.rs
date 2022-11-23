use std::{
    io::{Write},
    net::{TcpListener},
    sync::{
        Arc,
        Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::Duration
};

type TextLog = Arc<Mutex<Vec<String>>>;

pub struct InMemoryChatBuffer {
    text: TextLog,
    receiver: Receiver<String>,
    sender: Sender<String>,
}

impl InMemoryChatBuffer {
    pub fn new() -> InMemoryChatBuffer {
        let (tx, rx) = mpsc::channel();
        InMemoryChatBuffer {
            text: Arc::new(Mutex::new(vec![])),
            receiver: rx,
            sender: tx,
        }
    }

    // Create Senders that can send data to the chatlog
    pub fn create_tx(&self) -> Sender<String> {
        self.sender.clone()
    }

    // Create a closure that when called spins up a port (BLOCKING)
    pub fn create_listener(&self) {
        let text = self.text.clone();
        let listener = TcpListener::bind("0.0.0.0:8000");
        match listener {
            Ok(listnr) => {
                for stream in listnr.incoming() {
                    match stream {
                        Ok(mut stream_obj) => {
                            let text_array = text.lock().unwrap();
                            stream_obj.write_all(text_array.join("\n").as_bytes());
                        },
                        _ => {},
                    }
                }
            },
            _ => {},
        };
    }

    // Listen for updates to the chatlog (BLOCKING)
    pub fn listen_for_updates(&self) {
        for string in self.receiver.iter() {
            println!("String received: {}", string);
            let handler = &self.text.clone();
            handler.lock().unwrap().push(string);
        }
    }
}