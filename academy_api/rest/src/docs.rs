use std::fmt::Write;

use aide::{
    gen::in_context,
    openapi::{ReferenceOr, Response, Responses},
    transform::{TransformOperation, TransformResponse},
    OperationOutput,
};
use axum::{http::StatusCode, Json, Router};
use schemars::{
    schema::{Metadata, Schema, SchemaObject, SubschemaValidation},
    JsonSchema,
};

mod swagger;

pub fn router() -> Router<()> {
    swagger::router()
}

pub trait TransformOperationExt {
    fn add_response<R: JsonSchema>(
        self,
        code: StatusCode,
        description: impl Into<Option<&'static str>>,
    ) -> Self;

    fn add_response_with<R: JsonSchema>(
        self,
        code: StatusCode,
        description: impl Into<Option<&'static str>>,
        transform: impl FnOnce(
            TransformResponse<<Json<R> as OperationOutput>::Inner>,
        ) -> TransformResponse<<Json<R> as OperationOutput>::Inner>,
    ) -> Self;
}

impl TransformOperationExt for TransformOperation<'_> {
    fn add_response<R: JsonSchema>(
        self,
        code: StatusCode,
        description: impl Into<Option<&'static str>>,
    ) -> Self {
        self.add_response_with::<R>(code, description, |op| op)
    }

    fn add_response_with<R: JsonSchema>(
        mut self,
        code: StatusCode,
        description: impl Into<Option<&'static str>>,
        transform: impl FnOnce(
            TransformResponse<<Json<R> as OperationOutput>::Inner>,
        ) -> TransformResponse<<Json<R> as OperationOutput>::Inner>,
    ) -> Self {
        let mut response =
            in_context(|ctx| Json::<R>::operation_response(ctx, &mut Default::default()).unwrap());
        if let Some(description) = description.into() {
            response.description = description.into();
        }
        let _ = transform(TransformResponse::new(&mut response));

        let operation = self.inner_mut();
        let responses = match &mut operation.responses {
            Some(responses) => responses,
            None => operation.responses.insert(Default::default()),
        };

        merge_into_responses(code, response, responses);

        self
    }
}

fn merge_into_responses(code: StatusCode, src: Response, dst: &mut Responses) {
    let code = aide::openapi::StatusCode::Code(code.as_u16());

    let dst = match dst.responses.get_mut(&code) {
        Some(dst) => dst,
        None => {
            dst.responses.insert(code, ReferenceOr::Item(src));
            return;
        }
    };

    let ReferenceOr::Item(dst) = dst else {
        unimplemented!("cannot merge references yet")
    };

    for (k, v) in src.content {
        let d = dst.content.entry(k).or_insert_with(Default::default);
        match d.schema.take() {
            Some(s) => {
                let s = s.json_schema.into_object();
                let mut schemas = if s
                    .subschemas
                    .as_ref()
                    .and_then(|s| s.any_of.as_ref())
                    .is_some_and(|s| !s.is_empty())
                {
                    s.subschemas.unwrap().any_of.unwrap()
                } else {
                    vec![schema_with_description(s, dst.description.clone())]
                };

                schemas.extend(
                    v.schema
                        .map(|v| {
                            schema_with_description(
                                v.json_schema.into_object(),
                                src.description.clone(),
                            )
                        })
                        .filter(|v| !schemas.contains(v)),
                );

                let descriptions = schemas
                    .iter()
                    .map(|s| {
                        match s {
                            Schema::Bool(_) => None,
                            Schema::Object(obj) => obj.metadata.as_ref(),
                        }
                        .and_then(|m| m.title.as_deref())
                        .unwrap_or_default()
                    })
                    .collect::<Vec<_>>();
                if descriptions.len() == 1 {
                    dst.description = descriptions[0].into();
                } else {
                    dst.description =
                        "There are multiple possible responses with this status code:".into();
                    for d in descriptions {
                        write!(&mut dst.description, "\n- {d}").unwrap();
                    }
                }

                d.schema = Some(aide::openapi::SchemaObject {
                    json_schema: SchemaObject {
                        subschemas: Some(
                            SubschemaValidation {
                                any_of: Some(schemas),
                                ..Default::default()
                            }
                            .into(),
                        ),
                        ..Default::default()
                    }
                    .into(),
                    external_docs: None,
                    example: None,
                });
            }
            None => d.schema = v.schema,
        }
    }
}

fn schema_with_description(schema_object: SchemaObject, description: String) -> Schema {
    SchemaObject {
        metadata: Some(
            Metadata {
                title: Some(description),
                ..Default::default()
            }
            .into(),
        ),
        ..schema_object
    }
    .into()
}
