use crate::lobby::Lobby;
use crate::ws::ChatWebsocket;
use actix::Addr;
use actix_web::{get, web::Data, web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use uuid::Uuid;

#[get("/ws/")]
pub async fn start_connection(
    req: HttpRequest,
    stream: Payload,
    srv: Data<Addr<Lobby>>,
) -> Result<HttpResponse, Error> {
    let ws = ChatWebsocket::new(
        Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap(),
        srv.get_ref().clone(),
    );

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
