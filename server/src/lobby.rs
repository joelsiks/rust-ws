use crate::messages::{ClientActorMessage, Connect, Disconnect, WsMessage};
use actix::prelude::{Actor, Context, Handler, Recipient};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use chrono::{DateTime, Utc};

use crate::proto::*;

type Socket = Recipient<WsMessage>;

pub struct Lobby {
    sessions: HashMap<Uuid, Socket>,     // self id to self
    rooms: HashMap<Uuid, HashSet<Uuid>>, // room id to the list of users id
    users: HashMap<Uuid, String>,        // self id to username
    history: Vec<MessageOutput>,         // chat history
}

impl Default for Lobby {
    fn default() -> Self {
        Lobby {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            users: HashMap::new(),
            history: vec![],
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
            .iter()
            .for_each(|user_id| self.send_message(message, user_id));
    }

    fn send_to_everyone_except_self(&self, room_id: &Uuid, self_id: &Uuid, message: &String) {
        self.rooms
            .get(room_id)
            .unwrap()
            .iter()
            .filter(|conn_id| *conn_id.to_owned() != *self_id)
            .for_each(|user_id| self.send_message(message, user_id));
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.sessions.remove(&msg.self_id).is_some() {
            // Retrieve the username from the uuid -> username hashmap.
            let username = self.users.get(&msg.self_id).unwrap();

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
                if lobby.len() > 1 {
                    lobby.remove(&msg.self_id);
                } else {
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
        // Create a room if necessary, and then add the id to it.
        self.rooms
            .entry(msg.lobby_id)
            .or_insert_with(HashSet::new)
            .insert(msg.self_id);

        // Insert the user into the uuid -> username hashmap.
        self.users.insert(msg.self_id, msg.username);

        // Get the reference for the username back from the hashmap.
        let username = self.users.get(&msg.self_id).unwrap();

        // Echo to everyone in the room that a new uuid just joined.
        self.send_to_everyone_except_self(
            &msg.lobby_id,
            &msg.self_id,
            &serde_json::to_string(&Output::UserJoined(UserJoinedOutput::new(UserOutput::new(
                msg.self_id,
                username,
            ))))
            .unwrap(),
        );

        // Store the address of the client in the sessions hashmap.
        self.sessions.insert(msg.self_id, msg.addr);

        let connected_users: Vec<UserOutput> = self
            .users
            .iter()
            .map(|(id, name)| UserOutput::new(id.clone(), name))
            .collect();

        // Send self the new uuid.
        self.send_message(
            &serde_json::to_string(&Output::Joined(JoinedOutput::new(
                UserOutput::new(msg.self_id, username),
                connected_users,
                self.history.clone(),
            )))
            .unwrap(),
            &msg.self_id,
        );
    }
}

impl Handler<ClientActorMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _: &mut Context<Self>) {
        let timestamp: DateTime<Utc> = Utc::now();
        let username = self.users.get(&msg.id).unwrap();

        let message_output = MessageOutput::new(
            msg.id,
            UserOutput::new(msg.id, username),
            &msg.msg,
            timestamp,
        );

        self.send_to_everyone_except_self(
            &msg.room_id,
            &msg.id,
            &serde_json::to_string(&Output::UserPosted(UserPostedOutput::new(
                message_output.clone(),
            )))
            .unwrap(),
        );

        self.send_message(
            &serde_json::to_string(&Output::Posted(PostedOutput::new(message_output.clone())))
                .unwrap(),
            &msg.id,
        );

        // Push the message to the history.
        self.history.push(message_output);
    }
}
