
use regex::Regex;
use std::fmt::{Display, Formatter, Error};

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
pub enum ChatRequestVerb {
    INIT,
    TX,
    END,
    NONE,
}

impl ChatRequestVerb {
    pub fn from_str(string: &str) -> ChatRequestVerb {
        match string {
            "init" => ChatRequestVerb::INIT,
            "tx" => ChatRequestVerb::TX,
            "end" => ChatRequestVerb::END,
            _ => ChatRequestVerb::NONE
        }
    }
    pub fn to_string(&self) -> &str {
        match self {
            ChatRequestVerb::INIT => "init",
            ChatRequestVerb::TX => "tx",
            ChatRequestVerb::END => "end",
            ChatRequestVerb::NONE => "none"
        }
    }
 }

impl Display for ChatRequestVerb {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug)]
pub struct ChatRequest {
    pub subject: Option<String>,
    pub verb: ChatRequestVerb,
    pub object: Option<String>,
    pub status: ChatRequestStatus
}

 impl ChatRequest {
    pub fn from(string: String) -> ChatRequest {
        let default_result = ChatRequest {
            subject: None,
            verb: ChatRequestVerb::NONE,
            object: None,
            status: ChatRequestStatus::Invalid,
        };
        let parser = match Regex::new(r"^\[1:(.*)\]\[2:(.*)\]\[3:(.*)\]$") {
            Ok(v) => Some(v),
            _ => None
        };
        match parser {
            Some(x) => match x.captures(&string) {
                Some(captures) => {
                    return ChatRequest {
                        subject: Some(captures[1].to_string()),
                        verb: ChatRequestVerb::from_str(&captures[2]),
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
                        self.verb,
                        self.object.as_ref().unwrap().clone()
                    )
                )
            },
            ChatRequestStatus::Invalid => {
                None
            }
        }
        
    }

    pub fn to_log(&self) -> String {
        match self.status {
            ChatRequestStatus::Valid => {
                match self.verb {
                    ChatRequestVerb::INIT => format!(
                        "{} is connected!\r\n",
                        match self.subject.as_ref() {
                            Some(string) => string,
                            _ => "(none)",
                        },
                    ),
                    ChatRequestVerb::TX => format!(
                        "{}: {}\r\n",
                        match self.subject.as_ref() {
                            Some(string) => string,
                            _ => "(none)"
                        },
                        match self.object.as_ref() {
                            Some(string) => string,
                            _ => "(none)"
                        },
                    ),
                    ChatRequestVerb::END => format!(
                        "{} disconnected!",
                        match self.subject.as_ref() {
                            Some(string) => string,
                            _ => "(none)"
                        }
                    ),
                    _ => "error".to_string(),
                }
            },
            _ => "error".to_string(),
        }
    }
 }
