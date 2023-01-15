use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailAnalysis{
    pub plain_text_octets: u32,
    pub attachments: Vec<AttachmentDescription>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AttachmentDescription{
    pub file_name: String,
    pub size_octets: u32,
    pub is_file: bool
}