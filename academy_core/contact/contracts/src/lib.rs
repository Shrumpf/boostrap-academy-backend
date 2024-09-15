use std::future::Future;

use academy_models::{contact::ContactMessage, RecaptchaResponse};
use thiserror::Error;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait ContactFeatureService: Send + Sync + 'static {
    fn send_message(
        &self,
        message: ContactMessage,
        recaptcha_response: Option<RecaptchaResponse>,
    ) -> impl Future<Output = Result<(), ContactSendMessageError>> + Send;
}

#[derive(Debug, Error)]
pub enum ContactSendMessageError {
    #[error("Invalid recaptcha response")]
    Recaptcha,
    #[error("Failed to send message.")]
    Send,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
