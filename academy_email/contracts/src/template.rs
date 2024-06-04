use std::future::Future;

use academy_templates_contracts::{
    ResetPasswordTemplate, SubscribeNewsletterTemplate, VerifyEmailTemplate,
};
use email_address::EmailAddress;

#[cfg_attr(feature = "mock", mockall::automock)]
pub trait TemplateEmailService: Send + Sync + 'static {
    fn send_reset_password_email(
        &self,
        recipient: EmailAddress,
        data: &ResetPasswordTemplate,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send;

    fn send_subscribe_newsletter_email(
        &self,
        recipient: EmailAddress,
        data: &SubscribeNewsletterTemplate,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send;

    fn send_verification_email(
        &self,
        recipient: EmailAddress,
        data: &VerifyEmailTemplate,
    ) -> impl Future<Output = anyhow::Result<bool>> + Send;
}

#[cfg(feature = "mock")]
impl MockTemplateEmailService {
    pub fn with_send_reset_password_email(
        mut self,
        recipient: EmailAddress,
        data: ResetPasswordTemplate,
        result: bool,
    ) -> Self {
        self.expect_send_reset_password_email()
            .once()
            .with(
                mockall::predicate::eq(recipient),
                mockall::predicate::eq(data),
            )
            .return_once(move |_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }

    pub fn with_send_subscribe_newsletter_email(
        mut self,
        recipient: EmailAddress,
        data: SubscribeNewsletterTemplate,
        result: bool,
    ) -> Self {
        self.expect_send_subscribe_newsletter_email()
            .once()
            .with(
                mockall::predicate::eq(recipient),
                mockall::predicate::eq(data),
            )
            .return_once(move |_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }

    pub fn with_send_verification_email(
        mut self,
        recipient: EmailAddress,
        data: VerifyEmailTemplate,
        result: bool,
    ) -> Self {
        self.expect_send_verification_email()
            .once()
            .with(
                mockall::predicate::eq(recipient),
                mockall::predicate::eq(data),
            )
            .return_once(move |_, _| Box::pin(std::future::ready(Ok(result))));
        self
    }
}
