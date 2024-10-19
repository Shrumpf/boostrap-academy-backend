use std::{ops::Deref, sync::LazyLock};

use academy_utils::academy_version;

pub static USER_AGENT: LazyLock<String> = LazyLock::new(|| {
    let homepage = env!("CARGO_PKG_HOMEPAGE");
    let repository = env!("CARGO_PKG_REPOSITORY");
    let version = academy_version();

    format!("Bootstrap Academy Backend ({homepage}, {repository}, Version {version})")
});

const _: () = {
    assert!(!env!("CARGO_PKG_HOMEPAGE").is_empty());
    assert!(!env!("CARGO_PKG_REPOSITORY").is_empty());
};

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
                .user_agent(&*USER_AGENT)
                .build()
                .unwrap(),
        )
    }
}
