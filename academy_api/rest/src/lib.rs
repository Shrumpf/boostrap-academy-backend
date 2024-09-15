use std::net::{IpAddr, SocketAddr};

use academy_core_config_contracts::ConfigFeatureService;
use academy_core_contact_contracts::ContactFeatureService;
use academy_core_health_contracts::HealthFeatureService;
use academy_core_internal_contracts::InternalService;
use academy_core_mfa_contracts::MfaFeatureService;
use academy_core_oauth2_contracts::OAuth2FeatureService;
use academy_core_session_contracts::SessionFeatureService;
use academy_core_user_contracts::UserFeatureService;
use academy_di::Build;
use academy_utils::Apply;
use axum::Router;
use tokio::net::TcpListener;

mod extractors;
mod middlewares;
mod models;
mod routes;

#[derive(Debug, Clone, Build)]
pub struct RestServer<Health, Config, User, Session, Contact, Mfa, OAuth2, Internal> {
    health: Health,
    config: Config,
    user: User,
    session: Session,
    contact: Contact,
    mfa: Mfa,
    oauth2: OAuth2,
    internal: Internal,
}

impl<Health, Config, User, Session, Contact, Mfa, OAuth2, Internal>
    RestServer<Health, Config, User, Session, Contact, Mfa, OAuth2, Internal>
where
    Health: HealthFeatureService,
    Config: ConfigFeatureService,
    User: UserFeatureService,
    Session: SessionFeatureService,
    Contact: ContactFeatureService,
    Mfa: MfaFeatureService,
    OAuth2: OAuth2FeatureService,
    Internal: InternalService,
{
    pub async fn serve(
        self,
        host: IpAddr,
        port: u16,
        real_ip_config: Option<RealIpConfig>,
    ) -> anyhow::Result<()> {
        tracing::info!("test");
        let router = self
            .router()
            .apply(middlewares::panic_handler::add)
            .apply(middlewares::trace::add)
            .apply(middlewares::request_id::add)
            .apply(middlewares::client_ip::add(real_ip_config.map(Into::into)))
            .into_make_service_with_connect_info::<SocketAddr>();
        let listener = TcpListener::bind((host, port)).await?;
        axum::serve(listener, router).await.map_err(Into::into)
    }

    fn router(self) -> Router<()> {
        Router::new()
            .merge(routes::health::router(self.health.into()))
            .merge(routes::config::router(self.config.into()))
            .merge(routes::user::router(self.user.into()))
            .merge(routes::session::router(self.session.into()))
            .merge(routes::contact::router(self.contact.into()))
            .merge(routes::mfa::router(self.mfa.into()))
            .merge(routes::oauth2::router(self.oauth2.into()))
            .merge(routes::internal::router(self.internal.into()))
    }
}

#[derive(Debug)]
pub struct RealIpConfig {
    pub header: String,
    pub set_from: IpAddr,
}
