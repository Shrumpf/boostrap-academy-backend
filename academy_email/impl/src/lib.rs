use academy_email_contracts::{ContentType, Email, EmailService};
use academy_utils::Apply;
use anyhow::anyhow;
use email_address::EmailAddress;
use lettre::{
    message::{header, MessageBuilder},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

pub mod template;

#[derive(Debug, Clone)]
pub struct EmailServiceImpl {
    from: EmailAddress,
    transport: AsyncSmtpTransport<Tokio1Executor>,
}

impl EmailServiceImpl {
    pub async fn new(url: &str, from: EmailAddress) -> anyhow::Result<Self> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::from_url(url)?.build();

        Ok(Self { from, transport })
    }

    #[cfg(feature = "dummy")]
    pub async fn dummy() -> Self {
        Self::new("smtp://dummy", "dummy@example.com".parse().unwrap())
            .await
            .unwrap()
    }
}

impl EmailService for EmailServiceImpl {
    async fn send(&self, email: Email) -> anyhow::Result<bool> {
        let message = Message::builder()
            .from(self.from.as_str().parse()?)
            .to(email.recipient.as_str().parse()?)
            .apply_map(
                email.reply_to.map(|x| x.as_str().parse()).transpose()?,
                MessageBuilder::reply_to,
            )
            .subject(email.subject)
            .header(match email.content_type {
                ContentType::Text => header::ContentType::TEXT_PLAIN,
                ContentType::Html => header::ContentType::TEXT_HTML,
            })
            .body(email.body)?;

        self.transport
            .send(message)
            .await
            .map(|response| response.is_positive())
            .map_err(Into::into)
    }

    async fn ping(&self) -> anyhow::Result<()> {
        self.transport
            .test_connection()
            .await?
            .then_some(())
            .ok_or_else(|| anyhow!("Failed to ping smtp server"))
    }
}
