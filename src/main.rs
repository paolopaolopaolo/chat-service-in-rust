use std::{
    env::args,
    sync::{Arc, Mutex, mpsc},
    net::{TcpStream},
    io::{BufReader, BufRead, stdin, stdout},
    thread,
};
use chat_server::window::window::{SharedChatWindow, ChatWindow, ChatInput, lock_chat_window, WindowActions, println_starting_at};

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
    let cw: SharedChatWindow = Arc::new(Mutex::new(ChatWindow::new(name.clone())));
    let mut cw_clone0: SharedChatWindow = cw.clone();
    let mut cw_clone1: SharedChatWindow = cw.clone();
    let cw_clone2: SharedChatWindow = cw.clone();
    let locked_cw = lock_chat_window(&mut cw_clone0);
    locked_cw.print();

    let h1 = thread::spawn(move || {
        println_starting_at(&mut stdout(), String::from("test: thread started"), 25, 4);

        match connect {
            Ok(mut stream) => {
                let bufreader = BufReader::new(&mut stream);
                    let mut buf_array = bufreader
                        .lines()
                        .map(|i| i.unwrap());
                    loop {
                        match buf_array.next() {
                            Some(string) => {
                                println_starting_at(&mut stdout(), String::from("test: pre-chat lock"), 25, 4);
 
                                let mut locked_cw = lock_chat_window(&mut cw_clone1);
                                println_starting_at(&mut stdout(), format!("test: {}", string), 25, 4);
                                locked_cw.add_chat_line(string);
                            },
                            _ => {},
                        }
                    }
            },
            Err(v) => {println!("Error: {}", v)}
        }
    });

    let (tx, rx) = mpsc::channel();
    let h2 = thread::spawn(move || {
        let mut locked_chat_window = lock_chat_window(&cw_clone2);
        for received in rx.recv() {
            match received {
                WindowActions::ScrollUp => {
                    locked_chat_window.scroll_up();
                },
                _ => {}
            }
        }
    });

    let mut chat_input = ChatInput::new(name);
    let h3 = thread::spawn(move || {
        chat_input.capture_events(client_socket.as_str(), tx.clone());
    });

    h1.join();
    h2.join();
    h3.join();

}
