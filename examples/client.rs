use std::{env::args, net::{TcpStream}, io::{stdin, Write}};


fn main() {
    let cli_args: Vec<String> = args().collect();
    let socket: String = match cli_args.get(1) {
        Some(string) => string.clone(),
        _ => String::from("0.0.0.0:9000")
    };
    println!("socket: {}", socket);
    let mut name = String::new();
    println!("What's your name?");
    stdin()
        .read_line(&mut name)
        .expect("Something broke. Try again!");
    println!("--------");
    name = name.trim().to_string();
    // TODO: Move this logic to a thread
    let connect = TcpStream::connect(socket.as_str());
    match connect {
        Ok(mut stream) => {
            loop {
                let mut string = String::new();
                stdin().read_line(&mut string).expect("error");
                stream.write(
                    format!("{}: {}", name, string).as_bytes()
                ).expect("Writing to socket failed.");
            }
        },
        Err(v) => {println!("Error: {}", v)}
    }
    
}
