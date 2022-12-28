use actix_session::{
    config::{CookieContentSecurity, PersistentSession},
    storage::CookieSessionStore,
    SessionMiddleware,
};
use actix_web::{
    cookie::{time::Duration, Key},
    web, App, HttpResponse, HttpServer,
};
use constants::auth_cookie_name;
use dotenv::dotenv;
use handlers::auth::auth::auth_config;
use imap::Session;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use native_tls::TlsStream;
use std::{env, net::TcpStream};
use tokio::sync::Mutex;
use utils::auth_guards::AuthGuardFactory;

mod constants;
mod handlers;
mod utils;

pub struct AppState {
    imap_session: Option<Mutex<Session<TlsStream<TcpStream>>>>,
    smtp_session: Option<Mutex<AsyncSmtpTransport<Tokio1Executor>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let secret_key: Key = Key::derive_from(
        env::var("ENCRYPTION_KEY")
            .expect("ENCRYPTION_KEY must be set")
            .to_string()
            .as_bytes(),
    );

    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false)
                    .cookie_http_only(false)
                    .cookie_content_security(CookieContentSecurity::Signed)
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::hours(2)))
                    .cookie_name(auth_cookie_name.to_string())
                    .build(),
            )
            .wrap(AuthGuardFactory)
            .configure(app_config)
            .service(web::scope("/api").configure(auth_config))
            .app_data(web::Data::new(AppState {
                imap_session: None,
                smtp_session: None,
            }))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}

fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/app").route(web::get().to(|| async { HttpResponse::Ok().body("app") })),
    );
}
