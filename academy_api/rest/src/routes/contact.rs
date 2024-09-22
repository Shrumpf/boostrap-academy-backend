use std::sync::Arc;

use academy_core_contact_contracts::{ContactFeatureService, ContactSendMessageError};
use academy_models::RecaptchaResponse;
use aide::{
    axum::{routing, ApiRouter},
    transform::TransformOperation,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    docs::TransformOperationExt,
    errors::{
        error, internal_server_error, internal_server_error_docs, recaptcha_error_docs, ApiError,
        CoundNotSendMessageDetail, RecaptchaFailedDetail,
    },
    models::{contact::ApiContactMessage, OkResponse, StringOption},
};

pub const TAG: &str = "Contact";

pub fn router(service: Arc<impl ContactFeatureService>) -> ApiRouter<()> {
    ApiRouter::new()
        .api_route(
            "/auth/contact",
            routing::post_with(send_message, send_message_docs),
        )
        .with_state(service)
        .with_path_items(|op| op.tag(TAG))
}

#[derive(Deserialize, JsonSchema)]
struct SendMessageRequest {
    #[serde(flatten)]
    message: ApiContactMessage,
    recaptcha_response: StringOption<RecaptchaResponse>,
}

async fn send_message(
    service: State<Arc<impl ContactFeatureService>>,
    Json(SendMessageRequest {
        message,
        recaptcha_response,
    }): Json<SendMessageRequest>,
) -> Response {
    match service
        .send_message(message.into(), recaptcha_response.into())
        .await
    {
        Ok(()) => Json(OkResponse).into_response(),
        Err(ContactSendMessageError::Recaptcha) => {
            error(StatusCode::PRECONDITION_FAILED, RecaptchaFailedDetail)
        }
        Err(ContactSendMessageError::Send) => {
            error(StatusCode::INTERNAL_SERVER_ERROR, CoundNotSendMessageDetail)
        }
        Err(ContactSendMessageError::Other(err)) => internal_server_error(err),
    }
}

fn send_message_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Send a message to the support team.")
        .description("A reCAPTCHA response is required if reCAPTCHA is enabled.")
        .add_response::<OkResponse>(StatusCode::OK, "The message has been sent.")
        .with(recaptcha_error_docs)
        .add_response::<ApiError<CoundNotSendMessageDetail>>(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The message could not be sent.",
        )
        .with(internal_server_error_docs)
}
