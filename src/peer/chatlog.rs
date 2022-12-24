use std::{
    io::{Write, Error},
    net::{TcpListener, TcpStream},
    sync::{
        Arc,
        Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle}
};
use crate::{
    request::request::ChatRequest,
    threadpool::threadpool::Threadpool
};

type TextLog = Arc<Mutex<Vec<String>>>;

pub struct InMemoryChatBuffer {
    pub text: TextLog,
    receiver: Receiver<ChatRequest>,
    sender: Sender<ChatRequest>,
}

// TODO: Consider tightly coupling this to ChatRequest
fn handle_connection(stream: Result<TcpStream, Error>, text: TextLog) -> Result<(), Error> {
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
                                Err(e) => {
                                    println!("stream write error: {:?}", e);
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
        Err(e) => { println!("Connection broke: {:?}", e)},
    }
    Ok(())
}

//TODO turn functional parts into a trait and re-implement with trait
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
    pub fn create_tx(&self) -> Sender<ChatRequest> {
        self.sender.clone()
    }

    // Listen for updates to the chatlog (BLOCKING)
    pub fn listen_for_updates(&self) -> Result<(), Error> {
        for chat_request in self.receiver.iter() {
            match self.text.clone().lock() {
                Ok(mut arr) => {
                    arr.push(chat_request.to_log()); 
                },
                _ => { println!("Update listener failed"); }
            }
        }
        Ok(())
    }
}

fn create_listener(text: TextLog, socket: &str, executor_count: usize) -> Result<(), Error> {
    let listener = TcpListener::bind(socket)?;
    let mut tp = Threadpool::new(executor_count);
    for stream in listener.incoming() {
        let cloned_text = text.clone();
        tp.execute(move || { 
            handle_connection(stream, cloned_text).expect("Connection failed");
        });
    }
    Ok(())
}

pub fn create_listening_threads_from_inmemory_buffer(chat_buffer: InMemoryChatBuffer, socket_feed: String) -> (JoinHandle<Result<(), Error>>, JoinHandle<Result<(), Error>>, Sender<ChatRequest>) {
    let text = chat_buffer.text.clone();
    let sender = chat_buffer.create_tx();
    let handle0 = thread::spawn(move || {
        chat_buffer.listen_for_updates()
    });
    let handle1 = thread::spawn(move|| {
        create_listener(text, socket_feed.clone().as_str(), 1000)
    });
    (handle0, handle1, sender)
}
