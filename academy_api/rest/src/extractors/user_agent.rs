use std::convert::Infallible;

use aide::OperationInput;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::USER_AGENT, request::Parts},
};

pub struct UserAgent(pub Option<String>);

#[async_trait]
impl<S> FromRequestParts<S> for UserAgent {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Some(header) = parts.headers.get(USER_AGENT) else {
            return Ok(Self(None));
        };

        let value = String::from_utf8_lossy(header.as_bytes()).into_owned();

        Ok(Self(Some(value)))
    }
}

impl OperationInput for UserAgent {}
