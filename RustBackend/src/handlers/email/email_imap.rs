use std::{ops::Add, vec, cmp::{min, max}};

use actix_session::Session;
use actix_web::{
    http::header::{ContentDisposition, ContentEncoding, DispositionParam, DispositionType},
    web, Error, HttpResponse, Responder,
};
use imap::types::{Fetch, Flag, NameAttribute};
use regex::bytes::Regex;

use crate::{
    handlers::email::{
        helper_models::{EmailPartDescription, EncodingType},
        models::{
            EmailDetailAttachmentOutDTO, EmailDetailOutDTO, EmailInspectOutDTO, EmailListOutDTO,
        },
    },
    utils::{utils_session::check_is_valid_session, utils_transports::create_imap_session},
};

use super::{
    helper_models::EmailAnalysis,
    models::{
        EmailAttachmentInDTO, EmailDeleteInDTO, EmailDetailInDTO, EmailListInDTO, MailboxListOutDTO,
    },
};

use rustyknife::rfc2047::encoded_word;
use utf7_imap::{decode_utf7_imap, encode_utf7_imap};

async fn get_email_in_detail_from_inbox(
    session: Session,
    request: web::Query<EmailDetailInDTO>,
) -> impl Responder {
    let credentials = check_is_valid_session(&session).unwrap();
    let mut imap_session = create_imap_session(
        &credentials.email,
        &credentials.password,
        &credentials.get_imap_string(),
    )
    .await
    .unwrap();

    println!("Request: {:?}", request);
    imap_session
        .select(&encode_utf7_imap(request.mailbox_name.clone()))
        .unwrap();
    let email_message_raw = &imap_session
        .fetch(
            format!("{}", request.sequence_number),
            "(FLAGS BODYSTRUCTURE BODY[TEXT] ENVELOPE INTERNALDATE)",
        )
        .unwrap()[0];

    let structure = email_message_raw.bodystructure().unwrap();

    let mut description = EmailAnalysis {
        plain_text_octets: 0,
        attachments: vec![],
    };
    parse_body_structure(
        structure,
        email_message_raw,
        &mut description,
        String::new(),
        0,
    );

    let send_date = email_message_raw
        .internal_date()
        .unwrap_or_default()
        .naive_utc();

    let sender_bytes = email_message_raw
        .envelope()
        .unwrap()
        .from
        .as_ref()
        .unwrap_or(&vec![])[0]
        .mailbox
        .unwrap_or_default();

    let sender_host_bytes = email_message_raw
        .envelope()
        .unwrap()
        .from
        .as_ref()
        .unwrap_or(&vec![])[0]
        .host
        .unwrap_or_default();

    let subject_bytes = email_message_raw
        .envelope()
        .unwrap()
        .subject
        .unwrap_or_default();

    let sender = String::from_utf8(sender_bytes.to_vec()).unwrap_or_default()
        + "@"
        + &String::from_utf8(sender_host_bytes.to_vec()).unwrap_or_default();
    let (_, subject) = encoded_word(subject_bytes).unwrap_or((
        subject_bytes,
        String::from_utf8(subject_bytes.to_vec()).unwrap_or("Cant parse subject".to_string()),
    ));

    let mut response = EmailDetailOutDTO {
        from_address: sender,
        subject,
        send_date,
        body_text: String::new(),
        attachments: vec![],
    };

    if let Some(text_body) = description
        .attachments
        .iter()
        .find(|attach| attach.is_email_text)
    {
        let text_bytes =
            &email_message_raw.text().unwrap()[text_body.bytes_start..text_body.bytes_end];
        response.body_text = match text_body.encoding {
            EncodingType::SevenBit => String::from_utf8(text_bytes.to_vec()).unwrap_or_default(),
            EncodingType::Base64 => String::from_utf8(
                data_encoding::BASE64_MIME
                    .decode(text_bytes)
                    .unwrap_or_default(),
            )
            .unwrap_or_default(),
            EncodingType::Other => todo!(),
        }
    }

    for attach_info in description.attachments {
        if attach_info.is_file {
            let attach = EmailDetailAttachmentOutDTO {
                file_name: attach_info.file_name,
                size_octets: attach_info.size_octets,
                is_file: attach_info.is_file,
            };
            response.attachments.push(attach);
        }
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
        &credentials.get_imap_string(),
    )
    .await
    .unwrap();

    println!("Request: {:?}", request);
    imap_session
        .select(&encode_utf7_imap(request.mailbox_name.clone()))
        .unwrap();
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
    request: web::Query<EmailListInDTO>,
) -> impl Responder {
    let credentials = check_is_valid_session(&session).unwrap();
    let mut imap_session = create_imap_session(
        &credentials.email,
        &credentials.password,
        &credentials.get_imap_string(),
    )
    .await
    .unwrap();

    let mailbox_info = imap_session
        .select(&encode_utf7_imap(request.mailbox_name.clone()))
        .unwrap();
    println!("Mailbox info: {:?}", mailbox_info);

    if mailbox_info.exists == 0
    {
        return EmailListOutDTO {
            total_emails_count: mailbox_info.exists,
            requested_page_number: request.requested_page_number,
            page_size: request.page_size,
            emails: vec!(),
        };
    }

    let start_number = mailbox_info.exists - min(mailbox_info.exists, request.requested_page_number * request.page_size);
    let end_number = start_number - min(start_number, request.page_size + 1);

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

        let sender_host_bytes = message.envelope().unwrap().from.as_ref().unwrap_or(&vec![])[0]
            .host
            .unwrap_or_default();

        let subject_bytes = message.envelope().unwrap().subject.unwrap_or_default();

        let sender = String::from_utf8(sender_bytes.to_vec()).unwrap_or_default()
            + "@"
            + &String::from_utf8(sender_host_bytes.to_vec()).unwrap_or_default();
        let (_, subject) = encoded_word(subject_bytes).unwrap_or((
            subject_bytes,
            String::from_utf8(subject_bytes.to_vec()).unwrap_or("Cant parse subject".to_string()),
        ));

        let was_read = message.flags().contains(&Flag::Seen);
        let send_date = message.internal_date().unwrap_or_default().naive_utc();

        let message_out = EmailInspectOutDTO {
            from_address: sender,
            subject,
            was_read,
            send_date,
            sequence_number: message.message
        };

        messages_out.push(message_out);
    }

    let response = EmailListOutDTO {
        total_emails_count: mailbox_info.exists,
        requested_page_number: request.requested_page_number,
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
    separator: String,
    match_index: usize,
) {
    match structure {
        imap_proto::BodyStructure::Basic {
            common,
            other,
            extension: _,
        } => {
            println!("Basic body structure");
            println!(
                "BodyContentCommon: {:?}, BodyContentSinglePart: {:?}",
                common, other
            );

            let regex_string = format!(
                r"{}(\r\n|\n)[\S\s]*?(\r\n|\n)(\r\n|\n)([\S\s]*?)(\r\n|\n)--",
                separator
            );
            let regex = Regex::new(&regex_string).unwrap();
            let body_matches = regex.captures_iter(message.text().unwrap());

            let mut attachment_description = EmailPartDescription {
                file_name: "Unparsed attachment".to_string(),
                size_octets: other.octets,
                is_file: false,
                bytes_start: 0,
                bytes_end: 0,
                is_email_text: false,
                encoding: decide_encoding(other),
            };

            modify_part_description(
                body_matches,
                match_index,
                &mut attachment_description,
                common,
            );

            description.attachments.push(attachment_description);
        }
        imap_proto::BodyStructure::Text {
            common,
            other,
            lines,
            extension: _,
        } => {
            println!("Text body structure");
            println!(
                "BodyContentCommon: {:?}, BodyContentSinglePart: {:?}, Lines {}",
                common, other, lines
            );

            let regex_string = format!(
                r"{}(\r\n|\n)[\S\s]*?(\r\n|\n)(\r\n|\n)([\S\s]*?)(\r\n|\n)--",
                separator
            );
            let regex = Regex::new(&regex_string).unwrap();
            let body_matches = regex.captures_iter(message.text().unwrap());

            let mut attachment_description = EmailPartDescription {
                file_name: "Unparsed attachment".to_string(),
                size_octets: other.octets,
                is_file: false,
                bytes_start: 0,
                bytes_end: 0,
                is_email_text: false,
                encoding: decide_encoding(other),
            };

            modify_part_description(
                body_matches,
                match_index,
                &mut attachment_description,
                common,
            );

            description.attachments.push(attachment_description)
        }
        imap_proto::BodyStructure::Message {
            common: _,
            other: _,
            envelope: _,
            body: _,
            lines: _,
            extension: _,
        } => {
            println!("Message body structure ignored");
        }
        imap_proto::BodyStructure::Multipart {
            common,
            bodies,
            extension: _,
        } => {
            println!("Multipart body structure");
            println!("BodyContentCommon: {:?}", common);
            for (part_index, body) in bodies.iter().enumerate() {
                let boundary = if let Some(params) = &common.ty.params {
                    params.iter().find(|(desc, _)| *desc == "BOUNDARY")
                } else {
                    None
                };

                if let Some(boundary_value) = boundary {
                    parse_body_structure(
                        body,
                        message,
                        description,
                        boundary_value.1.to_string(),
                        part_index,
                    );
                }
            }
        }
    }
}

fn decide_encoding(other: &imap_proto::BodyContentSinglePart) -> EncodingType {
    match other.transfer_encoding {
        imap_proto::ContentEncoding::SevenBit => EncodingType::SevenBit,
        imap_proto::ContentEncoding::Base64 => EncodingType::Base64,
        _ => EncodingType::Other,
    }
}

fn modify_part_description(
    mut body_matches: regex::bytes::CaptureMatches,
    match_index: usize,
    attachment_description: &mut EmailPartDescription,
    common: &imap_proto::BodyContentCommon,
) {
    if let Some(capture_match) = body_matches.nth(match_index) {
        if let Some(result_match) = capture_match.get(4) {
            attachment_description.bytes_start = result_match.start();
            attachment_description.bytes_end = result_match.end();
        }
    }

    if let Some(disposition) = &common.disposition {
        if let Some(parameters) = &disposition.params {
            if let Some(file_name) = parameters.iter().find(|(desc, _)| desc == &"FILENAME") {
                attachment_description.is_file = true;
                attachment_description.file_name = file_name.1.to_string();
            }
        }
    } else {
        attachment_description.is_email_text =
            common.ty.ty == "TEXT" && common.ty.subtype == "PLAIN";
        attachment_description.file_name = "Email text".to_string();
    }
}

async fn download_attachment_from_email(
    session: Session,
    request: web::Query<EmailAttachmentInDTO>,
) -> impl Responder {
    let credentials = check_is_valid_session(&session).unwrap();
    let mut imap_session = create_imap_session(
        &credentials.email,
        &credentials.password,
        &credentials.get_imap_string(),
    )
    .await
    .unwrap();

    imap_session
        .select(&encode_utf7_imap(request.mailbox_name.clone()))
        .unwrap();
    let email_message_raw = &imap_session
        .fetch(
            format!("{}", request.sequence_number),
            "(FLAGS BODYSTRUCTURE BODY[TEXT] ENVELOPE INTERNALDATE)",
        )
        .unwrap()[0];
    let structure = email_message_raw.bodystructure().unwrap();

    let mut description = EmailAnalysis {
        plain_text_octets: 0,
        attachments: vec![],
    };
    parse_body_structure(
        structure,
        email_message_raw,
        &mut description,
        String::new(),
        0,
    );

    let found_attachment = description
        .attachments
        .iter()
        .find(|attachment| attachment.file_name == request.attachment_name);

    match found_attachment {
        Some(description) => {
            let result_bytes =
                &email_message_raw.text().unwrap()[description.bytes_start..description.bytes_end];

            let decoded_bytes = match description.encoding {
                EncodingType::SevenBit => result_bytes.to_vec(),
                EncodingType::Base64 => match data_encoding::BASE64_MIME.decode(result_bytes) {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        println!("Decoding error: {}", error);
                        vec![]
                    }
                },
                EncodingType::Other => result_bytes.to_vec(),
            };

            let content_disposition = ContentDisposition {
                disposition: DispositionType::Attachment,
                parameters: vec![DispositionParam::Filename(description.file_name.clone())],
            };

            HttpResponse::Ok()
                .insert_header(ContentEncoding::Identity)
                .insert_header(content_disposition)
                .content_type("application/octet-stream")
                .body(decoded_bytes)
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

async fn get_mailboxes(session: Session) -> impl Responder {
    let credentials = check_is_valid_session(&session).unwrap();
    let mut imap_session = create_imap_session(
        &credentials.email,
        &credentials.password,
        &credentials.get_imap_string(),
    )
    .await
    .unwrap();

    let mailboxes = imap_session.list(None, Some("*")).unwrap();
    let mut mailbox_names: Vec<String> = vec![];

    for mailbox in mailboxes.iter() {
        println!("Mailbox: {:?}", mailbox);
        if !mailbox.attributes().contains(&NameAttribute::NoSelect)
        {
            mailbox_names.push(decode_utf7_imap(mailbox.name().to_string()));
        }
    }

    MailboxListOutDTO { mailbox_names }
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
