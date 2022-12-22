
use std::{
    sync::mpsc::Sender,
    io::{BufReader, BufRead},
    net::{TcpListener, TcpStream, Shutdown},
    result::Result,
};

use crate::threadpool::threadpool::Threadpool;
use crate::request::request::{ChatRequest, ChatRequestStatus};

pub struct Server {
    socket: String,
    // log_path: String
}

// BLOCKING
fn handle_connection(mut stream: TcpStream, tx: Sender<ChatRequest>) {
    let buf_reader = BufReader::new(&mut stream);
    let mut body = buf_reader
        .lines()            
        .map(|item| match item {
            Ok(it) => it,
            _ => String::from("")
        });
    loop {
        match body.next() {
            Some(msg) => { 
                let message = msg.clone();
                let request = ChatRequest::from(message);
                match request.status {
                    ChatRequestStatus::Valid => {
                        match tx.send(request) {
                            Err(_) => { break; },
                            _ => {}
                        }
                    },
                    ChatRequestStatus::Invalid => {
                        println!("error: {:?}", request);
                        stream.shutdown(Shutdown::Both);
                        break;
                    }
                }
            },
            _ => { 
                println!("connection broken!"); 
                break; 
            },
        };
    }
}

impl Server {

    pub fn new(socket: &str) -> Option<Server> {
        Some(Server {
            socket: String::from(socket),
        })
    }

    pub fn start(&self, executor_count: usize, tx: Sender<ChatRequest>) {
        let mut threadpool = Threadpool::new(executor_count);
        let listener = TcpListener::bind(self.socket.clone());
        match listener {
            Ok(listener) => {
                for stream in listener.incoming() {
                    match stream {
                        Result::Ok(stream) => {
                            let tx_main = tx.clone();
                            threadpool.execute(move || {
                                handle_connection(stream, tx_main);
                            });
                        },
                        Result::Err(err) => println!("Collecting stream failed. Error: {}", err.kind().to_string()),
                    }
                }
            },
            Err(e) => {println!("{:?}", e)},
        }
    }

}
