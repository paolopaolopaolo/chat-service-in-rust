use std::{
    // sync::mpsc,
    net::{TcpStream},
    io::{self, BufReader, BufRead},
};
use regex::Regex;

// TODO: refactor BOT into a trait
pub struct Bot<F> 
    where F: Fn(String, &mut TcpStream) -> Option<()>
{
    wake_pattern: String,
    listens_on: TcpStream,
    writes_to: TcpStream,
    on_wake: F
}

impl<F> Bot<F> 
    where F: Fn(String, &mut TcpStream) -> Option<()> {
    pub fn new(wake_pattern: String, listens_port: String, writes_port: String, on_wake: F) -> Result<Bot<F>, io::Error> {
        let listens_on = TcpStream::connect(listens_port)?;
        let writes_to = TcpStream::connect(writes_port)?;
        // let (thread_spawner, thread_spawn_responder) = mpsc::channel::<u8>();
        return Result::Ok(
            Bot {
                wake_pattern,
                listens_on,
                writes_to,
                on_wake
            }
        );
    }


    pub fn listen_on(&mut self) {
        let reader = BufReader::new(&self.listens_on);
        let mut lines = reader
            .lines()
            .map(|item| {match item {
                Ok(string) => { 
                    let splitted = string.split(": ")
                        .map(|i| i.to_string())
                        .collect::<Vec<String>>();
                    if splitted.len() > 1 {
                        return splitted[1].clone();
                    }
                    String::new()
                    },
                _ => {String::new()}}
            });
        while let Some(line) = lines.next() {
            let pattern = Regex::new(format!("^{} *(.*)$", self.wake_pattern).as_str());
            match pattern {
                Ok(patt) => {
                    match patt.captures(line.as_str()) {
                        Some(arr) => {
                            // TODO: Spawn a thread to run the response
                            (self.on_wake)(arr[1].to_string(), &mut self.writes_to);
                        },
                        _ => {}
                    }
                },
                Err(err) => {
                    println!("break: {:?}", err);},
            }
        }
    }
}
