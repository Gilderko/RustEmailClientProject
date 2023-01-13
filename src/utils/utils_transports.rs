use std::net::TcpStream;

use imap::{Error, Session};
use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};
use native_tls::TlsStream;

pub async fn create_smtp_transport(
    username: &String,
    password: &String,
    domain: &String,
) -> Result<AsyncSmtpTransport<Tokio1Executor>, lettre::transport::smtp::Error> {
    let creds = Credentials::new(username.clone(), password.clone());

    let smtp_session =
        AsyncSmtpTransport::<Tokio1Executor>::relay(&domain)
            .unwrap()
            .credentials(creds)
            .build();
    
    let smtp_test = smtp_session.test_connection().await;

    match smtp_test {
        Ok(_) => Ok(smtp_session),
        Err(e) => Err(e),
    }
}

pub async fn create_imap_session(
    username: &String,
    password: &String,
    domain: &String,
) -> Result<Session<TlsStream<TcpStream>>, Error> {
    let tls = native_tls::TlsConnector::builder().build().unwrap();
    let client = imap::connect((domain.clone(), 993), domain.clone(), &tls).unwrap();
    let imap_session = client.login(&username, &password).map_err(|e| e.0);

    match imap_session {
        Ok(session) => Ok(session),
        Err(error) => Err(error),
    }
}
