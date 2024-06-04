use academy_config::DatabaseConfig;
use academy_persistence_postgres::{PostgresDatabase, PostgresDatabaseConfig};

pub async fn connect(config: &DatabaseConfig) -> anyhow::Result<PostgresDatabase> {
    PostgresDatabase::connect(&PostgresDatabaseConfig {
        url: config.url.clone(),
        max_connections: config.max_connections,
        min_connections: config.min_connections,
        acquire_timeout: config.acquire_timeout.into(),
        idle_timeout: config.idle_timeout.map(Into::into),
        max_lifetime: config.max_lifetime.map(Into::into),
    })
    .await
}
