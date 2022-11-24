use std::{
    env::args,
    net::{TcpStream},
    io::{BufReader, BufRead,stdin},
    thread,
};
use chat_server::window::window::{ChatWindow, ChatInput, set_up_input_to_window_listener};

fn main() {
    let cli_args: Vec<String> = args().collect();
    let socket: String = match cli_args.get(2) {
        Some(string) => string.clone(),
        _ => String::from("0.0.0.0:8000")
    };
    let client_socket = match cli_args.get(1) {
        Some(string) => string.clone(),
        _ => String::from("0.0.0.0:9000")
    };
    println!("What's your name?");
    let mut name = String::new();
    stdin().read_line(&mut name);
    name = name.trim().to_string();
    let connect = TcpStream::connect(socket.as_str());
    let mut cw = ChatWindow::new(name.clone());
    cw.print();
    let h1 = thread::spawn(move || {
        match connect {
            Ok(mut stream) => {
                let bufreader = BufReader::new(&mut stream);
                    let mut buf_array = bufreader
                        .lines()
                        .map(|i| i.unwrap());
                    loop {
                        match buf_array.next() {
                            Some(string) => { cw.add_chat_line(string);},
                            _ => {},
                        }
                    }
            },
            Err(v) => {println!("Error: {}", v)}
        }
    });

    // let h2 = thread::spawn(move || {
    //     set_up_input_to_window_listener(&mut cw)
    // });

    let mut chat_input = ChatInput::new(name);
    let h2 = thread::spawn(move || {
        chat_input.capture_events(client_socket.as_str());
    });

    h1.join();
    h2.join();

}
