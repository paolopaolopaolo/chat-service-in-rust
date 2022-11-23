
use std::{
    thread,
    sync::mpsc::{self, Sender, Receiver},
    io::{BufReader, BufRead, Write},
    net::{TcpListener, TcpStream},
    result::Result,
};

use crate::threadpool::threadpool::Threadpool;

pub struct Server {
    host: String,
    port: String,
    // log_path: String
}

// BLOCKING
fn handle_connection(mut stream: TcpStream, tx: Sender<String>) {
    let buf_reader = BufReader::new(&mut stream);
    let mut body = buf_reader
        .lines()            
        .map(|item| match item {
            Ok(it) => it,
            _ => String::from("")
        });
    loop {
        match body.next() {
            Some(msg) => { match tx.send(msg) {
                Ok(()) => { println!("message sent"); }
                Err(e) => {println!("message send failed: {:?}", e); break; }
            }},
            _ => { println!("connection broken!"); break; },
        };
    }
}

impl Server {

    pub fn new(host: &str, port: &str) -> Option<Server> {
        Some(Server {
            host: String::from(host),
            port: String::from(port),
        })
    }

    pub fn start(&self, executor_count: usize, tx: Sender<String>) {
        let mut threadpool = Threadpool::new(executor_count);
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port));
        match listener {
            Ok(listener) => {
                for stream in listener.incoming() {
                    println!("9000 stream accepted");
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
