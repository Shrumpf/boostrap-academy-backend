use std::future::Future;

use academy_models::contact::ContactMessage;
use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait ContactService: Send + Sync + 'static {
    fn send_message(
        &self,
        message: ContactMessage,
    ) -> impl Future<Output = Result<(), ContactSendMessageError>> + Send;
}

#[derive(Debug, Error)]
pub enum ContactSendMessageError {
    #[error("Failed to send message.")]
    Send,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
