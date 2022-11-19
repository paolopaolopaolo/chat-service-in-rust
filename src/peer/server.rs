
use std::{
    io::{BufReader, BufRead},
    net::{TcpListener, TcpStream},
    result::Result,
};

use crate::threadpool::threadpool::Threadpool;
// use crate::peer::request::Request;

pub struct Server {
    listener: TcpListener,
    // log_path: String
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let mut body = buf_reader
        .lines()            
        .map(|item| match item {
            Ok(it) => it,
            _ => String::from("")
        });
    loop {
        match body.next() {
            Some(msg) => println!("{}", msg),
            _ => {}
        }
    }

    // let request = Request::new(stream);
    // let mut request_body = request.unwrap();
    // println!("parsed request: {:?}", request_body);
    
    // request_body.stream.write_all(String::from("HTTP/1.1 200 OK\r\nContent-Length:0\r\n\r\n").as_bytes()).unwrap();
}

impl Server {

    pub fn new(host: &str, port: &str) -> Option<Server> {
        let listener = TcpListener::bind(format!("{host}:{port}"));
        match listener {
            Result::Ok(list) => Some(Server {
                listener: list,
                // log_path: String::from(log_path),
            }),
            Result::Err(err) => { println!("Binding to {}:{} failed. Error: {}", host, port, err.kind().to_string().to_uppercase()); None },
        }
    }

    pub fn start(&self, executor_count: usize) {
        let mut threadpool = Threadpool::new(executor_count);
        for stream in self.listener.incoming() {
            match stream {
                Result::Ok(stream) => {
                    threadpool.execute(move || {
                        handle_connection(stream);
                    });
                },
                Result::Err(err) => println!("Collecting stream failed. Error: {}", err.kind().to_string()),
            }
        }
    }

    

}
