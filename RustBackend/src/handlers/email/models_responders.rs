use actix_web::{body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder};

use super::models::{EmailDetailOutDTO, EmailListOutDTO, MailboxListOutDTO};

impl Responder for EmailDetailOutDTO {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = match serde_json::to_string(&self) {
            Ok(val) => val,
            Err(err) => {
                return HttpResponse::InternalServerError()
                    .body(format!("Error serializing response: {}", err))
            }
        };

        // Create response and set content type
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

impl Responder for EmailListOutDTO {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = match serde_json::to_string(&self) {
            Ok(val) => val,
            Err(err) => {
                return HttpResponse::InternalServerError()
                    .body(format!("Error serializing response: {}", err))
            }
        };

        // Create response and set content type
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

impl Responder for MailboxListOutDTO {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = match serde_json::to_string(&self) {
            Ok(val) => val,
            Err(err) => {
                return HttpResponse::InternalServerError()
                    .body(format!("Error serializing response: {}", err))
            }
        };

        // Create response and set content type
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}
