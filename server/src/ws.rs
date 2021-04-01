use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;
use uuid::Uuid;

use crate::lobby::Lobby;
use crate::messages::{ClientActorMessage, Connect, Disconnect, WsMessage};
use crate::proto::*;

// How often heartbeat pings are sent.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

// How long before lack of client response causes a timeout.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// WebSocket connections is a "long running" connection,
// so we want to handle it with an "actor"?
pub struct ChatWebsocket {
    room: Uuid,
    lobby_addr: Addr<Lobby>,
    hb: Instant,
    id: Uuid,
}

impl ChatWebsocket {
    pub fn new(room: Uuid, lobby: Addr<Lobby>) -> Self {
        ChatWebsocket {
            room,
            lobby_addr: lobby,
            hb: Instant::now(),
            id: Uuid::new_v4(),
        }
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("WebSocket client heartbeat failed, disconnecting.");

                act.lobby_addr.do_send(Disconnect {
                    self_id: act.id,
                    room_id: act.room,
                });

                ctx.stop();

                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for ChatWebsocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        self.lobby_addr.do_send(Disconnect {
            self_id: self.id,
            room_id: self.room,
        });
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatWebsocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // Process websocket messages
        match msg {
            Ok(ws::Message::Pong(_)) => (),
            _ => println!("WS: {:?}", msg),
        }

        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Ok(ws::Message::Text(text)) => {
                let result: serde_json::Result<Input> = serde_json::from_str(&text);
                if let Ok(input) = result {
                    match input {
                        Input::Join(inp) => {
                            self.lobby_addr.do_send(Connect {
                                addr: ctx.address().recipient(),
                                lobby_id: self.room,
                                self_id: self.id,
                                username: inp.username,
                            });
                        }
                        Input::Post(inp) => self.lobby_addr.do_send(ClientActorMessage {
                            id: self.id,
                            msg: inp.message,
                            room_id: self.room,
                        }),
                    };
                } else {
                    // TODO: Send error message (and close connection??).
                }
            }
            Err(e) => panic!(e), // TODO: Change this panic to something else (log and disconnect?).
        }
    }
}

impl Handler<WsMessage> for ChatWebsocket {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}