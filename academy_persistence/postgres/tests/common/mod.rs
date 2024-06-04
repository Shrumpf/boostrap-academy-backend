use std::path::Path;

use academy_config::DEFAULT_CONFIG_PATH;
use academy_persistence_contracts::{Database, Transaction};
use academy_persistence_postgres::{
    mfa::PostgresMfaRepository, session::PostgresSessionRepository, user::PostgresUserRepository,
    PostgresDatabase, PostgresDatabaseConfig,
};

pub type Db = PostgresDatabase;

pub async fn setup() -> Db {
    let db = setup_clean().await;

    db.run_migrations(None).await.unwrap();

    let mut txn = db.begin_transaction().await.unwrap();

    academy_demo::create(
        &mut txn,
        PostgresUserRepository,
        PostgresSessionRepository,
        PostgresMfaRepository,
    )
    .await
    .unwrap();

    txn.commit().await.unwrap();

    db
}

pub async fn setup_clean() -> Db {
    let mut paths = vec![Path::new(DEFAULT_CONFIG_PATH)];
    let extra = std::env::var("EXTRA_CONFIG");
    if let Ok(extra) = &extra {
        paths.push(Path::new(extra));
    }
    let config = academy_config::load(&paths).unwrap();

    let db = Db::connect(&PostgresDatabaseConfig {
        url: config.database.url,
        max_connections: config.database.max_connections,
        min_connections: config.database.min_connections,
        acquire_timeout: config.database.acquire_timeout.into(),
        idle_timeout: config.database.idle_timeout.map(Into::into),
        max_lifetime: config.database.max_lifetime.map(Into::into),
    })
    .await
    .unwrap();

    db.reset().await.unwrap();
    db
}
