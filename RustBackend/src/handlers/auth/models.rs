use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SignInMessage {
    pub email: String,
    pub password: String,
    pub domain: String,
}

impl SignInMessage {
    pub fn get_imap_string(&self) -> String {
        format!("imap.{}", &self.domain)
    }
}
