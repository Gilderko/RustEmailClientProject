use actix_session::Session;
use actix_web::{web, Error, HttpResponse, Responder};

use crate::utils::{utils_session::check_is_valid_session, utils_transports::create_imap_session};

async fn get_email_in_detail_from_inbox() -> impl Responder {
    HttpResponse::Ok()
}

async fn delete_email_from_inbox() -> impl Responder {
    HttpResponse::Ok()
}

async fn list_emails_from_inbox(session: Session) -> Result<HttpResponse, Error> {
    let credentials = check_is_valid_session(&session).unwrap();
    let mut imap_session = create_imap_session(
        &credentials.email,
        &credentials.password,
        &("imap.gmail.com".to_string()),
    )
    .await
    .unwrap();

    let mailbox_info = imap_session.select("INBOX").unwrap();
    println!("Mailbox info: \nEmails: {}", mailbox_info.exists);
    let start_number = mailbox_info.exists;
    let end_number = mailbox_info.exists;

    let messages = imap_session
        .fetch(
            format!("{}:{}", end_number, start_number),
            "(FLAGS BODY[TEXT] RFC822.SIZE ENVELOPE)",
        )
        .unwrap();
    for message in messages.into_iter() {
        let sender = message.envelope().unwrap().from.as_ref().unwrap()[0]
            .mailbox
            .unwrap();
        let sender = std::str::from_utf8(sender)
            .expect("sender was not valid utf-8")
            .to_string();

        let body = message.text().unwrap();
        let body = std::str::from_utf8(body)
            .expect("message was not valid utf-8")
            .to_string();

        println!("Message sender: {}\nMessage body: \n{}", sender, body);
    }

    imap_session.logout();
    Ok(HttpResponse::Ok().body("Ok"))
}

async fn download_attachment_from_email(session: Session) -> impl Responder {
    HttpResponse::Ok()
}

pub fn email_imap_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/email/list").route(web::get().to(list_emails_from_inbox)));
}
