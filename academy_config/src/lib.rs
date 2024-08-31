use std::{net::IpAddr, path::Path};

use academy_models::mfa::TotpSecretLength;
use anyhow::Context;
use config::{File, FileFormat};
use email_address::EmailAddress;
use serde::Deserialize;
use url::Url;

pub const DEFAULT_CONFIG_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../config.toml");

pub fn load(paths: &[impl AsRef<Path>]) -> anyhow::Result<Config> {
    paths
        .iter()
        .try_fold(config::Config::builder(), |builder, path| {
            let path = path.as_ref();
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read config file at {}", path.display()))?;
            let source = File::from_str(&content, FileFormat::Toml);
            anyhow::Ok(builder.add_source(source))
        })?
        .build()?
        .try_deserialize()
        .context("Failed to load config")
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub http: HttpConfig,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub email: EmailConfig,
    pub jwt: JwtConfig,
    pub health: HealthConfig,
    pub user: UserConfig,
    pub session: SessionConfig,
    pub totp: TotpConfig,
    pub contact: ContactConfig,
    pub recaptcha: Option<RecaptchaConfig>,
}

#[derive(Debug, Deserialize)]
pub struct HttpConfig {
    pub host: IpAddr,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
}

#[derive(Debug, Deserialize)]
pub struct CacheConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
}

#[derive(Debug, Deserialize)]
pub struct EmailConfig {
    pub smtp_url: String,
    pub from: EmailAddress,
}

#[derive(Debug, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
}

#[derive(Debug, Deserialize)]
pub struct HealthConfig {
    pub cache_ttl: Duration,
}

#[derive(Debug, Deserialize)]
pub struct UserConfig {
    pub name_change_rate_limit: Duration,
    pub verification_code_ttl: Duration,
    pub verification_redirect_url: String,
    pub password_reset_code_ttl: Duration,
    pub password_reset_redirect_url: String,
    pub newsletter_code_ttl: Duration,
    pub newsletter_redirect_url: String,
}

#[derive(Debug, Deserialize)]
pub struct SessionConfig {
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
    pub refresh_token_length: usize,
}

#[derive(Debug, Deserialize)]
pub struct TotpConfig {
    pub secret_length: TotpSecretLength,
}

#[derive(Debug, Deserialize)]
pub struct ContactConfig {
    pub email: EmailAddress,
}

#[derive(Debug, Deserialize)]
pub struct RecaptchaConfig {
    pub siteverify_endpoint: Url,
    pub sitekey: String,
    pub secret: String,
    pub min_score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration(pub std::time::Duration);

impl From<Duration> for std::time::Duration {
    fn from(value: Duration) -> Self {
        value.0
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut out = std::time::Duration::default();
        for part in s.split_whitespace() {
            let mut bytes = part.bytes();
            let mut seconds = 0;
            for b in bytes.by_ref() {
                match b {
                    b'0'..=b'9' => seconds = seconds * 10 + (b - b'0') as u64,
                    b's' => break,
                    b'm' => {
                        seconds *= 60;
                        break;
                    }
                    b'h' => {
                        seconds *= 3600;
                        break;
                    }
                    b'd' => {
                        seconds *= 24 * 3600;
                        break;
                    }
                    _ => return Err(serde::de::Error::custom("Invalid duration")),
                }
            }
            if bytes.next().is_some() {
                return Err(serde::de::Error::custom("Invalid duration"));
            }
            out += std::time::Duration::from_secs(seconds);
        }
        Ok(Self(out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_default_config() {
        load(&[Path::new(DEFAULT_CONFIG_PATH)]).unwrap();
    }

    #[test]
    fn parse_duration() {
        for (input, expected) in [
            ("13s", Some(13)),
            ("42m", Some(42 * 60)),
            ("7h", Some(7 * 60 * 60)),
            ("20d", Some(20 * 24 * 60 * 60)),
            ("", Some(0)),
            ("1d 2h 3m 4s", Some(((24 + 2) * 60 + 3) * 60 + 4)),
            ("xyz", None),
            ("7dd", None),
        ] {
            let input = serde_json::Value::String(input.into());
            let output = serde_json::from_value::<Duration>(input)
                .ok()
                .map(|x| x.0.as_secs());
            assert_eq!(output, expected);
        }
    }
}
