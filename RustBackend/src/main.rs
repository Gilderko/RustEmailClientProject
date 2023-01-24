use actix_session::{
    config::{CookieContentSecurity, PersistentSession},
    storage::CookieSessionStore,
    SessionMiddleware,
};
use actix_web::{
    cookie::{time::Duration, Key},
    web, App, HttpResponse, HttpServer,
};
use constants::AUTH_COOKIE_NAME;
use dotenv::dotenv;
use handlers::{
    auth::auth::auth_config,
    email::{email_imap::email_imap_config, email_smtp::email_smtp_config},
};
use imap::Session;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use native_tls::TlsStream;
use std::{env, net::TcpStream};
use tokio::sync::Mutex;
use utils::auth_guards::AuthGuardFactory;

mod constants;
mod handlers;
mod utils;

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
                    .cookie_name(AUTH_COOKIE_NAME.to_string())
                    .build(),
            )
            .configure(app_config)
            .service(web::scope("/auth").configure(auth_config))
            .service(
                web::scope("/api")
                    .configure(email_smtp_config)
                    .configure(email_imap_config)
                    .wrap(AuthGuardFactory),
            )
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
