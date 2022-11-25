use std::{
    env::args,
    sync::{Arc, Mutex, mpsc},
    net::{TcpStream},
    io::{BufReader, BufRead},
    thread,
};
use chat_server::window::window::{
    SharedChatWindow,
    ChatWindow,
    ChatInput,
    lock_chat_window,
    WindowActions,
    BasicInputPanel
};

fn main() {
    let cli_args: Vec<String> = args().collect();
    let client_socket = match cli_args.get(1) {
        Some(string) => string.clone(),
        _ => String::from("0.0.0.0:9000")
    };
    let socket: String = match cli_args.get(2) {
        Some(string) => string.clone(),
        _ => String::from("0.0.0.0:8000")
    };
    let width: Option<usize> = match cli_args.get(3) {
        Some(string) => match string.parse::<usize>() {
            Ok(value) => Some(value),
            _ => None
        },
        _ => None
    };
    let height: Option<usize> = match cli_args.get(4) {
        Some(string) => match string.parse::<usize>() {
            Ok(value) => Some(value),
            _ => None
        },
        _ => None
    };

    let mut name = String::new();
    // Fancy UI for adding your name
    {
        let mut basic_panel = BasicInputPanel::new();
        basic_panel.enable_raw();
        basic_panel.print();

        let value = basic_panel.capture_input();
        name = value.clone();
    }

    // Instantiate and clone ChatWindow with Shared State
    let cw: SharedChatWindow = Arc::new(Mutex::new(ChatWindow::new(name.clone(), width, height)));
    let mut cw_clone0: SharedChatWindow = cw.clone();
    let mut cw_clone1: SharedChatWindow = cw.clone();
    let cw_clone2: SharedChatWindow = cw.clone();

    // Prints the initial window. Blocking.
    // TODO: re-print the window on screen re-size
    {
        let locked_cw = lock_chat_window(&mut cw_clone0);
        locked_cw.print();
    }

    // Thread 1: Connects the ChatWindow to traffic from ChatLog feed and adds lines to the ChatWindow
    let h1 = thread::spawn(move || {
        let connect = TcpStream::connect(socket.as_str());
        match connect {
            Ok(mut stream) => {
                let bufreader = BufReader::new(&mut stream);
                    let mut buf_array = bufreader
                        .lines()
                        .map(|i| i.unwrap());
                    loop {
                        match buf_array.next() {
                            Some(string) => { 
                                let mut locked_cw = lock_chat_window(&mut cw_clone1);
                                locked_cw.add_chat_line(string);
                            },
                            _ => {},
                        }
                    }
            },
            Err(v) => {println!("Error: {}", v)}
        }
    });

    // Thread 2: Handles events transmitted from ChatInput and tells ChatWindow what to do in response.
    // TODO: Put this in a method/function.
    let (tx, rx) = mpsc::channel();
    let h2 = thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(received) => {
                    match received {
                        WindowActions::ScrollUp => {
                            let mut locked_chat_window = lock_chat_window(&cw_clone2);
                            locked_chat_window.scroll_up();
                        },
                        WindowActions::ScrollDown => {
                            let mut locked_chat_window = lock_chat_window(&cw_clone2);
                            locked_chat_window.scroll_down();
                        },
                        WindowActions::Resize(x, y) => {
                            let mut locked_chat_window = lock_chat_window(&cw_clone2);
                            locked_chat_window.dimensions.width = x;
                            locked_chat_window.dimensions.height = y;
                            locked_chat_window.print();
                        }
                        _ => {}
                    }
                },
                _ => {},
            }
        }
    });

    let h3 = thread::spawn(move || {
        let mut chat_input = ChatInput::new(name, width, height);
        chat_input.capture_events(client_socket.as_str(), tx.clone());
    });
    h1.join().expect("sad h1");
    h2.join().expect("sad h2");
    h3.join().expect("sad h3");

}
