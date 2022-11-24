use std::io::{
    stdout,
};
use crossterm::{
    execute,
    event::{
        KeyEvent, KeyModifiers, KeyCode
    },
    terminal::{
        Clear,
        ClearType,
        disable_raw_mode
    },
};

fn handle_modified_keys(modifiers: KeyModifiers, code: KeyCode) {
    match modifiers {
        KeyModifiers::CONTROL => {
            match code {
                KeyCode::Char(char) => {
                    match char {
                        'c' => {
                            execute!(stdout(), Clear(ClearType::All)).unwrap();
                            disable_raw_mode().expect("error with disable raw mode");
                            println!("CTRL+C to exit");
                        },
                        _ => {}
                    }
                }
                _ => {},
            }
        },
        _ => {}
    }
}

// fn handle_key_codes(&mut self, code: KeyCode) {
//     match code {
//         KeyCode::Char(char) => {
//             self.text = format!("{}{}", self.text, char);
//             println_starting_at(
//                 &mut stdout(),
//                 self.text.clone(),
//                 START_AT_ROW,
//                 START_AT_COLUMN
//             );
//         },
//         KeyCode::Left => {
//             println_starting_at(&mut stdout(), 
//                 "Left Key Pressed!".to_string(), 
//                 START_AT_ROW + 10, 
//                 START_AT_COLUMN
//             );
//         },
//         KeyCode::Right => {
//             println_starting_at(&mut stdout(), 
//                 "Right Key Pressed!".to_string(), 
//                 START_AT_ROW + 10, 
//                 START_AT_COLUMN
//             );
//         },
//         KeyCode::Up => {
//             println_starting_at(&mut stdout(), 
//                 "Up Key Pressed!".to_string(), 
//                 START_AT_ROW + 10, 
//                 START_AT_COLUMN
//             );
//         },
//         KeyCode::Down => {
//             println_starting_at(&mut stdout(), 
//                 "Down Key Pressed!".to_string(), 
//                 START_AT_ROW + 10, 
//                 START_AT_COLUMN
//             );
//         },
//         KeyCode::Enter => {
//             let target_string = format!("\r\n{}: {}\r\n", self.name.clone(), self.text.clone());
//             stream.write(target_string.as_bytes()).expect("write failed");
//             self.text = "".to_string();

//             println_starting_at(
//                 &mut stdout(),
//                 self.text.clone(),
//                 START_AT_ROW,
//                 START_AT_COLUMN
//             );
//         },
//         KeyCode::Backspace => {
//             self.text = self.text[0..self.text.len() - 1].to_string();

//             println_starting_at(
//                 &mut stdout(),
//                 self.text.clone(),
//                 START_AT_ROW,
//                 START_AT_COLUMN
//             );
//         }
//         _ => {
//             // println_starting_at(
//             //     &mut stdout(),
//             //     format!("event: {:?}", event),
//             //     START_AT_ROW,
//             //     START_AT_COLUMN
//             // );
//         },
//     }
// }

pub fn handle_key_interactions(event: KeyEvent) {
    handle_modified_keys(event.modifiers, event.code);
}