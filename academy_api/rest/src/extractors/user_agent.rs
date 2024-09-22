use std::convert::Infallible;

use aide::{
    gen::GenContext,
    openapi::{
        HeaderStyle, Operation, Parameter, ParameterData, ParameterSchemaOrContent, SchemaObject,
    },
    operation::add_parameters,
    OperationInput,
};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::USER_AGENT, request::Parts},
};
use schemars::JsonSchema;

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

impl OperationInput for UserAgent {
    fn operation_input(ctx: &mut GenContext, operation: &mut Operation) {
        let parameter = Parameter::Header {
            parameter_data: ParameterData {
                name: "User-Agent".into(),
                description: None,
                required: false,
                deprecated: None,
                format: ParameterSchemaOrContent::Schema(SchemaObject {
                    json_schema: String::json_schema(&mut ctx.schema),
                    external_docs: None,
                    example: None,
                }),
                example: None,
                examples: Default::default(),
                explode: None,
                extensions: Default::default(),
            },
            style: HeaderStyle::Simple,
        };

        add_parameters(ctx, operation, [parameter]);
    }
}
