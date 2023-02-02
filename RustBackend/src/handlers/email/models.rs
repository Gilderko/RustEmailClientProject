use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct EmailInDTO {
    pub to_address: String,
    pub subject: String,
    pub body: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailListInDTO {
    pub requested_page_number: u32,
    pub page_size: u32,
    pub mailbox_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailListOutDTO {
    pub total_emails_count: u32,
    pub requested_page_number: u32,
    pub page_size: u32,
    pub emails: Vec<EmailInspectOutDTO>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailInspectOutDTO {
    pub from_address: String,
    pub subject: String,
    pub was_read: bool,
    pub send_date: NaiveDateTime,
    pub sequence_number: u32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailDeleteInDTO {
    pub mailbox_name: String,
    pub sequence_set_top: u32,
    pub sequence_set_bottom: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailDetailInDTO {
    pub mailbox_name: String,
    pub sequence_number: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailDetailOutDTO {
    pub from_address: String,
    pub subject: String,
    pub send_date: NaiveDateTime,
    pub body_text: String,
    pub attachments: Vec<EmailDetailAttachmentOutDTO>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailDetailAttachmentOutDTO {
    pub file_name: String,
    pub size_octets: u32,
    pub is_file: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MailboxListOutDTO {
    pub mailbox_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailAttachmentInDTO {
    pub mailbox_name: String,
    pub sequence_number: u32,
    pub attachment_name: String,
}
