use std::sync::{Arc, LazyLock};

use academy_di::Build;
use academy_extern_contracts::vat::VatApiService;
use anyhow::Context;
use regex::Regex;
use serde::Deserialize;
use url::Url;

use crate::http::HttpClient;

/// https://ec.europa.eu/taxation_customs/vies/#/vat-validation
/// https://ec.europa.eu/taxation_customs/vies/#/technical-information
const VALIDATE_ENDPOINT: &str = "https://ec.europa.eu/taxation_customs/vies/rest-api/ms/";

#[derive(Debug, Clone, Build)]
pub struct VatApiServiceImpl {
    config: VatApiServiceConfig,
    #[di(default)]
    http: HttpClient,
}

#[derive(Debug, Clone)]
pub struct VatApiServiceConfig {
    validate_endpoint: Arc<Url>,
}

impl VatApiServiceConfig {
    pub fn new(validate_endpoint_override: Option<Url>) -> Self {
        Self {
            validate_endpoint: validate_endpoint_override
                .unwrap_or_else(|| VALIDATE_ENDPOINT.parse().unwrap())
                .into(),
        }
    }
}

static VAT_ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^([A-Z]{2}) *([0-9A-Z]+)$").unwrap());

impl VatApiService for VatApiServiceImpl {
    async fn is_vat_id_valid(&self, vat_id: &str) -> anyhow::Result<bool> {
        let Some(captures) = VAT_ID_REGEX.captures(vat_id) else {
            return Ok(false);
        };

        let url = self
            .config
            .validate_endpoint
            .join(&format!("{}/vat/{}", &captures[1], &captures[2]))
            .context("Failed to build vat validate URL")?;

        self.http
            .get(url)
            .send()
            .await
            .context("Failed to send vat validate request")?
            .error_for_status()
            .context("Vat validate request returned an error")?
            .json::<IsVatIdValidResponse>()
            .await
            .map(|x| x.is_valid)
            .context("Failed to deserialize vat validate response")
    }
}

#[derive(Deserialize)]
struct IsVatIdValidResponse {
    #[serde(rename = "isValid")]
    is_valid: bool,
}
