use std::{
    io::{Write, stdout},
    thread,
    sync::{mpsc, Arc, Mutex},
    time::Duration,
};
use crossterm::event::{
    read
};
use chat_server::window::window::{
    ChatWindow,
    println,
};

fn handle_interactions() {

}

fn main() {
    let (tx, rx) = mpsc::channel();
    let mut cw = ChatWindow::new();
    // Receives strings that we should render on our chat window.
    let handle1 = thread::spawn(move || {
        cw.print();
        loop {
            match rx.recv() {
                Ok(string) => {
                    cw.add_chat_line(string);
                },
                Err(e) => { println(&mut stdout(),format!("error: {:?}", e)); },
            };
        }
    });
    // Artificially creates strings that we should render on the chat window
    // TODO: connect to log_port and listen for traffic on that port.
    let handle2 = thread::spawn(move || {
        println!("handle2");
        loop {
            // TODO: bind to server-port that returns this data
            thread::sleep(Duration::from_secs(1));
            tx.send(String::from("Theo: Hello what is up?")).unwrap();
            thread::sleep(Duration::from_secs(1));
            tx.send(String::from("Zeke: Poopy doopy"));
            thread::sleep(Duration::from_secs(1));
            tx.send(String::from("Theo: That's dumb"));
            thread::sleep(Duration::from_secs(1));
            tx.send(String::from("Zeke: You're dumb"));
        }
    });
    // let handle3 = thread::spawn(move || {
    //     let mut unwrapped_chat_window = slice_listener_clone.lock().unwrap();
    //     unwrapped_chat_window.listen_for_slice_changes();
    // });
    // TODO: handle interactions that get sent (use examples/client.rs as a guide)
    handle1.join();
    handle2.join();
    // handle3.join();
}