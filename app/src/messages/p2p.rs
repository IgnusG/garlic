use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct P2PMessage {
    pub message_type: P2P,
    pub data: Option<Vec<u8>>
}
impl P2PMessage {
    pub fn new(message_type: P2P) -> P2PMessage {
        P2PMessage {
            message_type: message_type,
            data: None
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum P2P {
    Knock,
    WhosThere,
    Handshake,
    Incomming,
    Forward,
    Data
}
