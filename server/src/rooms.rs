use crate::proto::{MessageOutput, UserOutput};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Used to represent a chat room which multiple clients can connect to and
/// chat with each other. Currently each chat room keeps track of its own history
/// of messages, but this should in the future be moved to an external database.
pub struct ChatRoom {
    /// Used to identify the chat room.
    pub id: Uuid,
    pub name: String,

    /// Maximum amount of clients allowed to connect to the chat room.
    pub max_clients: usize,

    /// Maps uuid to username for all connected clients.
    pub clients: HashMap<Uuid, String>,

    /// Clients that are currently typing in the chat room.
    pub typing_clients: HashSet<Uuid>,

    /// Chat history.
    pub history: Vec<MessageOutput>,
}

impl ChatRoom {
    pub fn new(id: Uuid, name: String, max_clients: usize) -> Self {
        ChatRoom {
            id,
            name,
            max_clients,
            clients: HashMap::new(),
            typing_clients: HashSet::new(),
            history: Vec::new(),
        }
    }

    /// Adds a new client to the set of connected clients.
    pub fn add_client(&mut self, client_id: &Uuid, username: String) {
        self.clients.insert(client_id.clone(), username);
    }

    /// Removes a client from the set of connected clients.
    pub fn remove_client(&mut self, client_id: &Uuid) -> Option<String> {
        self.clients.remove(client_id)
    }

    /// Retrieves a username from the (id -> username) client hashmap.
    pub fn get_username(&self, client_id: &Uuid) -> Option<&String> {
        self.clients.get(client_id)
    }

    /// Adds a client to the list of typing clients.
    pub fn add_typing_client(&mut self, client_id: &Uuid) {
        self.typing_clients.insert(client_id.clone());
    }

    /// Removes a client from the list of typing clients.
    pub fn remove_typing_client(&mut self, client_id: &Uuid) {
        self.typing_clients.remove(client_id);
    }

    /// Returns a list of all clients (id, username) who are currently typing
    /// in the chat room.
    pub fn get_typing_clients(&self) -> Vec<UserOutput> {
        self.typing_clients
            .iter()
            .map(|client_id| {
                UserOutput::new(client_id.clone(), self.get_username(client_id).unwrap())
            })
            .collect()
    }
}
