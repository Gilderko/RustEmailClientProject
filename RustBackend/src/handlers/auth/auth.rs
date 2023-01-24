use actix_session::Session;
use actix_web::{
    web::{self, Json},
    HttpResponse, Responder,
};

use crate::{
    constants::{AUTH_DOMAIN_STRING, AUTH_EMAIL_STRING, AUTH_PASSWORD_STRING},
    utils::utils_transports::{create_imap_session, create_smtp_transport},
};

use super::models::SignInMessage;

async fn sign_in(credentials: Json<SignInMessage>, session: Session) -> impl Responder {
    println!("Trying to sign in");
    let cred_values = credentials.into_inner();

    println!("Credentials {:?}", cred_values);
    let smtp_domain = "smtp.gmail.com".to_string();
    let imap_domain = "imap.gmail.com".to_string();

    // Enable SMTP session
    let smtp_session =
        create_smtp_transport(&cred_values.email, &cred_values.password, &smtp_domain).await;

    if let Err(smtp_error) = &smtp_session {
        return HttpResponse::Unauthorized()
            .body(format! {"SMTP error: {}",smtp_error.to_string()});
    }

    // Enable IMAP session
    let imap_session =
        create_imap_session(&cred_values.email, &cred_values.password, &imap_domain).await;

    if let Err(imap_error) = &imap_session {
        return HttpResponse::Unauthorized()
            .body(format! {"IMAP error: {}",imap_error.to_string()});
    }

    if let (Ok(mut imap), Ok(_)) = (imap_session, smtp_session) {
        // Save email and password to session
        let mut result = session.insert(AUTH_EMAIL_STRING, cred_values.email);
        if let Err(error) = result {
            return HttpResponse::Unauthorized()
                .body(format!("Email add to session error: {}", error));
        }

        result = session.insert(AUTH_PASSWORD_STRING, cred_values.password);
        if let Err(error) = result {
            return HttpResponse::Unauthorized()
                .body(format!("Password add to session error: {}", error));
        }

        result = session.insert(AUTH_DOMAIN_STRING, cred_values.domain);
        if let Err(error) = result {
            return HttpResponse::Unauthorized()
                .body(format!("Password add to session error: {}", error));
        }

        println!("Session status: {:?}", session.status());
        println!("Session entries: {:?}", session.entries());

        imap.logout().unwrap();
        HttpResponse::Ok().body("IMAP and SMTP sessions created")
    } else {
        HttpResponse::Unauthorized().body("Failed to establish sessions")
    }
}

async fn sign_out(session: Session) -> impl Responder {
    println!("Session status: {:?}", session.status());
    println!("Session entries: {:?}", session.entries());

    let email_result = session.get::<String>(AUTH_EMAIL_STRING);

    if let Ok(Some(_)) = email_result {
        session.purge();
        HttpResponse::Ok().body("Successfully signed out")
    } else {
        HttpResponse::Unauthorized().body("Unauthorized signing out")
    }
}

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("sign-in").route(web::post().to(sign_in)))
        .service(web::resource("sign-out").route(web::post().to(sign_out)));
}
