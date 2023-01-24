use std::{
    fs::{read, File},
    io::{Read, Write},
};

use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{
    web::{self, Bytes},
    Error, HttpResponse, Responder,
};
use futures_util::{StreamExt, TryStreamExt};
use lettre::{
    message::{
        header::{self, ContentType},
        Attachment, MultiPart, SinglePart, SinglePartBuilder,
    },
    AsyncTransport, Message,
};

use crate::utils::{
    utils_session::check_is_valid_session, utils_transports::create_smtp_transport,
};

use super::models::EmailInDTO;

async fn send_email(mut payload: Multipart, session: Session) -> Result<HttpResponse, Error> {
    // Check the session
    let sess_values = check_is_valid_session(&session)?;

    // Create initial email struct
    let mut email_struct = EmailInDTO {
        to_address: String::new(),
        subject: String::new(),
        body: String::new(),
    };

    let mut file_complete_path = Vec::new();

    // Iterate over multipart stream
    while let Some(mut field) = payload.try_next().await? {
        // Found a file
        if let Some(file_name) = field.content_disposition().get_filename() {
            let filepath = format!("./tmp/{}", file_name);
            file_complete_path.push((filepath.clone(), file_name.to_string()));

            // File::create is blocking operation, use threadpool
            let mut file_created = web::block(|| std::fs::File::create(filepath)).await??;

            // Field in turn is stream of *Bytes* object
            while let Some(Ok(chunk)) = field.next().await {
                // Filesystem operations are blocking, we have to use threadpool
                file_created =
                    web::block(move || file_created.write_all(&chunk).map(|_| file_created))
                        .await??;
            }
        } else {
            let bytes_result = if let Some(value) = field.next().await {
                if let Ok(only_bytes) = value {
                    only_bytes
                } else {
                    Bytes::new()
                }
            } else {
                Bytes::new()
            };

            match field.content_disposition().get_name().unwrap() {
                "to_address" => {
                    println!("to_address");
                    email_struct.to_address = String::from_utf8(bytes_result.to_vec()).unwrap()
                }
                "subject" => {
                    println!("subject");
                    email_struct.subject = String::from_utf8(bytes_result.to_vec()).unwrap()
                }
                "body" => {
                    println!("body");
                    email_struct.body = String::from_utf8(bytes_result.to_vec()).unwrap()
                }
                result => {
                    print!("Other name {}", result);
                }
            }
        }
    }

    let mut body_total = MultiPart::mixed().singlepart(
        SinglePart::builder()
            .content_type(ContentType::TEXT_PLAIN)
            .body(email_struct.body.to_string()),
    );

    if !file_complete_path.is_empty() {
        for path in file_complete_path.into_iter() {
            let file_content = web::block(move || read(path.0)).await??;
            let content_type_guess = mime_guess::from_path(&path.1);

            body_total = body_total.singlepart(
                Attachment::new(path.1.clone()).body(
                    file_content,
                    content_type_guess
                        .first_or_octet_stream()
                        .to_string()
                        .parse()
                        .unwrap(),
                ),
            );
        }
    }

    let email = Message::builder()
        .to(email_struct.to_address.parse().unwrap())
        .from(sess_values.email.parse().unwrap())
        .subject(email_struct.subject.to_string())
        .multipart(body_total)
        .unwrap();

    if let Ok(session) = create_smtp_transport(
        &sess_values.email,
        &sess_values.password,
        &sess_values.get_smtp_string(),
    )
    .await
    {
        session.send(email).await.unwrap();
    }

    Ok(HttpResponse::Ok().body("Ok"))
}

pub fn email_smtp_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/email/send").route(web::post().to(send_email)));
}
