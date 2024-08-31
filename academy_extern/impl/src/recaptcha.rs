use std::sync::Arc;

use academy_di::Build;
use academy_extern_contracts::recaptcha::{RecaptchaApiService, RecaptchaSiteverifyResponse};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Build)]
pub struct RecaptchaApiServiceImpl {
    config: Arc<RecaptchaApiServiceConfig>,
    #[state]
    client: Arc<Client>,
}

#[derive(Debug)]
pub struct RecaptchaApiServiceConfig {
    pub siteverify_endpoint: Url,
    pub secret: String,
}

impl RecaptchaApiService for RecaptchaApiServiceImpl {
    async fn siteverify(&self, response: &str) -> anyhow::Result<RecaptchaSiteverifyResponse> {
        self.client
            .post(self.config.siteverify_endpoint.clone())
            .json(&SiteverifyRequest {
                response,
                secret: &self.config.secret,
            })
            .send()
            .await?
            .error_for_status()?
            .json::<SiteverifyResponse>()
            .await
            .map(Into::into)
            .map_err(Into::into)
    }
}

#[derive(Serialize)]
struct SiteverifyRequest<'a> {
    response: &'a str,
    secret: &'a str,
}

#[derive(Deserialize)]
struct SiteverifyResponse {
    success: bool,
    score: Option<f64>,
}

impl From<SiteverifyResponse> for RecaptchaSiteverifyResponse {
    fn from(value: SiteverifyResponse) -> Self {
        Self {
            success: value.success,
            score: value.score,
        }
    }
}
