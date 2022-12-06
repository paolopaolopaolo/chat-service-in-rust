
use regex::Regex;
use core::str::Bytes;

/** 
 * Chat Service Request Protocol
 *  -------------------
 * | Request Structure |
 *  -------------------
 * 
 * [1:SUBJECT][2:VERB][3:OBJECT]
 * 
 * SUBJECT, VERB, and OBJECT are all to be escaped and encoded as utf-8 strings.
 * 
 * Subject
 * -------
 * Indicates the originator of the request. Alphanumerics and some special cha only.
 * 
 * Verbs
 * -----
 * * INIT: Starts a request.
 * * TX: Transmits a message
 * * END: Ends the request.
 * 
 * **/

 #[derive(Debug)]

 pub enum ChatRequestStatus {
    Valid,
    Invalid,
 }

 #[derive(Debug)]
 pub struct ChatRequest {
    pub subject: Option<String>,
    pub verb: Option<String>,
    pub object: Option<String>,
    pub status: ChatRequestStatus
 }

 impl ChatRequest {
    pub fn from(string: String) -> ChatRequest {
        let default_result = ChatRequest {
            subject: None,
            verb: None,
            object: None,
            status: ChatRequestStatus::Invalid,
        };
        let parser = match Regex::new(r"\[1:(.*)\]\[2:(.*)\]\[3:(.*)\]") {
            Ok(v) => Some(v),
            _ => None
        };
        match parser {
            Some(x) => match x.captures(&string) {
                Some(captures) => {
                    return ChatRequest {
                        subject: Some(captures[1].to_string()),
                        verb: Some(captures[2].to_string()),
                        object: Some(captures[3].to_string()),
                        status: ChatRequestStatus::Valid
                    };
                },
                _ => { return default_result; },
            },
            None => { return default_result; },
        }
    }

    pub fn to_string_opt(&self) -> Option<String> {
        match self.status {
            ChatRequestStatus::Valid => {
                Some(
                    format!("[1:{}][2:{}][3:{}]\r\n",
                        self.subject.as_ref().unwrap().clone(),
                        self.verb.as_ref().unwrap().clone(),
                        self.object.as_ref().unwrap().clone()
                    )
                )
            },
            ChatRequestStatus::Invalid => {
                None
            }
        }
        
    }

    pub fn to_log(&self) -> Option<String> {
        match self.status {
            ChatRequestStatus::Valid => {
                let verb = self.verb.as_ref().unwrap().as_str();
                match verb {
                    "init" => return Some(format!(
                        "{} is connected!\r\n",
                        self.subject.as_ref().unwrap().clone(),
                    )),
                    "tx" => return Some(format!(
                        "{}: {}\r\n",
                        self.subject.as_ref().unwrap().clone(),
                        self.object.as_ref().unwrap().clone(),
                    )),
                    _ => return Some("error".to_string()),
                }
            },
            _ => None,
        }
    }
 }
