use std::{vec, path::{Path, PathBuf}};

use actix_files::NamedFile;
use actix_session::Session;
use actix_web::{http::header::{ContentType, ContentEncoding}, web, Error, HttpRequest, HttpResponse, Responder};
use imap::types::{Fetch, Flag};
use regex::bytes::Regex;

use crate::{
    handlers::email::{
        helper_models::AttachmentDescription,
        models::{EmailDetailOutDTO, EmailInspectOutDTO, EmailListOutDTO, EmailDetailAttachmentOutDTO},
    },
    utils::{utils_session::check_is_valid_session, utils_transports::create_imap_session},
};

use super::{
    helper_models::EmailAnalysis,
    models::{EmailDeleteInDTO, EmailDetailInDTO, EmailListInDTO, MailboxListOutDTO},
};

async fn get_email_in_detail_from_inbox(
    session: Session,
    request: web::Json<EmailDetailInDTO>,
) -> impl Responder {
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
            "(FLAGS BODYSTRUCTURE BODY[TEXT] ENVELOPE INTERNALDATE)",
        )
        .unwrap()[0];
    println!(
        "Text Section {}",
        String::from_utf8(
            email_message_raw
                .section(&imap_proto::SectionPath::Full(
                    imap_proto::MessageSection::Text
                ))
                .unwrap()
                .to_vec()
        )
        .unwrap()
    );
    let structure = email_message_raw.bodystructure().unwrap();

    let mut description = EmailAnalysis {
        plain_text_octets: 0,
        attachments: vec![],
    };
    parse_body_structure(structure, email_message_raw, &mut description);

    println!("Description: {:?}", description);
    let body_bytes = email_message_raw.text().unwrap();
    let limit_low = description.attachments.iter().fold(0, |x,y| x + y.size_octets) as usize;
    let text_bytes = &body_bytes[0 .. (body_bytes.len() - limit_low)];

    let mut body = String::from_utf8(text_bytes.to_vec()).unwrap();
    let regex_string = format!(r"\r\n\r\n([\S\s]{{{}}})\r\n", description.plain_text_octets);
    println!("Regex string: {}", regex_string);
    let regex = Regex::new(&regex_string).unwrap();
    let body_matches = regex.captures_iter(email_message_raw.text().unwrap());
    
    println!("RegexMatch:");
    for body_match in body_matches {
        println!("Body match: {:?}", body_match);
        body = String::from_utf8(body_match[1].to_vec()).unwrap();
        break;
    }

    let sender_bytes = email_message_raw.envelope().unwrap().from.as_ref().unwrap()[0]
        .mailbox
        .unwrap_or_default();

    let subject_bytes = email_message_raw
        .envelope()
        .unwrap()
        .subject
        .unwrap_or_default();

    let sender = String::from_utf8(sender_bytes.to_vec()).unwrap_or_default();
    let subject = String::from_utf8(subject_bytes.to_vec()).unwrap_or_default();
    let send_date = email_message_raw
        .internal_date()
        .unwrap_or_default()
        .naive_utc();

    let mut response = EmailDetailOutDTO {
        from_address: sender,
        subject: subject,
        send_date: send_date,
        body_text: body,
        attachments: vec![],
    };

    for attach_info in description.attachments {
        let attach = EmailDetailAttachmentOutDTO { file_name: attach_info.file_name, size_octets: attach_info.size_octets, is_file: attach_info.is_file };
        response.attachments.push(attach);
    }

    imap_session.logout().unwrap();
    response
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

fn parse_body_structure(
    structure: &imap_proto::BodyStructure,
    message: &Fetch,
    description: &mut EmailAnalysis,
) {
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
            let file_name = common
                .ty
                .params
                .as_ref()
                .unwrap()
                .iter()
                .find(|(x, _)| *x == "NAME")
                .unwrap_or_else(|| &("", ""));
            let file_size = other.octets;
            let attachment_description = AttachmentDescription {
                file_name: file_name.1.to_string(),
                size_octets: file_size,
                is_file: true,
            };

            description.attachments.push(attachment_description);
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

            if common.ty.ty == "TEXT" && common.ty.subtype == "PLAIN" {
                description.plain_text_octets = other.octets;
            } else {
                let attachment_description = AttachmentDescription {
                    file_name: String::new(),
                    size_octets: other.octets,
                    is_file: false,
                };

                description.attachments.push(attachment_description);
            }
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
                parse_body_structure(body, message, description);
                println!("Next header equal depth");
            }
            println!("Recursion comming out");
        }
    }
}

async fn download_attachment_from_email(session: Session) -> Result<NamedFile, Error> {
    let path: PathBuf = "./tmp/T_ES.tese".parse().unwrap();
    Ok(NamedFile::open(path)?)
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
    let mut mailbox_names: Vec<String> = vec![];

    for mailbox in mailboxes.iter() {
        println!("Mailbox: {:?}", mailbox);
        mailbox_names.push(mailbox.name().to_string());
    }

    let response = MailboxListOutDTO { mailbox_names };

    response
}

pub fn email_imap_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/email")
            .route(web::get().to(list_emails_from_inbox))
            .route(web::delete().to(delete_email_from_inbox)),
    )
    .service(web::resource("/mailbox").route(web::get().to(get_mailboxes)))
    .service(web::resource("/emailDetail").route(web::get().to(get_email_in_detail_from_inbox)))
    .service(web::resource("/attachment").route(web::get().to(download_attachment_from_email)));
}
