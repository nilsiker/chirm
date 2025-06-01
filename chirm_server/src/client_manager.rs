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

        if self.is_connected(&id, &clients).await {
            warn!("user already exists");
            return Err("user is already connected");
        }

        clients.insert(id.clone(), tx);
        self.broadcast(
            &OutMessage::UserConnected { user: id },
            clients,
        )
        .await;

        Ok(())
    }

    pub async fn disconnect<'a>(&self, id: &'a str) {
        let mut clients = self.clients.lock().await;
        if clients.remove(id).is_some() {
            info!("disconnected {id}");
            let disconnect_msg = OutMessage::UserDisconnected {
                user: id.to_string(),
            };

            self.broadcast(&disconnect_msg, clients).await;
        }
    }

    async fn is_connected<'a>(&self, id: &'a str, clients_guard: &ClientsGuard<'_>) -> bool {
        clients_guard.contains_key(id)
    }

    pub async fn get_user_tx(&self, id: &str) -> Option<UnboundedSender<Message>> {
        self.clients.lock().await.get(id).cloned()
    }

    pub async fn list(&self) -> Vec<String> {
        self.clients.lock().await.keys().cloned().collect()
    }

    async fn broadcast(&self, msg: &OutMessage, clients_guard: ClientsGuard<'_>) {
        for tx in clients_guard.values() {
            let _ = tx.send(Message::text(serde_json::to_string(msg).unwrap()));
        }
    }
}
