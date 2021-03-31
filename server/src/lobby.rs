use crate::messages::{ClientActorMessage, Connect, Disconnect, WsMessage};
use actix::prelude::{Actor, Context, Handler, Recipient};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

type Socket = Recipient<WsMessage>;

pub struct Lobby {
    sessions: HashMap<Uuid, Socket>,     // self id to self
    rooms: HashMap<Uuid, HashSet<Uuid>>, // room id to the list of users id
}

impl Default for Lobby {
    fn default() -> Self {
        Lobby {
            sessions: HashMap::new(),
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
            self.send_to_everyone_except_self(
                &msg.room_id,
                &msg.self_id,
                &format!("{} disconnected", &msg.self_id),
            );

            if let Some(lobby) = self.rooms.get_mut(&msg.room_id) {
                if lobby.len() > 1 {
                    lobby.remove(&msg.self_id);
                } else {
                    self.rooms.remove(&msg.room_id);
                }
            }
        }
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
        // Echo to everyone in the room that a new uuid just joined.
        self.send_to_everyone_except_self(
            &msg.lobby_id,
            &msg.self_id,
            &format!("{} just joined", msg.self_id),
        );
        // Store the address.
        self.sessions.insert(msg.self_id, msg.addr);

        // Send self the new uuid.
        self.send_message(&format!("your id is {}", msg.self_id), &msg.self_id);
    }
}

impl Handler<ClientActorMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _: &mut Context<Self>) {
        if msg.msg.starts_with("\\w") {
            if let Some(id_to) = msg.msg.split(' ').collect::<Vec<&str>>().get(1) {
                // TODO: Match on this parse.
                self.send_message(&msg.msg, &Uuid::parse_str(id_to).unwrap());
            }
        } else {
            self.send_to_everyone(&msg.room_id, &msg.msg);
        }
    }
}
