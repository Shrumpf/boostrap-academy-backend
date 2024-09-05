use std::future::Future;

use academy_models::email_address::EmailAddressWithName;

pub mod template;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait EmailService: Send + Sync + 'static {
    fn send(&self, email: Email) -> impl Future<Output = anyhow::Result<bool>> + Send;

    fn ping(&self) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email {
    pub recipient: EmailAddressWithName,
    pub subject: String,
    pub body: String,
    pub content_type: ContentType,
    pub reply_to: Option<EmailAddressWithName>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Text,
    Html,
}

#[cfg(feature = "mock")]
impl MockEmailService {
    pub fn with_send(mut self, email: Email, result: bool) -> Self {
        self.expect_send()
            .once()
            .with(mockall::predicate::eq(email))
            .return_once(move |_| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
