use academy_cache_contracts::CacheService;
use academy_config::Config;
use academy_di::Provides;
use academy_email_contracts::EmailService;
use academy_persistence_contracts::Database;
use tracing::info;

use crate::{
    cache, database, email,
    environment::{types::RestServer, ConfigProvider, Provider},
};

pub async fn serve(config: Config) -> anyhow::Result<()> {
    info!("Connecting to database");
    let database = database::connect(&config.database).await?;
    database.ping().await?;

    info!("Applying pending migrations");
    let mut applied = false;
    for name in database.run_migrations(None).await? {
        info!("Applied {name}");
        applied = true;
    }
    if !applied {
        info!("No migrations pending");
    }

    info!("Connecting to valkey cache");
    let cache = cache::connect(&config.cache).await?;
    cache.ping().await?;

    info!("Connecting to smtp server");
    let email = email::connect(&config.email).await?;
    email.ping().await?;

    let config_provider = ConfigProvider::new(&config)?;
    let mut provider = Provider::new(config_provider, database, cache, email);
    let server: RestServer = provider.provide();
    info!(
        "Starting http server on {}:{}",
        config.http.host, config.http.port
    );
    server.serve(config.http.host, config.http.port).await
}
