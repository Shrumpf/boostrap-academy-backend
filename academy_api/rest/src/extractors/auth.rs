use std::convert::Infallible;

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

pub struct ApiToken(pub String);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for ApiToken {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map(|t| t.token().into())
                .unwrap_or_default(),
        ))
    }
}
