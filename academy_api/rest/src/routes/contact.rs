use std::sync::Arc;

use academy_core_contact_contracts::{ContactSendMessageError, ContactService};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};

use super::{error, internal_server_error};
use crate::models::contact::ApiContactMessage;

pub fn router(service: Arc<impl ContactService>) -> Router<()> {
    Router::new()
        .route("/auth/contact", routing::post(send_message))
        .with_state(service)
}

async fn send_message(
    service: State<Arc<impl ContactService>>,
    Json(message): Json<ApiContactMessage>,
) -> Response {
    match service.send_message(message.into()).await {
        Ok(()) => Json(true).into_response(),
        Err(ContactSendMessageError::Send) => {
            error(StatusCode::INTERNAL_SERVER_ERROR, "Could not send message")
        }
        Err(ContactSendMessageError::Other(err)) => internal_server_error(err),
    }
}
