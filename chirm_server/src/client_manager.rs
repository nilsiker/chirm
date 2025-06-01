use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, MutexGuard, mpsc::UnboundedSender};
use tracing::{info, warn};
use warp::filters::ws::Message;

use crate::messaging::OutMessage;

type Clients = Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>>>;
type ClientsGuard<'a> = MutexGuard<'a, HashMap<String, UnboundedSender<Message>>>;

#[derive(Debug)]
pub struct ClientManager {
    clients: Clients,
}

impl ClientManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[tracing::instrument(skip(self, tx))]
    pub async fn connect(&self, id: String, tx: UnboundedSender<Message>) -> Result<(), &str> {
        let mut clients = self.clients.lock().await;

        match clients.contains_key(&id) {
            true => {
                warn!("user is already connected");
                tx.send(Message::text(
                    serde_json::to_string(&OutMessage::Error {
                        message: "user is already connected".to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap();
                Err("user is already connected")
            }
            false => {
                let users_broadcast = OutMessage::BroadcastUsers {
                    users: clients.keys().cloned().collect(),
                };
                tx.send(Message::text(
                    serde_json::to_string(&users_broadcast).unwrap(),
                ))
                .unwrap();

                clients.insert(id.clone(), tx);
                self.broadcast(&OutMessage::UserConnected { user: id }, &clients)
                    .await;
                Ok(())
            }
        }
    }

    pub async fn disconnect(&self, id: &str) {
        let mut clients = self.clients.lock().await;
        if clients.remove(id).is_some() {
            info!("disconnected {id}");
            let disconnect_msg = OutMessage::UserDisconnected {
                user: id.to_string(),
            };

            self.broadcast(&disconnect_msg, &clients).await;
        }
    }

    pub async fn get_user_tx(&self, id: &str) -> Option<UnboundedSender<Message>> {
        self.clients.lock().await.get(id).cloned()
    }

    async fn broadcast(&self, msg: &OutMessage, clients_guard: &ClientsGuard<'_>) {
        for tx in clients_guard.values() {
            let _ = tx.send(Message::text(serde_json::to_string(msg).unwrap()));
        }
    }
}
