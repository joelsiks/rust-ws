use actix::prelude::{Message, Recipient};
use uuid::Uuid;

// ChatWebsocket responds to this to pipe it though to the actual client.
#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
}

// ChatWebsocket sends this to connect to a lobby.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub addr: Recipient<WsMessage>,
    pub lobby_id: Uuid,
    pub self_id: Uuid,
    pub username: String,
}

// ChatWebsocket sends this to disconnect from a lobby.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub room_id: Uuid,
    pub self_id: Uuid,
}

// Client sends this to the lobby for the lobby to echo it out.
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientActorMessage {
    pub id: Uuid,
    pub msg: String,
    pub room_id: Uuid,
}
