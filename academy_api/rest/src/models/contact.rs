use academy_models::contact::{
    ContactMessage, ContactMessageAuthor, ContactMessageAuthorName, ContactMessageContent,
    ContactMessageSubject,
};
use email_address::EmailAddress;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ApiContactMessage {
    pub name: ContactMessageAuthorName,
    pub email: EmailAddress,
    pub subject: ContactMessageSubject,
    pub message: ContactMessageContent,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiContactMessageAuthor {}

impl From<ApiContactMessage> for ContactMessage {
    fn from(value: ApiContactMessage) -> Self {
        Self {
            author: ContactMessageAuthor {
                name: value.name,
                email: value.email,
            },
            subject: value.subject,
            content: value.message,
        }
    }
}
