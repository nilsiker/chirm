use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InMessage {
    Connect { id: String },
    Disconnect { id: String },
    Offer { to: String, sdp: String },
    Answer { to: String, sdp: String },
    IceCandidate { to: String, candidate: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutMessage {
    BroadcastUsers { users: Vec<String> },
    UserConnected { user: String },
    UserDisconnected { user: String },
    Offer { from: String, sdp: String },
    Answer { from: String, sdp: String },
    IceCandidate { from: String, candidate: String },
    Error { message: String },
}