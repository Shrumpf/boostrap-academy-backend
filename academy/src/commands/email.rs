use academy_config::Config;
use academy_email_contracts::{ContentType, Email, EmailService};
use academy_email_impl::EmailServiceImpl;
use academy_models::email_address::EmailAddressWithName;
use anyhow::ensure;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum EmailCommand {
    /// Test email deliverability
    Test { recipient: EmailAddressWithName },
}

impl EmailCommand {
    pub async fn invoke(self, config: Config) -> anyhow::Result<()> {
        match self {
            EmailCommand::Test { recipient } => test(config, recipient).await,
        }
    }
}

async fn test(config: Config, recipient: EmailAddressWithName) -> anyhow::Result<()> {
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
