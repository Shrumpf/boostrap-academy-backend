use academy_models::{
    contact::{
        ContactMessage, ContactMessageAuthor, ContactMessageAuthorName, ContactMessageContent,
        ContactMessageSubject,
    },
    email_address::EmailAddress,
};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ApiContactMessage {
    /// Full name of the user
    pub name: ContactMessageAuthorName,
    /// Email address of the user
    pub email: EmailAddress,
    /// Subject of the message
    pub subject: ContactMessageSubject,
    /// Content of the message
    pub message: ContactMessageContent,
}

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
