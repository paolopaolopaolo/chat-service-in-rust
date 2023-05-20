use chat_service::{
    peer::{
        server::Server,
        chatlog::{
            InMemoryChatBuffer, 
            create_listening_threads_from_inmemory_buffer
        },
    },
};
use std::{
    any::Any,
    env::args,
    thread::{JoinHandle, self}, io::Error,
};

const DEFAULT_EXECUTOR_COUNT: usize = 20;

fn get_socket_client(cli_args: &Vec<String>) -> String {
    match cli_args.get(1) {
        Some(x) => x.clone(),
        _ => String::from("0.0.0.0:9000")
    }
}

fn get_socket_feed(cli_args: &Vec<String>) -> String {
    match cli_args.get(2) {
        Some(x) => x.clone(),
        _ => String::from("0.0.0.0:8000")
    }
}

fn get_executor_count(cli_args: &Vec<String>) -> usize {
    match cli_args.get(3) {
        Some(x) => match x.parse::<usize>() {
                Ok(count) => count,
                _ => DEFAULT_EXECUTOR_COUNT,
        },
        _ => DEFAULT_EXECUTOR_COUNT,
    }
}

fn flatten_joins(joins: Vec<JoinHandle<Result<(), Error>>>) -> Result<(), Box<dyn Any + Send + 'static>> {
    for join_handle in joins {
        join_handle.join()?;
    }
    Ok(())
}

fn main() {
    let cli_args: Vec<String> = args().collect();
    let socket_client = get_socket_client(&cli_args);
    let socket_feed = get_socket_feed(&cli_args);  
    let executor_count = get_executor_count(&cli_args);
    let chat_buffer = InMemoryChatBuffer::new();
    let (handle0, handle2, tx) = create_listening_threads_from_inmemory_buffer(chat_buffer, socket_feed);
    let server = Server::new(socket_client.as_str());
    let handle1 = thread::spawn(move || {
        server.start(executor_count, tx.clone())
    });

    match flatten_joins(vec![
        handle0,
        handle1,
        handle2,
    ]) {
        Ok(()) => { println!("Ok!"); },
        _ => { println!("Not ok!"); }
    };
}
