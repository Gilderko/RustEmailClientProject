use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SignInMessage {
    pub email: String,
    pub password: String,
}
