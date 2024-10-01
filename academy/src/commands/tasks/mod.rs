use academy_config::Config;
use academy_persistence_contracts::{session::SessionRepository, Database, Transaction};
use academy_persistence_postgres::session::PostgresSessionRepository;
use anyhow::Context;
use chrono::Utc;
use clap::Subcommand;
use tracing::info;

use crate::database;

#[derive(Debug, Subcommand)]
pub enum TaskCommand {
    /// Remove expired records from the database.
    PruneDatabase,
}

impl TaskCommand {
    pub async fn invoke(self, config: Config) -> anyhow::Result<()> {
        match self {
            TaskCommand::PruneDatabase => prune_database(config).await,
        }
    }
}

async fn prune_database(config: Config) -> anyhow::Result<()> {
    let db = database::connect(&config.database).await?;
    let mut txn = db.begin_transaction().await?;

    let session_repo = PostgresSessionRepository;
    let now = Utc::now();
    let pruned = session_repo
        .delete_by_updated_at(&mut txn, now - config.session.refresh_token_ttl.0)
        .await
        .context("Failed to prune sessions")?;
    info!("Pruned {pruned} expired sessions.");

    txn.commit().await?;

    Ok(())
}
