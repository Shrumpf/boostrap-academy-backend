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
    errors::{
        error, internal_server_error, ApiError, CoundNotSendMessageDetail, RecaptchaFailedDetail,
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
        .response::<200, Json<OkResponse>>()
        .response_with::<412, Json<ApiError<RecaptchaFailedDetail>>, _>(|op| {
            op.description(
                "reCAPTCHA is enabled but no valid reCAPTCHA response has been provided.",
            )
        })
        .response_with::<500, Json<ApiError<CoundNotSendMessageDetail>>, _>(|op| {
            op.description("The message could not be sent.")
        })
        .tag(TAG)
}
