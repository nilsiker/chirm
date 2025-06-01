mod client_manager;
mod messaging;

use client_manager::ClientManager;
use futures::{SinkExt, StreamExt};
use messaging::{InMessage, OutMessage};
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tracing::info;
use warp::{Filter, filters::ws::Message};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("debug").init();

    let client_manager = Arc::new(ClientManager::new());

    let client_manager_filter = warp::any().map(move || client_manager.clone());

    let route = warp::path("ws")
        .and(warp::ws())
        .and(client_manager_filter)
        .map(|ws: warp::ws::Ws, clients| ws.on_upgrade(move |socket| handle_ws(socket, clients)));

    info!("Signaling server listening on ws://localhost:3030/ws");
    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_ws(ws: warp::ws::WebSocket, clients: Arc<ClientManager>) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = unbounded_channel::<Message>();

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg.clone()).await.is_err() {
                break;
            } else if msg.to_str().unwrap() == "close" {
                ws_tx.close().await.unwrap();
            }
        }
    });

    let mut user = None;

    while let Some(Ok(msg)) = ws_rx.next().await {
        if let Some(signal_msg) = handle_ws_message(msg) {
            match signal_msg {
                InMessage::Connect { id } => {
                    user = Some(id.clone());
                    handle_connect_message(id, clients.clone(), tx.clone()).await;
                }
                InMessage::Disconnect { id } => {
                    clients.disconnect(&id).await;
                }
                InMessage::Offer { to, sdp } => {
                    info!("Received offer for {to}: {sdp}");
                    if let Some(tx) = clients.get_user_tx(&to).await {
                        let relay = OutMessage::Answer {
                            from: user.clone().unwrap().to_string(),
                            sdp,
                        };
                        tx.send(Message::text(format!("{:?}", relay))).unwrap();
                    }
                }
                InMessage::Answer { to, sdp } => {
                    info!("Received answer for {to}: {sdp}");
                }
                InMessage::IceCandidate { to, candidate } => {
                    info!("Received ICE candidate for {to}: {candidate}");
                }
            }
        }
    }

    handle_ws_closing();
}

fn handle_ws_closing() {
    info!("ws closing...");
}

async fn handle_connect_message(
    id: String,
    clients: Arc<ClientManager>,
    tx: UnboundedSender<Message>,
) {
    info!("Registering client with id: {id}");

    match clients.connect(id.clone(), tx.clone()).await {
        Ok(_) => {
            let users = clients.list().await;
            if !users.is_empty() {
                let users_broadcast =
                    serde_json::to_string(&OutMessage::BroadcastUsers { users: users }).unwrap();
                tx.send(Message::text(users_broadcast)).unwrap();
            }
        }
        Err(e) => {
            tx.send(Message::text(e)).unwrap();
        }
    }
}

fn handle_ws_message(msg: Message) -> Option<InMessage> {
    if let Ok(text) = msg.to_str() {
        serde_json::from_str::<InMessage>(text).ok()
    } else {
        None
    }
}
