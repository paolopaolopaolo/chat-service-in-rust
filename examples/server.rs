use chat_server::peer::server::Server;
use chat_server::peer::chatlog::InMemoryChatBuffer;
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
    let tx = chat_buffer.create_tx();
    let server = Server::new(host, port);
   
    
    let handle0 = thread::spawn(move || {
        chat_buffer.listen_for_updates();
    });
    // chat_listener();
    let handle1 = thread::spawn(move || {
        match server {
            Some(server) => { server.start(executor_count, tx); },
            _ => { println!("Aborted!"); }
        }
    });
    // let handle2 = thread::spawn(move|| {
    //     chat_buffer.create_listener();
    // });
    handle0.join();
    handle1.join();
}
