use std::sync::Arc;

use academy_di::Build;
use academy_extern_contracts::recaptcha::{RecaptchaApiService, RecaptchaSiteverifyResponse};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::http::HttpClient;

const SITEVERIFY_ENDPOINT: &str = "https://www.google.com/recaptcha/api/siteverify";

#[derive(Debug, Clone, Build)]
pub struct RecaptchaApiServiceImpl {
    config: RecaptchaApiServiceConfig,
    #[state]
    client: HttpClient,
}

#[derive(Debug, Clone)]
pub struct RecaptchaApiServiceConfig {
    siteverify_endpoint: Arc<Url>,
}

impl RecaptchaApiServiceConfig {
    pub fn new(siteverify_endpoint_override: Option<Url>) -> Self {
        Self {
            siteverify_endpoint: siteverify_endpoint_override
                .unwrap_or_else(|| SITEVERIFY_ENDPOINT.parse().unwrap())
                .into(),
        }
    }
}

impl RecaptchaApiService for RecaptchaApiServiceImpl {
    async fn siteverify(
        &self,
        response: &str,
        secret: &str,
    ) -> anyhow::Result<RecaptchaSiteverifyResponse> {
        self.client
            .post((*self.config.siteverify_endpoint).clone())
            .form(&SiteverifyRequest { response, secret })
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
