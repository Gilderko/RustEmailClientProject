pub struct EmailAnalysis {
    pub plain_text_octets: u32,
    pub attachments: Vec<EmailPartDescription>,
}

pub struct EmailPartDescription {
    pub file_name: String,
    pub size_octets: u32,
    pub bytes_start: usize,
    pub bytes_end: usize,
    pub is_file: bool,
    pub is_email_text: bool,
    pub encoding: EncodingType,
}

pub enum EncodingType {
    SevenBit,
    Base64,
    Other,
}
