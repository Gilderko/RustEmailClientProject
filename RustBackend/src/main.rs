use actix_cors::Cors;
use actix_session::{
    config::{CookieContentSecurity, PersistentSession},
    storage::CookieSessionStore,
    SessionMiddleware,
};
use actix_web::{
    cookie::{time::Duration, Key, SameSite},
    web, App, HttpResponse, HttpServer,
};
use constants::AUTH_COOKIE_NAME;
use dotenv::dotenv;
use handlers::{
    auth::auth::auth_config,
    email::{email_imap::email_imap_config, email_smtp::email_smtp_config},
};
use std::env;
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
            .as_bytes(),
    );

    let port = match env::var("PORT") {
        Ok(number) => number.parse::<u16>()?,
        Err(_) => 8080,
    };

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    // .allowed_origin("http://localhost:5173/")
                    .allowed_methods(vec!["GET", "POST", "DELETE"])
                    // .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT, http::header::ACCESS_CONTROL_ALLOW_ORIGIN])
                    // .allowed_header(http::header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false)
                    .cookie_http_only(false)
                    .cookie_content_security(CookieContentSecurity::Signed)
                    .cookie_same_site(SameSite::Lax)
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
    .bind(("127.0.0.1", port))?
    .run()
    .await?;

    Ok(())
}

fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/app").route(web::get().to(|| async { HttpResponse::Ok().body("app") })),
    );
}
