use chat_service::{
    peer::{
        server::Server,
        chatlog::{
            InMemoryChatBuffer, 
            create_listening_threads_from_inmemory_buffer
        },
    },
    experiment::bot::{
        Bot
    },
    request::request::{
        ChatRequest,
        ChatRequestVerb,
        ChatRequestStatus
    }
};
use std::{
    env::args,
    thread,
    io::{Write},
    time::Duration
};
use reqwest;
use serde_json;

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

fn main() {
    let cli_args: Vec<String> = args().collect();
    let socket_client = get_socket_client(&cli_args);
    let socket_client_clone = socket_client.clone();
    let socket_feed = get_socket_feed(&cli_args);  
    let socket_feed_clone = socket_feed.clone();
    let executor_count = get_executor_count(&cli_args);
    let chat_buffer = InMemoryChatBuffer::new();
    let (handle0, handle2, tx) = create_listening_threads_from_inmemory_buffer(chat_buffer, socket_feed);
    let server = Server::new(socket_client.as_str());
    let handle1 = thread::spawn(move || {
        match server {
            Some(server) => { server.start(executor_count, tx.clone()); },
            _ => { println!("Aborted!"); }
        }
    });

    // Creates a bot
    let bot = Bot::new(
        String::from(r"\\bot"),
        String::from(socket_feed_clone),
        String::from(socket_client_clone),
        |string, stream| {
            println!("wake word activated!");
            let json = format!("{{\"prompt\": \"{}\", \"max_tokens\": 100}}", string);
            let client= reqwest::blocking::Client::new();
            let resp = client.post("https://api.openai.com/v1/engines/davinci/completions")
                .header("Authorization", "wouldnt you like to see this")
                .header("Content-Type", "application/json")
                .body(json)
                .send();

            match resp {
                Ok(resp) => {
                    let json_resp = match resp.json::<serde_json::Value>() {
                        Ok(hashmap) => hashmap,
                        Err(e) => serde_json::Value::String(format!("{:?}", e)),
                    };
                    println!("success = {:?}", json_resp);
                    let choices = json_resp.get("choices")?;
                    let arr = choices.as_array()?;
                    for val in arr {
                        let text = val.get("text")?;
                        let new_line = text.to_string();
                        let splits = new_line.split("\\n");
                        for line in splits {
                            let request = ChatRequest {
                                subject: Some(String::from("Bot")),
                                verb: ChatRequestVerb::TX,
                                object: Some(String::from(line)),
                                status: ChatRequestStatus::Valid
                            };
                            let string_to_write = request.to_string_opt()?;
                            stream.write(string_to_write.as_bytes()).expect("spuh");
                            thread::sleep(Duration::from_millis(500));
                        }
                    }
                    Some(())
                },
                Err(err) => {
                    println!("error: {:?}", err);
                    None
                },
            }
        }
    );
    match bot {
        Ok(mut bot) => {
            let handle3 = thread::spawn(move || {
                bot.listen_on();
            });
            handle3.join().expect("bot listener should keep running");
        },
        Err(err) => {println!("Error: {}", err);}
    }
    
    handle0.join().expect("chatlog listener should have kept running");
    handle1.join().expect("chat input listener should have kept running");
    handle2.join().expect("chat feed listener should have kept running");
}
