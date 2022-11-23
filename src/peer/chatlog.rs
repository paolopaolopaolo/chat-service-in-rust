use std::{
    io::{Write, Error},
    net::{TcpListener, TcpStream},
    sync::{
        Arc,
        Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::Duration
};
use crate::threadpool::threadpool::Threadpool;

type TextLog = Arc<Mutex<Vec<String>>>;

pub struct InMemoryChatBuffer {
    pub text: TextLog,
    receiver: Receiver<String>,
    sender: Sender<String>,
}

fn handle_connection(stream: Result<TcpStream, Error>, text: TextLog) {
    match stream {
        Ok(mut stream_obj) => {
            println!("stream connected to 8000");
            // Adds deduping so we only write what hasn't been written yet.
            let mut start_from: usize = 0;
            loop {
                match text.try_lock() {
                    Ok(array) => {
                        let end_at = array.len();
                        if end_at - start_from > usize::MIN {
                            println!("start_from: {}, end_at: {}", start_from, end_at);
                            println!("ok??? array to print:\n{:?}", array[start_from..end_at].join("\n"));
                            stream_obj.write(format!("{}\n", array[start_from..end_at].join("\n")).as_bytes());
                            start_from = end_at;
                        }
                    },
                    _ => { println!("fail, try again"); },
                }
                thread::sleep(Duration::from_millis(500));
            }
        },
        Err(e) => { println!("Lock failed: {:?}", e)},
    }
}

pub fn create_listener(text: TextLog, socket: &str) {
    let listener = TcpListener::bind(socket);
    let mut tp = Threadpool::new(100);
    match listener {
        Ok(listnr) => {
            for stream in listnr.incoming() {
                let cloned_text = text.clone();
                tp.execute(move || { handle_connection(stream, cloned_text)});
            }
        },
        _ => {},
    };
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
    

    // Listen for updates to the chatlog (BLOCKING)
    pub fn listen_for_updates(&self) {
        for string in self.receiver.iter() {
            let handler = &self.text.clone();
            // TODO: handle this better
            handler.lock().unwrap().push(string);
        }
    }
}