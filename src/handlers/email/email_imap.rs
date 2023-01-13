use actix_session::Session;
use actix_web::{Responder, HttpResponse};

async fn get_email_in_detail_from_inbox() -> impl Responder {
    HttpResponse::Ok()
}

async fn delete_email_from_inbox() -> impl Responder {
    HttpResponse::Ok()
}

async fn list_emails_from_inbox() -> impl Responder {
    HttpResponse::Ok()
}

async fn download_attachment_from_email(session: Session) -> impl Responder {
    HttpResponse::Ok()
}