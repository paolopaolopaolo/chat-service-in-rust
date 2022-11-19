mod peer;
mod threadpool;
use peer::server::Server;
use std::env::args;

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

    let server = Server::new(host, port);

    match server {
        Some(server) => { server.start(executor_count); },
        _ => { println!("Aborted!"); }
    }
}
