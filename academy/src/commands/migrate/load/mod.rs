use academy_persistence_postgres::PostgresDatabase;
use bb8::{Pool, PooledConnection};
use bb8_postgres::{tokio_postgres::NoTls, PostgresConnectionManager};
use clap::Subcommand;

use super::migration_logs;

mod auth;

#[derive(Debug, Subcommand)]
pub enum LoadCommand {
    Auth { url: String },
}

impl LoadCommand {
    pub async fn invoke(self, db: PostgresDatabase) -> anyhow::Result<()> {
        migration_logs(&db.run_migrations(None).await?, "applied");
        match self {
            LoadCommand::Auth { url } => auth::load(db, connect(url).await?).await,
        }
    }
}

type DbConnection = PooledConnection<'static, PostgresConnectionManager<NoTls>>;

async fn connect(url: String) -> anyhow::Result<DbConnection> {
    let manager = PostgresConnectionManager::new(url.parse()?, NoTls);
    let pool = Pool::builder().build(manager).await?;
    let conn = pool.get_owned().await?;
    Ok(conn)
}
