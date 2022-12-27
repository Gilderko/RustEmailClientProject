use actix_web::{HttpServer, App};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    HttpServer::new(move || App::new())
        .bind(("127.0.0.1", 8090))?
        .run()
        .await?;

    Ok(())
}
