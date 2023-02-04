use std::{
    fs::{read, remove_file},
    io::{Error, ErrorKind, Write},
};

use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{
    web::{self, Bytes},
    HttpResponse,
};
use futures_util::{StreamExt, TryStreamExt};
use lettre::{
    message::{header::ContentType, Attachment, MultiPart, SinglePart},
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
    while let Some(mut field) = payload.try_next().await.unwrap() {
        match field.content_disposition().get_filename() {
            // Found a file
            Some(file_name) => {
                println!("Got file {file_name}");
                let filepath = format!("./tmp/{}", file_name);
                file_complete_path.push((filepath.clone(), file_name.to_string()));

                let mut file_created;

                // File::create is blocking operation, use threadpool
                match web::block(|| std::fs::File::create(filepath)).await {
                    Ok(res) => match res {
                        Ok(file) => file_created = file,
                        Err(err) => {
                            return Err(Error::new(
                                ErrorKind::Other,
                                format!("Creating file Error {:?}", err),
                            ))
                        }
                    },
                    Err(err) => {
                        return Err(Error::new(
                            ErrorKind::Other,
                            format!("Creating file Blocking Error {:?}", err),
                        ))
                    }
                };
                // Field in turn is stream of *Bytes* object
                while let Some(Ok(chunk)) = field.next().await {
                    // Filesystem operations are blocking, we have to use threadpool
                    match web::block(move || file_created.write_all(&chunk).map(|_| file_created))
                        .await
                    {
                        Ok(res) => match res {
                            Ok(file) => file_created = file,
                            Err(err) => {
                                return Err(Error::new(
                                    ErrorKind::Other,
                                    format!("Creating file Error {:?}", err),
                                ))
                            }
                        },
                        Err(err) => {
                            return Err(Error::new(
                                ErrorKind::Other,
                                format!("Creating file Blocking Error {:?}", err),
                            ))
                        }
                    }
                }
            }
            _ => {
                let field_value = match field.next().await {
                    Some(Ok(only_bytes)) => only_bytes,
                    Some(Err(err)) => {
                        eprintln!("Error: {:?}", err);
                        Bytes::new()
                    }
                    None => Bytes::new(),
                };

                let field_content_disposion_name = field.content_disposition().get_name();

                if field_content_disposion_name.is_none() {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "couldnt parse name from field content disposion",
                    ));
                }

                match field_content_disposion_name.unwrap() {
                    "to_address" => {
                        println!("to_address");
                        email_struct.to_address = String::from_utf8(field_value.to_vec())
                            .expect("coudlnt parse to_address from field");
                    }
                    "subject" => {
                        println!("subject");
                        email_struct.subject = String::from_utf8(field_value.to_vec())
                            .expect("coudlnt parse subject from field");
                    }
                    "body" => {
                        println!("body");
                        email_struct.body = String::from_utf8(field_value.to_vec())
                            .expect("coudlnt parse body from field");
                    }
                    other => {
                        print!("Other name {}", other);
                    }
                }
            }
        };
    }

    let mut body_total = MultiPart::mixed().singlepart(
        SinglePart::builder()
            .content_type(ContentType::TEXT_PLAIN)
            .body(email_struct.body.to_string()),
    );

    if !file_complete_path.is_empty() {
        for (path, name) in file_complete_path.into_iter() {
            let file_content;
            let path_copy = path.clone();
            match web::block(move || read(path)).await {
                Ok(res) => match res {
                    Ok(content) => {
                        file_content = content;
                        remove_file(path_copy);
                    }
                    Err(err) => {
                        return Err(Error::new(
                            ErrorKind::Other,
                            format!("Error reading file content {:?}", err),
                        ))
                    }
                },
                Err(err) => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("Error reading file Blocking Error {:?}", err),
                    ))
                }
            };

            let content_type;

            match mime_guess::from_path(&name)
                .first_or_octet_stream()
                .to_string()
                .parse()
            {
                Ok(con_type) => content_type = con_type,
                Err(err) => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("Error parsing content_type (CotnentTypeErr) {:?}", err),
                    ))
                }
            }
            
            body_total =
                body_total.singlepart(Attachment::new(name).body(file_content, content_type));
        }
    }

    match Message::builder()
        .to(email_struct.to_address.parse().unwrap())
        .from(sess_values.email.parse().unwrap())
        .subject(email_struct.subject.to_string())
        .multipart(body_total)
    {
        Ok(message) => {
            let session = create_smtp_transport(
                &sess_values.email,
                &sess_values.password,
                &sess_values.get_smtp_string(),
            )
            .await
            .unwrap();

            let send_result = session.send(message).await;

            if send_result.is_err() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Couldnt build message {:?}", send_result.err()),
                ));
            }
        }
        Err(err) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Couldnt build message {:?}", err),
            ))
        }
    };

    Ok(HttpResponse::Ok().body("Ok"))
}

pub fn email_smtp_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/email/send").route(web::post().to(send_email)));
}
