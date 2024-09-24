use aide::redoc::Redoc;
use axum::Router;

pub fn router() -> Router<()> {
    Router::new().route("/redoc", Redoc::new("/openapi.json").axum_route().into())
}
