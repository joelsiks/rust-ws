use crate::messages::{ClientActorMessage, Connect, Disconnect, WsConnect, WsMessage};
use crate::rooms::ChatRoom;
use actix::prelude::{Actor, Context, Handler, Recipient};
use std::collections::HashMap;
use uuid::Uuid;

use chrono::{DateTime, Utc};

use crate::proto::*;

type Socket = Recipient<WsMessage>;

pub struct Lobby {
    sessions: HashMap<Uuid, Socket>, // self id to self
    users: HashMap<Uuid, String>,    // self id to username
    rooms: HashMap<Uuid, ChatRoom>,  // room id to the list of users id
}

impl Default for Lobby {
    fn default() -> Self {
        Lobby {
            sessions: HashMap::new(),
            users: HashMap::new(),
            rooms: HashMap::new(),
        }
    }
}

impl Lobby {
    // TODO: Change the return type of this function to a Result.
    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage(message.to_owned()));
        } else {
            println!("Attempting to send message but couldn't find user id.");
        }
    }

    fn send_to_everyone(&self, room_id: &Uuid, message: &String) {
        self.rooms
            .get(room_id)
            .unwrap()
            .clients
            .keys()
            .for_each(|client_id| self.send_message(message, client_id));
    }

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

impl Handler<WsConnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: WsConnect, _: &mut Context<Self>) {
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

impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.sessions.remove(&msg.self_id).is_some() {
            // Get the username from the current room.
            let username = self
                .rooms
                .get(&msg.room_id)
                .unwrap()
                .get_username(&msg.self_id)
                .unwrap();

            self.send_to_everyone_except_self(
                &msg.room_id,
                &msg.self_id,
                &serde_json::to_string(&Output::UserLeft(UserLeftOutput::new(
                    msg.self_id,
                    username,
                )))
                .unwrap(),
            );

            if let Some(lobby) = self.rooms.get_mut(&msg.room_id) {
                if lobby.clients.len() > 1 {
                    lobby.remove_client(&msg.self_id);
                } else {
                    // TODO: Decide if to remove rooms when they are empty or not!
                    self.rooms.remove(&msg.room_id);
                }
            }
        }

        self.users.remove(&msg.self_id);
    }
}

impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        // Create a room if necessary.
        if self.rooms.get(&msg.lobby_id).is_none() {
            self.rooms.insert(
                msg.lobby_id.clone(),
                ChatRoom::new(msg.lobby_id.clone(), String::from("test_lobby"), 10),
            );
        }

        // Echo to everyone in the room that a new uuid just joined.
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

        // Get all connected users information from the current room.
        let connected_users: Vec<UserOutput> = current_room
            .clients
            .iter()
            .map(|(client_id, client_name)| UserOutput::new(client_id.clone(), client_name))
            .collect();

        // Send the client information that the join was successful.
        self.send_message(
            &serde_json::to_string(&Output::Joined(JoinedOutput::new(
                UserOutput::new(msg.self_id, &msg.username),
                connected_users,
                room_chat_history,
            )))
            .unwrap(),
            &msg.self_id,
        );
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

        self.send_to_everyone_except_self(
            &msg.room_id,
            &msg.id,
            &serde_json::to_string(&Output::UserPosted(UserPostedOutput::new(
                message_output.clone(),
            )))
            .unwrap(),
        );

        self.send_message(
            &serde_json::to_string(&Output::Posted(PostedOutput::new(message_output))).unwrap(),
            &msg.id,
        );
    }
}
