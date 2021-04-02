mod lobby;
mod messages;
mod proto;
mod rooms;
mod start_connection;
mod ws;

use actix::Actor;
use actix_web::{App, HttpServer};
use lobby::Lobby;
use start_connection::start_connection as start_connection_route;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let chat_server = Lobby::default().start();

    HttpServer::new(move || {
        App::new()
            .service(start_connection_route)
            .data(chat_server.clone())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
