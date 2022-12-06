use chat_service::peer::server::Server;
use chat_service::peer::chatlog::{InMemoryChatBuffer, create_listening_threads_from_inmemory_buffer};
use std::{
    env::args,
    thread
};

const DEFAULT_EXECUTOR_COUNT: usize = 20;

fn main() {
    let cli_args: Vec<String> = args().collect();
    let socket_client = match cli_args.get(1) {
        Some(x) => x,
        _ => "0.0.0.0:9000"
    };
    let socket_feed = match cli_args.get(2) {
        Some(x) => x.clone(),
        _ => String::from("0.0.0.0:8000")
    };  
    let executor_count = match cli_args.get(3) {
        Some(x) => match x.parse::<usize>() {
                Ok(count) => count,
                _ => DEFAULT_EXECUTOR_COUNT,
        },
        _ => DEFAULT_EXECUTOR_COUNT,
    };
    let chat_buffer = InMemoryChatBuffer::new();
    let (handle0, handle2, tx) = create_listening_threads_from_inmemory_buffer(chat_buffer, socket_feed);
    let server = Server::new(socket_client);
    let handle1 = thread::spawn(move || {
        match server {
            Some(server) => { server.start(executor_count, tx.clone()); },
            _ => { println!("Aborted!"); }
        }
    });
    handle0.join().expect("handle0 fail");
    handle1.join().expect("handle1 fail");
    handle2.join().expect("handle2 fail");
}
