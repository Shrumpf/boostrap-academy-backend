use std::panic::AssertUnwindSafe;

use anyhow::anyhow;
use axum::{
    extract::Request,
    middleware::{from_fn, Next},
    response::Response,
    Router,
};
use futures::FutureExt;

use crate::routes::internal_server_error;

pub fn add<S: Clone + Send + Sync + 'static>(router: Router<S>) -> Router<S> {
    router.layer(from_fn(middleware))
}

async fn middleware(request: Request, next: Next) -> Response {
    match AssertUnwindSafe(next.run(request)).catch_unwind().await {
        Ok(response) => response,
        Err(_) => internal_server_error(anyhow!("request handler panicked")),
    }
}
