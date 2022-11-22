
use std::{
    thread,
    sync::mpsc::{self, Sender, Receiver},
    io::{BufReader, BufRead},
    net::{TcpListener, TcpStream},
    result::Result,
};

use crate::threadpool::threadpool::Threadpool;

pub struct Server {
    listener: TcpListener,
    log_listener: TcpListener,
    // log_path: String
}

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
            Some(msg) => { tx.send(msg).unwrap();},
            _ => { tx.send(String::from("")).unwrap(); },
        };
    }

    // let request = Request::new(stream);
    // let mut request_body = request.unwrap();
    // println!("parsed request: {:?}", request_body);
    
    // request_body.stream.write_all(String::from("HTTP/1.1 200 OK\r\nContent-Length:0\r\n\r\n").as_bytes()).unwrap();
}

impl Server {

    pub fn new(host: &str, port: &str, log_port: Option<&str>) -> Option<Server> {
        let listener = TcpListener::bind(format!("{host}:{port}"));
        let log_listener = match log_port {
            Some(lp) => TcpListener::bind(format!("{host}:{lp}")),
            _ => TcpListener::bind(format!("{host}:8000")),
        };
        match listener {
            Result::Ok(list) => {
                match log_listener {
                    Result::Ok(log_list) => Some(Server {
                        listener: list,
                        log_listener: log_list,
                        // log_path: String::from(log_path),
                    }),
                    Result::Err(err) => {
                        println!(
                            "Binding to {}:{} failed. Error: {}",
                            host, 
                            match log_port {
                                Some(lp) => lp,
                                _ => "8000",
                            },
                            err.kind().to_string().to_uppercase()
                        );
                        None 
                    }
                }
                
            },
            Result::Err(err) => { println!("Binding to {}:{} failed. Error: {}", host, port, err.kind().to_string().to_uppercase()); None },
        }
    }

    pub fn start(&self, executor_count: usize) {
        let (tx, rx) = mpsc::channel::<String>();
        let mut threadpool = Threadpool::new(executor_count);
        // Loop that maintains status of chat-log
        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(string) => { println!("{:?}", string); },
                    _ => {println!("error");}
                }
            }
        });

        for stream in self.listener.incoming() {
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
    }

}
