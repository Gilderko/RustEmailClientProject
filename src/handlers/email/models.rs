use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct EmailInDTO {
    pub to_address: String,
    pub subject: String,
    pub body: String,
}

#[derive(Deserialize, Debug)]
pub struct EmailListInDTO {
    pub requested_page_number: u32,
    pub page_size: u32,
}

#[derive(Serialize, Debug)]
pub struct EmailListOutDTO {
    pub total_emails_count: u32,
    pub requested_page_number: u32,
    pub page_size: u32,
    pub emails: Vec<EmailInspectOutDTO>,
}

#[derive(Serialize, Debug)]
pub struct EmailInspectOutDTO {
    pub from_address: String,
    pub subject: String,
    pub was_read: bool,
    pub attachment_count: u32,
    pub send_date: String,
}

#[derive(Deserialize, Debug)]
pub struct EmailDeleteInDTO{
    pub mailbox_name: String,
    pub sequence_set_top: u32,
    pub sequence_set_bottom: u32
}