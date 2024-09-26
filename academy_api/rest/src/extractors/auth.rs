use std::{convert::Infallible, marker::PhantomData};

use aide::{gen::GenContext, openapi::Operation, OperationInput};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

/// Extract Bearer API tokens from the Authorization header
pub struct ApiToken<T: ApiTokenType = UserApiToken>(pub String, PhantomData<T>);

pub trait ApiTokenType: private::Sealed {
    const NAME: &str;
}
pub struct UserApiToken;
pub struct InternalApiToken;
impl ApiTokenType for UserApiToken {
    const NAME: &str = "Token";
}
impl ApiTokenType for InternalApiToken {
    const NAME: &str = "InternalToken";
}

mod private {
    use super::*;
    pub trait Sealed {}
    impl Sealed for UserApiToken {}
    impl Sealed for InternalApiToken {}
}

#[async_trait]
impl<S: Send + Sync, T: ApiTokenType> FromRequestParts<S> for ApiToken<T> {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(
            parts
                .headers
                .get(AUTHORIZATION)
                .and_then(|x| x.to_str().ok())
                .map(|x| x.strip_prefix("Bearer ").unwrap_or(x).into())
                .unwrap_or_default(),
            Default::default(),
        ))
    }
}

impl<T: ApiTokenType> OperationInput for ApiToken<T> {
    fn operation_input(_ctx: &mut GenContext, operation: &mut Operation) {
        operation
            .security
            .push([(T::NAME.into(), Vec::new())].into());
    }
}
