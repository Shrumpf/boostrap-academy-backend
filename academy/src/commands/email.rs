use academy_config::Config;
use academy_email_contracts::{ContentType, Email, EmailService};
use academy_email_impl::EmailServiceImpl;
use anyhow::ensure;
use clap::Subcommand;
use email_address::EmailAddress;

#[derive(Debug, Subcommand)]
pub enum EmailCommand {
    /// Test email deliverability
    Test { recipient: EmailAddress },
}

impl EmailCommand {
    pub async fn invoke(self, config: Config) -> anyhow::Result<()> {
        match self {
            EmailCommand::Test { recipient } => test(config, recipient).await,
        }
    }
}

async fn test(config: Config, recipient: EmailAddress) -> anyhow::Result<()> {
    let email_service = EmailServiceImpl::new(&config.email.smtp_url, config.email.from).await?;

    let ok = email_service
        .send(Email {
            recipient,
            subject: "Email Deliverability Test".into(),
            body: "Email deliverability seems to be working!".into(),
            content_type: ContentType::Text,
            reply_to: None,
        })
        .await?;

    ensure!(ok, "Failed to send email");

    Ok(())
}
