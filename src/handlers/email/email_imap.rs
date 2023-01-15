use std::vec;

use actix_session::Session;
use actix_web::{http::header::ContentType, web, Error, HttpRequest, HttpResponse, Responder};
use imap::types::{Fetch, Flag};

use crate::{
    handlers::email::models::{EmailDetailOutDTO, EmailInspectOutDTO, EmailListOutDTO},
    utils::{utils_session::check_is_valid_session, utils_transports::create_imap_session},
};

use super::models::{EmailDeleteInDTO, EmailDetailInDTO, EmailListInDTO, MailboxListOutDTO};

async fn get_email_in_detail_from_inbox(
    session: Session,
    request: web::Json<EmailDetailInDTO>,
) -> Result<HttpResponse, Error> {
    let credentials = check_is_valid_session(&session).unwrap();
    let mut imap_session = create_imap_session(
        &credentials.email,
        &credentials.password,
        &("imap.gmail.com".to_string()),
    )
    .await
    .unwrap();

    println!("Request: {:?}", request);
    imap_session.select(&request.mailbox_name).unwrap();
    let email_message_raw = &imap_session
        .fetch(
            format!("{}", request.sequence_number),
            "(FLAGS BODYSTRUCTURE BODY[TEXT] RFC822.SIZE ENVELOPE)",
        )
        .unwrap()[0];

    let structure = email_message_raw.bodystructure().unwrap();

    let email_result = EmailDetailOutDTO {
        from_address: todo!(),
        subject: todo!(),
        attachment_count: todo!(),
        send_date: todo!(),
        body_text: todo!(),
    };

    imap_session.logout().unwrap();
    Ok(HttpResponse::Ok().body("Ok"))
}

async fn delete_email_from_inbox(
    session: Session,
    request: web::Json<EmailDeleteInDTO>,
) -> Result<HttpResponse, Error> {
    let credentials = check_is_valid_session(&session).unwrap();
    let mut imap_session = create_imap_session(
        &credentials.email,
        &credentials.password,
        &("imap.gmail.com".to_string()),
    )
    .await
    .unwrap();

    println!("Request: {:?}", request);
    imap_session.select(&request.mailbox_name).unwrap();
    imap_session
        .store(
            format!(
                "{}:{}",
                request.sequence_set_top, request.sequence_set_bottom
            ),
            "+FLAGS (\\Deleted)",
        )
        .unwrap();
    imap_session.expunge().unwrap();

    imap_session.logout().unwrap();
    Ok(HttpResponse::Ok().body("Ok"))
}

async fn list_emails_from_inbox(
    session: Session,
    request: web::Json<EmailListInDTO>,
) -> impl Responder {
    let credentials = check_is_valid_session(&session).unwrap();
    let mut imap_session = create_imap_session(
        &credentials.email,
        &credentials.password,
        &("imap.gmail.com".to_string()),
    )
    .await
    .unwrap();

    let mailbox_info = imap_session.select(&request.mailbox_name).unwrap();
    println!("Mailbox info: {:?}", mailbox_info);

    let start_number = mailbox_info.exists - request.requested_page_number * request.page_size;
    let end_number = start_number - request.page_size;

    let messages_raw = imap_session
        .fetch(
            format!("{}:{}", end_number, start_number),
            "(FLAGS BODYSTRUCTURE BODY[TEXT] RFC822.SIZE ENVELOPE INTERNALDATE)",
        )
        .unwrap();

    let mut messages_out: Vec<EmailInspectOutDTO> = vec![];

    for message in messages_raw.into_iter() {
        let sender_bytes = message.envelope().unwrap().from.as_ref().unwrap()[0]
            .mailbox
            .unwrap_or_default();

        let subject_bytes = message.envelope().unwrap().subject.unwrap_or_default();

        let sender = String::from_utf8(sender_bytes.to_vec()).unwrap_or_default();
        let subject = String::from_utf8(subject_bytes.to_vec()).unwrap_or_default();
        let was_read = message.flags().contains(&Flag::Seen);
        let send_date = message.internal_date().unwrap_or_default().naive_utc();

        println!(
            "Body: {}",
            String::from_utf8(message.text().unwrap().to_vec()).unwrap()
        );

        let message_out = EmailInspectOutDTO {
            from_address: sender,
            subject: subject,
            was_read: was_read,
            send_date: send_date,
        };

        messages_out.push(message_out);
    }

    let response = EmailListOutDTO {
        total_emails_count: mailbox_info.exists,
        requested_page_number: request.page_size,
        page_size: request.page_size,
        emails: messages_out,
    };

    imap_session.logout().unwrap();
    response
}

fn describe_structure(structure: &imap_proto::BodyStructure, message: &Fetch) {
    match structure {
        imap_proto::BodyStructure::Basic {
            common,
            other,
            extension,
        } => {
            println!("Basic body structure");
            println!(
                "BodyContentCommon: {:?}, BodyContentSinglePart: {:?}",
                common, other
            );
        }
        imap_proto::BodyStructure::Text {
            common,
            other,
            lines,
            extension,
        } => {
            println!("Text body structure");
            println!(
                "BodyContentCommon: {:?}, BodyContentSinglePart: {:?}, Lines {}",
                common, other, lines
            );
        }
        imap_proto::BodyStructure::Message {
            common,
            other,
            envelope,
            body,
            lines,
            extension,
        } => {
            println!("Message body structure");
            println!(
                "BodyContentCommon: {:?}, BodyContentSinglePart: {:?}, Envelope: {:?}, Lines {}",
                common, other, envelope, lines
            );
        }
        imap_proto::BodyStructure::Multipart {
            common,
            bodies,
            extension,
        } => {
            println!("Multipart body structure");
            println!("BodyContentCommon: {:?}", common);
            for body in bodies {
                describe_structure(body, message);
                println!("Next header equal depth");
            }
            println!("Recursion comming out");
        }
    }
}

async fn download_attachment_from_email(session: Session) -> impl Responder {
    HttpResponse::Ok()
}

async fn get_mailboxes(session: Session) -> impl Responder {
    let credentials = check_is_valid_session(&session).unwrap();
    let mut imap_session = create_imap_session(
        &credentials.email,
        &credentials.password,
        &("imap.gmail.com".to_string()),
    )
    .await
    .unwrap();

    let mailboxes = imap_session.list(None, Some("*")).unwrap();
    let mut mailbox_names: Vec<String> = vec!();
    
    for mailbox in mailboxes.iter(){
        println!("Mailbox: {:?}", mailbox);
        mailbox_names.push(mailbox.name().to_string());
    }

    let response = MailboxListOutDTO {
        mailbox_names,
    };
    
    response
}

pub fn email_imap_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/email")
            .route(web::get().to(list_emails_from_inbox))
            .route(web::delete().to(delete_email_from_inbox)),
    )
    .service(web::resource("/mailbox").route(web::get().to(get_mailboxes)));
}
