use std::{sync::MutexGuard, thread::LocalKey};

use actix_session::Session;
use actix_web::{
    post,
    web::{self, Json},
    HttpResponse, Responder,
};

use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, Message, Tokio1Executor,
};
use serde_json::json;

use crate::AppState;

use super::models::SignInMessage;

async fn sign_in(
    data: web::Data<AppState>,
    credentials: Json<SignInMessage>,
    session: Session,
) -> impl Responder {
    println!("Trying to sign in");

    let cred_values = credentials.into_inner();
    let creds = Credentials::new(
        cred_values.email.to_string(),
        cred_values.password.to_string(),
    );

    println!("Credentials {:?}", cred_values);
    let smtp_domain = "smtp.gmail.com";
    let imap_domain = "imap.gmail.com";

    // Enable SMTP session
    let smtp_session: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay(smtp_domain)
            .unwrap()
            .credentials(creds)
            .build();
    let smtp_test = smtp_session.test_connection().await;

    if let Err(smtp_error) = &smtp_test {
        return HttpResponse::Unauthorized().body(format!{"SMTP error: {}",smtp_error.to_string()});
    }

    // Enable IMAP session
    let tls = native_tls::TlsConnector::builder().build().unwrap();
    let client = imap::connect((imap_domain, 993), imap_domain, &tls).unwrap();
    let imap_session = client
        .login(
            cred_values.email.to_string(),
            cred_values.password.to_string(),
        )
        .map_err(|e| e.0);

    if let Err(imap_error) = &imap_session {
        return HttpResponse::Unauthorized().body(format!{"IMAP error: {}",imap_error.to_string()});
    }

    if let Ok(new_session) = imap_session {
        // Logout and save IMAP session into app state
        if let Some(ref current_imap_session) = data.imap_session {
            let mut locked_imap_session = current_imap_session.lock().await;
            locked_imap_session.logout();
            *locked_imap_session = new_session;
        }

        // Save SMTP session into app state
        if let Some(ref current_smtp_session) = data.smtp_session {
            *current_smtp_session.lock().await = smtp_session;
        }
       
        // Save email and password to session
        let mut result = session.insert("user_email", cred_values.email);
        if let Err(error) = result {
            println!("Email add to session error: {}", error);
        }

        result = session.insert("user_password", cred_values.password);
        if let Err(error) = result {
            println!("Password add to session error: {}", error);
        }

        println!("Session status: {:?}", session.status());
        println!("Session entries: {:?}", session.entries());

        HttpResponse::Ok().body("IMAP and SMTP sessions created")
    }
    else {
        HttpResponse::Unauthorized().body("Failed to establish sessions")
    }

    
}

async fn sign_out(session: Session) -> impl Responder {
    println!("Session status: {:?}", session.status());
    println!("Session entries: {:?}", session.entries());

    let email_result = session.get::<String>("user_email");

    if let Err(email_err) = &email_result {
        println!("Session error: {}", email_err);
    }

    if let Ok(Some(_)) = email_result {
        session.purge();
        HttpResponse::Ok().body("Successfully signed out")
    } else {
        HttpResponse::Unauthorized().body("Unauthorized signing out")
    }
}

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/auth/sign-in").route(web::post().to(sign_in)))
        .service(web::resource("/auth/sign-out").route(web::post().to(sign_out)));
}
