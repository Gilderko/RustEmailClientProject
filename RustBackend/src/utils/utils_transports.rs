use std::{io::Error, io::ErrorKind, net::TcpStream};

use imap::Session;
use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};
use native_tls::TlsStream;

pub async fn create_smtp_transport(
    username: &str,
    password: &str,
    domain: &str,
) -> Result<AsyncSmtpTransport<Tokio1Executor>, Error> {
    let creds = Credentials::new(username.to_owned(), password.to_owned());

    let smtp_session = match AsyncSmtpTransport::<Tokio1Executor>::relay(domain) {
        Ok(session) => session,
        Err(err) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("SMTP transport relay failed: {}", err),
            ))
        }
    }
    .credentials(creds)
    .build();

    match smtp_session.test_connection().await {
        Ok(_) => Ok(smtp_session),
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            format!("SMTP test connection failed: {}", e),
        )),
    }
}

pub async fn create_imap_session(
    username: &String,
    password: &String,
    domain: &str,
) -> Result<Session<TlsStream<TcpStream>>, Error> {
    let tls = match native_tls::TlsConnector::builder().build() {
        Ok(val) => val,
        Err(err) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("TlsConnector build failed: {:?}", err),
            ))
        }
    };

    match imap::connect((domain.to_owned(), 993), domain, &tls) {
        Ok(client) => match client.login(username, password) {
            Ok(session) => Ok(session),
            Err(err) => Err(Error::new(
                ErrorKind::Other,
                format!("IMAP login failed: {:?}", err),
            )),
        },
        Err(err) => Err(Error::new(
            ErrorKind::Other,
            format!("IMAP connect failed: {:?}", err),
        )),
    }
}
