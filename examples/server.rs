use chat_server::peer::server::Server;
use chat_server::peer::chatlog::{InMemoryChatBuffer, create_listener};
use std::{
    env::args,
    thread
};

const DEFAULT_EXECUTOR_COUNT: usize = 20;

fn main() {
    let cli_args: Vec<String> = args().collect();
    let host = match cli_args.get(1) {
        Some(x) => x,
        _ => "0.0.0.0"
    };
    let port = match cli_args.get(2) {
        Some(x) => x,
        _ => "9000"
    };

    let executor_count = match cli_args.get(3) {
        Some(x) => match x.parse::<usize>() {
                Ok(count) => count,
                _ => DEFAULT_EXECUTOR_COUNT,
        },
        _ => DEFAULT_EXECUTOR_COUNT,
    };
    let mut chat_buffer = InMemoryChatBuffer::new();
    let mut text = chat_buffer.text.clone();
    let tx = chat_buffer.create_tx();
    let server = Server::new(host, port);
   
    
    let handle0 = thread::spawn(move || {
        println!("chat buffer listening for updates");
        chat_buffer.listen_for_updates();
    });
    let handle1 = thread::spawn(move || {
        println!("server started");
        match server {
            Some(server) => { server.start(executor_count, tx); },
            _ => { println!("Aborted!"); }
        }
    });
    let handle2 = thread::spawn(move|| {
        println!("listener create");
        create_listener(text, "0.0.0.0:8000");
    });
    handle0.join().expect("handle0 fail");
    handle1.join().expect("handle1 fail");
    handle2.join().expect("handle2 fail");
}
