use std::{env::args, io::Write, thread, time::Duration};
use serde_json::{json};
use chat_service::{request::request::{ChatRequest, ChatRequestVerb, ChatRequestStatus}, experiment::bot::Bot};

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

fn main() {
    let cli_args: Vec<String> = args().collect();
    let socket_client = get_socket_client(&cli_args);
    let socket_client_clone = socket_client.clone();
    let socket_feed = get_socket_feed(&cli_args);  
    let socket_feed_clone = socket_feed.clone();

   // Creates a bot
   let bot = Bot::new(
    String::from(r"/paolobot"),
    String::from(socket_feed_clone),
    String::from(socket_client_clone),
    |string, stream| {
        println!("wake word activated!");
        let json = format!("{{\"prompt\": \"{}\", \"max_tokens\": 100}}", string);
        let client= reqwest::blocking::Client::new();
        let resp = client.post("https://api.openai.com/v1/engines/davinci/completions")
            .header("Authorization", "boo")
            .header("Content-Type", "application/json")
            .body(json)
            .send();

        match resp {
            Ok(resp) => {
                let json_resp = match resp.json::<serde_json::Value>() {
                    Ok(hashmap) => hashmap,
                    Err(e) => serde_json::Value::String(format!("{:?}", e)),
                };
                let error_message = json!([{"text": json_resp["error"]["message"]}]);
                let response_to_render = match json_resp.get("choices") {
                    Some(choices) => choices,
                    _ => &error_message,
                };
                let arr = response_to_render.as_array().unwrap();
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
        let handle = thread::spawn(move || {
            bot.listen_on();
        });
        handle.join().expect("bot listener should keep running");
    },
    _ => {}
}
}

