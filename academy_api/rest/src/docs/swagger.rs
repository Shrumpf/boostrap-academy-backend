use axum::{
    response::{Html, IntoResponse, Response},
    routing, Router,
};

const SWAGGER_UI_HTML: &str = concat!(
    r#"
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <title>API Documentation - Bootstrap Academy</title>
    <style type="text/css">
"#,
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/swagger-ui/swagger-ui.css"
    )),
    r#"
</style>
  </head>

  <body>
    <div id="swagger-ui"></div>
    <script>
"#,
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/swagger-ui/swagger-ui-bundle.js"
    )),
    r#"
      window.ui = SwaggerUIBundle({
        url: "/openapi.json",
        dom_id: '#swagger-ui',
        deepLinking: true,
        displayRequestDuration: true,
      });
    </script>
  </body>
</html>
"#
);

pub fn router() -> Router<()> {
    Router::new().route("/docs", routing::get(serve_swagger_ui))
}

async fn serve_swagger_ui() -> Response {
    Html(SWAGGER_UI_HTML).into_response()
}
