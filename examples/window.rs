use std::{
    env::args,
    net::{TcpStream},
    io::{BufReader, BufRead}
};
use chat_server::window::window::{ChatWindow};

fn main() {
    let cli_args: Vec<String> = args().collect();
    let socket: String = match cli_args.get(1) {
        Some(string) => string.clone(),
        _ => String::from("0.0.0.0:8000")
    };
    let connect = TcpStream::connect(socket.as_str());
    let mut cw = ChatWindow::new();
    cw.print();
    match connect {
        Ok(mut stream) => {
            let bufreader = BufReader::new(&mut stream);
                let mut buf_array = bufreader
                    .lines()
                    .map(|i| i.unwrap());
                loop {
                    match buf_array.next() {
                        Some(string) => { cw.add_chat_line(string); },
                        _ => { () },
                    }
                }
        },
        Err(v) => {println!("Error: {}", v)}
    }
    
}
