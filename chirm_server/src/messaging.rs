use serde::{Deserialize, Serialize};
use webrtc::{ice_transport::ice_candidate::RTCIceCandidateInit, peer_connection::sdp::session_description::RTCSessionDescription};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InMessage {
    Connect {
        id: String,
    },
    Disconnect {
        id: String,
    },
    Offer {
        to: String,
        sdp: RTCSessionDescription,
    },
    Answer {
        to: String,
        sdp: RTCSessionDescription,
    },
    IceCandidate {
        to: String,
        candidate: RTCIceCandidateInit,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutMessage {
    BroadcastUsers {
        users: Vec<String>,
    },
    UserConnected {
        user: String,
    },
    UserDisconnected {
        user: String,
    },
    Offer {
        from: String,
        to: String,
        sdp: RTCSessionDescription,
    },
    Answer {
        from: String,
        to: String,
        sdp: RTCSessionDescription,
    },
    IceCandidate {
        from: String,
        candidate: RTCIceCandidateInit,
    },
    Error {
        message: String,
    },
}
