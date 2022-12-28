use std::io::{Error, ErrorKind};

use actix_session::Session;

use crate::{
    constants::{auth_email_string, auth_password_string},
    handlers::auth::models::SignInMessage,
};

pub fn check_is_valid_session(session: &Session) -> Result<SignInMessage, Error> {
    let email = session.get::<String>(auth_email_string);
    let password = session.get::<String>(auth_password_string);

    if let (Ok(Some(email_value)), Ok(Some(password_value))) = (email, password) {
        Ok(SignInMessage {
            email: email_value,
            password: password_value,
        })
    } else {
        Err(Error::new(ErrorKind::Other, "Unauthenticated"))
    }
}
