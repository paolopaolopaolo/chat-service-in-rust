use std::{
    io::{Write, Error},
    net::{TcpListener, TcpStream},
    sync::{
        Arc,
        Mutex,
        mpsc::{self, Receiver, Sender},
    }
};
use crate::threadpool::threadpool::Threadpool;

type TextLog = Arc<Mutex<Vec<String>>>;

pub struct InMemoryChatBuffer {
    pub text: TextLog,
    receiver: Receiver<String>,
    sender: Sender<String>,
}

fn handle_connection(stream: Result<TcpStream, Error>, text: TextLog) -> Result<(), ()> {
    match stream {
        Ok(mut stream_obj) => {
            // Adds deduping so we only write what hasn't been written yet.
            let mut start_from: usize = 0;
            loop {
                match text.try_lock() {
                    Ok(array) => {
                        let end_at = array.len();
                        if end_at - start_from > usize::MIN {
                            match stream_obj.write(
                                format!(
                                    "{}\n",
                                    array[start_from..end_at].join("\n")
                                ).as_bytes()) {
                                Err(err) => {
                                    println!("chatlog.rs:39 {:?}", err);
                                    break;
                                },
                                _ => {}
                            }
                            ;
                            start_from = end_at;
                        }
                    },
                    _ => { },
                }
            }
        },
        Err(e) => { println!("chatlog.rs:49 Connection broke: {:?}", e)},
    }
    Ok(())
}

pub fn create_listener(text: TextLog, socket: &str) {
    let listener = TcpListener::bind(socket);
    let mut tp = Threadpool::new(10000);
    match listener {
        Ok(listnr) => {
            for stream in listnr.incoming() {
                let cloned_text = text.clone();
                tp.execute(move || { handle_connection(stream, cloned_text); });
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

    // Listen for updates to the chatlog (BLOCKING)
    pub fn listen_for_updates(&self) {
        for string in self.receiver.iter() {
            // TODO: handle this better
            match self.text.clone().lock() {
                Ok(mut arr) => { arr.push(string); },
                _ => { println!("Lock failed when listening for updates"); }
            }
        }
    }
}