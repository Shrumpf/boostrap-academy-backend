use std::convert::Infallible;

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

pub struct ApiToken(pub String);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for ApiToken {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(
            parts
                .headers
                .get(AUTHORIZATION)
                .and_then(|x| x.to_str().ok())
                .map(|x| x.strip_prefix("Bearer ").unwrap_or(x).into())
                .unwrap_or_default(),
        ))
    }
}
