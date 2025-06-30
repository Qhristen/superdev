use actix_web::{web, App, HttpServer};

mod handler;
mod response;
mod error;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(handler::hello))
            .route("/keypair", web::post().to(handler::generate_keypair))
            .route("/token/create", web::post().to(handler::create_token))
            .route("/token/mint", web::post().to(handler::mint_token))
            .route("/message/sign", web::post().to(handler::sign_message))
            .route("/message/verify", web::post().to(handler::verify_message))
            .route("/send/sol", web::post().to(handler::send_sol))
            .route("/send/token", web::post().to(handler::send_token))
    })
    .bind("127.0.0.1:8090")?
    .run()
    .await
}