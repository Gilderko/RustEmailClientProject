use std::io::{Error, ErrorKind};

use actix_session::Session;
use crate::{
    constants::{AUTH_EMAIL_STRING, AUTH_PASSWORD_STRING},
    handlers::auth::models::SignInMessage,
};

pub fn check_is_valid_session(session: &Session) -> Result<SignInMessage, Error> {
    let email = session.get::<String>(AUTH_EMAIL_STRING);
    let password = session.get::<String>(AUTH_PASSWORD_STRING);

    if let (Ok(Some(email_value)), Ok(Some(password_value))) = (email, password) {
        Ok(SignInMessage {
            email: email_value,
            password: password_value,
        })
    } else {
        Err(Error::new(ErrorKind::Other, "Unauthenticated"))
    }
}

