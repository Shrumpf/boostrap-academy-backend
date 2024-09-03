use std::ops::Deref;

pub const USER_AGENT: &str = concat!(
    "Bootstrap Academy Backend (",
    env!("CARGO_PKG_HOMEPAGE"),
    ", ",
    env!("CARGO_PKG_REPOSITORY"),
    ", Version ",
    env!("CARGO_PKG_VERSION"),
    ")"
);

#[derive(Debug, Clone)]
pub struct HttpClient(reqwest::Client);

impl Deref for HttpClient {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self(
            reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
        )
    }
}
