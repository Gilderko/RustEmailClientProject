use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct EmailInDTO{
    pub to_address: String,
    pub subject: String,
    pub body: String,
}