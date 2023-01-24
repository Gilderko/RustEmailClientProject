use std::io::{Error, ErrorKind};

use crate::{
    constants::{AUTH_EMAIL_STRING, AUTH_PASSWORD_STRING, AUTH_DOMAIN_STRING},
    handlers::auth::models::SignInMessage,
};
use actix_session::Session;

pub fn check_is_valid_session(session: &Session) -> Result<SignInMessage, Error> {
    let email = session.get::<String>(AUTH_EMAIL_STRING);
    let password = session.get::<String>(AUTH_PASSWORD_STRING);
    let domain = session.get::<String>(AUTH_DOMAIN_STRING);

    if let (Ok(Some(email_value)), Ok(Some(password_value)), Ok(Some(domain_value))) =
        (email, password, domain)
    {
        Ok(SignInMessage {
            email: email_value,
            password: password_value,
            domain: domain_value,
        })
    } else {
        Err(Error::new(ErrorKind::Other, "Unauthenticated"))
    }
}
