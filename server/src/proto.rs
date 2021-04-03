use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOutput {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageOutput {
    pub id: Uuid,
    pub user: UserOutput,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

// Used to represent a chatroom with connected users.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub connected_clients: usize,
    pub max_clients: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum Input {
    #[serde(rename = "join")]
    Join(JoinInput),
    #[serde(rename = "leave")]
    Leave,
    #[serde(rename = "post")]
    Post(PostInput),
    #[serde(rename = "typing")]
    Typing(TypingInput),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinInput {
    pub username: String,
    pub room: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostInput {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TypingInput {
    #[serde(rename = "started")]
    Started,
    #[serde(rename = "stopped")]
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Output {
    #[serde(rename = "error")]
    Error(OutputError),
    #[serde(rename = "rooms")]
    Rooms(RoomsOutput),
    #[serde(rename = "joined")]
    Joined(JoinedOutput),
    #[serde(rename = "user-joined")]
    UserJoined(UserJoinedOutput),
    #[serde(rename = "user-left")]
    UserLeft(UserLeftOutput),
    #[serde(rename = "posted")]
    Posted(PostedOutput),
    #[serde(rename = "user-posted")]
    UserPosted(UserPostedOutput),
    #[serde(rename = "user-typing")]
    Typing(TypingOutput),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "code")]
pub enum OutputError {
    #[serde(rename = "name-taken")]
    NameTaken,
    #[serde(rename = "invalid-name")]
    InvalidName,
    #[serde(rename = "not-joined")]
    NotJoined,
    #[serde(rename = "invalid-message-body")]
    InvalidMessageBody,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomsOutput {
    pub rooms: Vec<Room>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinedOutput {
    pub user: UserOutput,
    pub others: Vec<UserOutput>,
    pub messages: Vec<MessageOutput>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserJoinedOutput {
    pub user: UserOutput,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserLeftOutput {
    pub user: UserOutput,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostedOutput {
    pub message: MessageOutput,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPostedOutput {
    pub message: MessageOutput,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypingOutput {
    pub status: TypingInput,
    pub user: UserOutput,
}

impl UserOutput {
    pub fn new(id: Uuid, name: &str) -> Self {
        UserOutput {
            id,
            name: String::from(name),
        }
    }
}

impl MessageOutput {
    pub fn new(id: Uuid, user: UserOutput, body: &str, created_at: DateTime<Utc>) -> Self {
        MessageOutput {
            id,
            user,
            body: String::from(body),
            created_at,
        }
    }
}

impl Room {
    pub fn new(id: Uuid, name: String, connected_clients: usize, max_clients: usize) -> Self {
        Room {
            id,
            name,
            connected_clients,
            max_clients,
        }
    }
}

impl RoomsOutput {
    pub fn new(rooms: Vec<Room>) -> Self {
        RoomsOutput { rooms }
    }
}

impl JoinedOutput {
    pub fn new(user: UserOutput, others: Vec<UserOutput>, messages: Vec<MessageOutput>) -> Self {
        JoinedOutput {
            user,
            others,
            messages,
        }
    }
}

impl UserJoinedOutput {
    pub fn new(user: UserOutput) -> Self {
        UserJoinedOutput { user }
    }
}

impl UserLeftOutput {
    pub fn new(user_id: Uuid, username: &String) -> Self {
        UserLeftOutput {
            user: UserOutput::new(user_id, username),
        }
    }
}

impl PostedOutput {
    pub fn new(message: MessageOutput) -> Self {
        PostedOutput { message }
    }
}

impl UserPostedOutput {
    pub fn new(message: MessageOutput) -> Self {
        UserPostedOutput { message }
    }
}

impl TypingOutput {
    pub fn new(status: TypingInput, user: UserOutput) -> Self {
        TypingOutput { status, user }
    }
}
