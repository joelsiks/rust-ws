use crate::proto::MessageOutput;
use std::collections::HashMap;
use uuid::Uuid;

pub struct ChatRoom {
    pub id: Uuid,
    pub name: String,

    pub max_clients: usize,

    // Set of connected user's ids.
    pub clients: HashMap<Uuid, String>,

    // Chat history.
    pub history: Vec<MessageOutput>,
}

impl ChatRoom {
    pub fn new(id: Uuid, name: String, max_clients: usize) -> Self {
        ChatRoom {
            id,
            name,
            max_clients,
            clients: HashMap::new(),
            history: vec![],
        }
    }

    /// Adds a new client to the set of connected clients by cloning the passed
    /// client id.
    pub fn add_client(&mut self, client_id: &Uuid, username: String) {
        self.clients.insert(client_id.clone(), username);
    }

    /// Removes a client from the set of connected clients.
    pub fn remove_client(&mut self, client_id: &Uuid) -> Option<String> {
        self.clients.remove(client_id)
    }

    pub fn get_username(&self, client_id: &Uuid) -> Option<&String> {
        self.clients.get(client_id)
    }
}
