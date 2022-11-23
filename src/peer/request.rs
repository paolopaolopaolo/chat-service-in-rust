// use std::{
//     io::{Read},
//     net::{TcpStream}, string::FromUtf8Error
// };

// enum Status {
//     OK,
//     Unauthorized,
//     Forbidden,
//     NotFound,
//     ServerError
// }

// #[derive(Debug)]
// pub struct Request {
//     pub stream: TcpStream,
//     pub method: String,
//     pub path: String,
//     pub headers: String,
//     pub body: String,
// }

// impl Request {
//     pub fn new(mut request: TcpStream) -> Option<Request> {
//         let mut output = [0u8; 10];
//         let mut vec: Vec<u8> = vec![];
//         loop {
//             let bytes = request.read(&mut output);
//             match bytes { 
//                 Ok(bytes) => {
//                     if bytes == usize::MIN { break; }
//                     else {
//                         let new_bytes: Vec<u8> = output
//                             .to_vec()
//                             .iter()
//                             .filter(|zero| **zero != 0u8)
//                             .map(|item| *item)
//                             .collect();
//                         let len = new_bytes.len();
//                         vec.extend(new_bytes);
//                         if len < 10 {
//                             break;
//                         }
//                         output = [0u8; 10];
//                     }
//                 },
//                 Err(err) => println!("error: {:?}", err),
//             }
//         }
//         match String::from_utf8(vec) {
//             Ok(v) => {
//                 let split_string: Vec<String> = v.split("\r\n\r\n").map(String::from).collect();
//                 let split_header: Vec<String> = split_string[0].split("\r\n").map(String::from).collect();
//                 let split_header_len: usize = split_header.len();
//                 println!("{}", split_header[1..split_header_len].join("\n"));
//                 println!("body: {}", split_string[1]);
//                 Some(Request {
//                     stream: request,
//                     method: String::from(""),
//                     path: split_header.get(0).unwrap().clone(),
//                     headers: format!("{}", split_header[1..split_header_len].join("\n")),
//                     body: format!("{}", split_string[1])
//                 })
//             },
//             Err(v) => { println!("error: {}", v); None },
//         }
        
//     }
// }
