use crate::messages::{ClientActorMessage, Connect, Disconnect, Join, Typing, WsMessage};
use crate::proto::*;
use crate::rooms::ChatRoom;
use actix::prelude::{Actor, Context, Handler, Recipient};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

type Socket = Recipient<WsMessage>;

/// The lobby keeps track of all available chatrooms that clients can connect
/// to and the socket for every connected client.
pub struct Lobby {
    sessions: HashMap<Uuid, Socket>, // self id to self
    rooms: HashMap<Uuid, ChatRoom>,  // room id to a chatroom.
}

impl Default for Lobby {
    fn default() -> Self {
        let mut lobby = Lobby {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
        };

        let default_room_id = Uuid::new_v4();

        lobby.rooms.insert(
            default_room_id,
            ChatRoom::new(default_room_id, "Default room".to_string(), 10),
        );

        let joels_room_id = Uuid::new_v4();

        lobby.rooms.insert(
            joels_room_id,
            ChatRoom::new(joels_room_id, "Joel's room".to_string(), 10),
        );

        lobby
    }
}

impl Lobby {
    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage(message.to_owned()));
        } else {
            println!("Attempting to send message but couldn't find client id.");
        }
    }

    // Sends a message to every client connected to a chatroom.
    fn send_to_everyone(&self, room_id: &Uuid, message: &String) {
        self.rooms
            .get(room_id)
            .unwrap()
            .clients
            .keys()
            .for_each(|client_id| self.send_message(message, client_id));
    }

    // Sends a message to every client connected to a chatroom except one client
    // specified by `self_id`.
    fn send_to_everyone_except_self(&self, room_id: &Uuid, self_id: &Uuid, message: &String) {
        self.rooms
            .get(room_id)
            .unwrap()
            .clients
            .keys()
            .filter(|client_id| *client_id.to_owned() != *self_id)
            .for_each(|client_id| self.send_message(message, client_id));
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

impl Handler<Connect> for Lobby {
    type Result = ();

    // When a WebSocket client is first connected, the WsConnect message is sent
    // on the server to send information about all the available rooms to the
    // client. This happens before the server gets information about what
    // username the client has and what room the client wants to join.
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        let _ = msg.addr.do_send(WsMessage(
            serde_json::to_string(&Output::Rooms(RoomsOutput::new(
                self.rooms
                    .values()
                    .map(|room| {
                        Room::new(
                            room.id.clone(),
                            room.name.clone(),
                            room.clients.len(),
                            room.max_clients,
                        )
                    })
                    .collect(),
            )))
            .unwrap(),
        ));
    }
}

impl Handler<Join> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        // Create a room if necessary.
        if self.rooms.get(&msg.lobby_id).is_none() {
            self.rooms.insert(
                msg.lobby_id.clone(),
                ChatRoom::new(msg.lobby_id.clone(), format!("{}'s room", msg.username), 10),
            );
        }

        // Echo to everyone in the room that a new client just joined.
        self.send_to_everyone_except_self(
            &msg.lobby_id,
            &msg.self_id,
            &serde_json::to_string(&Output::UserJoined(UserJoinedOutput::new(UserOutput::new(
                msg.self_id,
                &msg.username,
            ))))
            .unwrap(),
        );

        // Get a mutable reference to the current room.
        let current_room = self.rooms.get_mut(&msg.lobby_id).unwrap();

        // Add the client to the chatroom.
        current_room.add_client(&msg.self_id, msg.username.clone());

        // Store the address of the client in the sessions hashmap.
        self.sessions.insert(msg.self_id, msg.addr);

        // Get the chat history for the current room.
        let room_chat_history = current_room.history.clone();

        // Get all connected clients information from the current room.
        let connected_clients: Vec<UserOutput> = current_room
            .clients
            .iter()
            .map(|(client_id, client_name)| UserOutput::new(client_id.clone(), client_name))
            .collect();
        // Get all clients that are currently typing.
        let typing_clients = current_room.get_typing_clients();

        // Send the client information that the join was successful, along with
        // information about other connected clients and the history of the
        // chatroom.
        self.send_message(
            &serde_json::to_string(&Output::Joined(JoinedOutput::new(
                UserOutput::new(msg.self_id, &msg.username),
                connected_clients,
                room_chat_history,
                typing_clients,
            )))
            .unwrap(),
            &msg.self_id,
        );
    }
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.sessions.remove(&msg.self_id).is_some() {
            // Get a mutable reference to the current room.
            let current_room = self.rooms.get_mut(&msg.room_id).unwrap();

            // Clone the username from the current room.
            let username = current_room.get_username(&msg.self_id).unwrap().clone();

            // Remove the client from the current room.
            current_room.remove_client(&msg.self_id);

            // If the client was typing, send out a message that they've stopped
            // typing to all clients.
            if current_room.remove_typing_client(&msg.self_id) {
                self.send_to_everyone(
                    &msg.room_id,
                    &serde_json::to_string(&Output::Typing(TypingOutput::new(
                        TypingInput::Stopped,
                        UserOutput::new(msg.self_id.clone(), &username),
                    )))
                    .unwrap(),
                );
            }

            // Send message to all other clients in the same room that the
            // client has disconnected.
            self.send_to_everyone(
                &msg.room_id,
                &serde_json::to_string(&Output::UserLeft(UserLeftOutput::new(
                    msg.self_id,
                    &username,
                )))
                .unwrap(),
            );
        }
    }
}

impl Handler<Typing> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Typing, _: &mut Context<Self>) {
        // Get a mutable reference to the current room.
        let current_room = self.rooms.get_mut(&msg.room_id).unwrap();

        // Add or remove the client from the typing clients in the room.
        match msg.status {
            TypingInput::Started => current_room.add_typing_client(&msg.id),
            TypingInput::Stopped => {
                current_room.remove_typing_client(&msg.id);
            }
        }

        // Get the username from the current room.
        let username = current_room.get_username(&msg.id).unwrap();

        // Construct the message to send out to all other users.
        let message = serde_json::to_string(&Output::Typing(TypingOutput::new(
            msg.status,
            UserOutput::new(msg.id, username),
        )))
        .unwrap();

        // Echo to all other users that the client is typing.
        self.send_to_everyone_except_self(&msg.room_id, &msg.id, &message);
    }
}

impl Handler<ClientActorMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _: &mut Context<Self>) {
        // Timestamp for when the message was received.
        let timestamp: DateTime<Utc> = Utc::now();

        // Get the username from the current room.
        let username = self
            .rooms
            .get(&msg.room_id)
            .unwrap()
            .get_username(&msg.id)
            .unwrap();

        // Construct the message to be sent to all clients in the chat room.
        let message_output = MessageOutput::new(
            msg.id,
            UserOutput::new(msg.id, &username),
            &msg.msg,
            timestamp,
        );

        // Push the message to the history.
        self.rooms
            .get_mut(&msg.room_id)
            .unwrap()
            .history
            .push(message_output.clone());

        // Send the message to all other clients in the chatroom.
        self.send_to_everyone_except_self(
            &msg.room_id,
            &msg.id,
            &serde_json::to_string(&Output::UserPosted(UserPostedOutput::new(
                message_output.clone(),
            )))
            .unwrap(),
        );

        // Send information about the message to the client that sent it.
        self.send_message(
            &serde_json::to_string(&Output::Posted(PostedOutput::new(message_output))).unwrap(),
            &msg.id,
        );
    }
}
