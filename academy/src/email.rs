use academy_config::EmailConfig;
use academy_email_impl::EmailServiceImpl;
use anyhow::Context;

/// Connect to the SMTP server
pub async fn connect(config: &EmailConfig) -> anyhow::Result<EmailServiceImpl> {
    EmailServiceImpl::new(&config.smtp_url, config.from.clone())
        .await
        .context("Failed to connect to SMTP server")
}
