use std::net::IpAddr;

use academy_core_config_contracts::ConfigService;
use academy_core_contact_contracts::ContactService;
use academy_core_health_contracts::HealthService;
use academy_core_internal_contracts::InternalService;
use academy_core_mfa_contracts::MfaService;
use academy_core_oauth2_contracts::OAuth2Service;
use academy_core_session_contracts::SessionService;
use academy_core_user_contracts::UserService;
use academy_di::Build;
use axum::Router;
use tokio::net::TcpListener;

mod extractors;
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
    Health: HealthService,
    Config: ConfigService,
    User: UserService,
    Session: SessionService,
    Contact: ContactService,
    Mfa: MfaService,
    OAuth2: OAuth2Service,
    Internal: InternalService,
{
    pub async fn serve(self, host: IpAddr, port: u16) -> anyhow::Result<()> {
        let router = self.router();
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
