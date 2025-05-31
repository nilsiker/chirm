use futures::{SinkExt, StreamExt, stream::SplitSink};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use warp::{
    Filter,
    filters::ws::{Message, WebSocket},
};

type Clients = Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>>>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignalMessage {
    Register { id: String },
    Offer { to: String, sdp: String },
    Answer { to: String, sdp: String },
    IceCandidate { to: String, candidate: String },
}

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    let clients_filter = warp::any().map(move || clients.clone());

    let route = warp::path("ws")
        .and(warp::ws())
        .and(clients_filter)
        .map(|ws: warp::ws::Ws, clients| ws.on_upgrade(move |socket| handle_ws(socket, clients)));

    println!("Signaling server listening on ws://localhost:3030/ws");
    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_ws(ws: warp::ws::WebSocket, clients: Clients) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, _) = unbounded_channel::<Message>();

    while let Some(Ok(msg)) = ws_rx.next().await {
        if let Some(signal_msg) = handle_ws_message(msg) {
            match signal_msg {
                SignalMessage::Register { id } => {
                    handle_register_message(id, &clients, &mut ws_tx, tx.clone()).await
                }
                SignalMessage::Offer { to, sdp } => {
                    println!("Received offer for {to}: {sdp}");
                    // Route to target client
                }
                SignalMessage::Answer { to, sdp } => {
                    println!("Received answer for {to}: {sdp}");
                }
                SignalMessage::IceCandidate { to, candidate } => {
                    println!("Received ICE candidate for {to}: {candidate}");
                }
            }
        }
    }

    println!("ws closing...");
}

async fn handle_register_message(
    id: String,
    clients: &Clients,
    ws_tx: &mut SplitSink<WebSocket, Message>,
    tx: UnboundedSender<Message>,
) {
    println!("Registering client with id: {id}");
    if clients.lock().unwrap().contains_key(&id) {
        ws_tx
            .send(Message::text("Conflict, user exists"))
            .await
            .unwrap();
        ws_tx.close().await.unwrap();
    } else {
        clients.lock().unwrap().insert(id, tx);
        ws_tx.send(Message::text("Registered")).await.unwrap();
    }
}

fn handle_ws_message(msg: Message) -> Option<SignalMessage> {
    if let Ok(text) = msg.to_str() {
        serde_json::from_str::<SignalMessage>(text).ok()
    } else {
        None
    }
}
