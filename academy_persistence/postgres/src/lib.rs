use std::{collections::HashSet, fmt::Write, time::Duration};

use academy_models::Sha256Hash;
use academy_persistence_contracts::{Database, Transaction};
use anyhow::anyhow;
use bb8::{Pool, PooledConnection};
use bb8_postgres::{
    tokio_postgres::{self, NoTls},
    PostgresConnectionManager,
};
use ouroboros::self_referencing;

pub mod mfa;
pub mod oauth2;
pub mod session;
pub mod user;

type PgClient = tokio_postgres::Client;
type PgPooledConnection = PooledConnection<'static, PostgresConnectionManager<NoTls>>;
type PgTransaction<'a> = tokio_postgres::Transaction<'a>;

#[derive(Debug, Clone)]
pub struct PostgresDatabase {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

#[derive(Debug)]
pub struct PostgresDatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
}

impl PostgresDatabase {
    pub async fn connect(config: &PostgresDatabaseConfig) -> anyhow::Result<Self> {
        let manager = PostgresConnectionManager::new(config.url.parse()?, NoTls);
        let pool = Pool::builder()
            .max_size(config.max_connections)
            .min_idle(config.min_connections)
            .connection_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .build(manager)
            .await?;

        Ok(Self { pool })
    }

    #[cfg(feature = "dummy")]
    pub async fn dummy() -> Self {
        let manager = PostgresConnectionManager::new("".parse().unwrap(), NoTls);
        Self {
            pool: Pool::builder().build_unchecked(manager),
        }
    }

    pub async fn list_migrations(&self) -> anyhow::Result<Vec<MigrationStatus>> {
        let conn = self.pool.get().await?;
        create_migrations_table(&conn).await?;
        list_migrations(&conn).await
    }

    pub async fn run_migrations(&self, cnt: Option<usize>) -> anyhow::Result<Vec<&'static str>> {
        let mut conn = self.pool.get().await?;
        create_migrations_table(&conn).await?;

        let mut out = Vec::new();
        let insert_migration = conn
            .prepare("insert into _migrations (name) values ($1);")
            .await?;
        let pending = list_migrations(&conn)
            .await?
            .into_iter()
            .filter_map(|MigrationStatus { migration, applied }| (!applied).then_some(migration))
            .take(cnt.unwrap_or(usize::MAX));
        for migration in pending {
            let txn = conn.transaction().await?;
            txn.batch_execute(migration.up).await?;
            txn.execute(&insert_migration, &[&migration.name]).await?;
            txn.commit().await?;
            out.push(migration.name);
        }
        Ok(out)
    }

    pub async fn revert_migrations(&self, cnt: Option<usize>) -> anyhow::Result<Vec<&'static str>> {
        let mut conn = self.pool.get().await?;
        create_migrations_table(&conn).await?;

        let mut out = Vec::new();
        let revert_migration = conn
            .prepare("delete from _migrations where name=$1")
            .await?;
        let applied = list_migrations(&conn)
            .await?
            .into_iter()
            .rev()
            .filter_map(|MigrationStatus { migration, applied }| applied.then_some(migration))
            .take(cnt.unwrap_or(usize::MAX));
        for migration in applied {
            let txn = conn.transaction().await?;
            txn.batch_execute(migration.down).await?;
            txn.execute(&revert_migration, &[&migration.name]).await?;
            txn.commit().await?;
            out.push(migration.name);
        }

        Ok(out)
    }

    pub async fn reset(&self) -> anyhow::Result<()> {
        self.execute("drop schema public cascade; create schema public;")
            .await
    }

    pub async fn execute(&self, query: &str) -> anyhow::Result<()> {
        let conn = self.pool.get().await?;
        conn.batch_execute(query).await?;
        Ok(())
    }
}

impl Database for PostgresDatabase {
    type Transaction = PostgresTransaction;

    async fn begin_transaction(&self) -> anyhow::Result<Self::Transaction> {
        let conn = self.pool.get_owned().await?;
        Ok(PostgresTransactionAsyncSendTryBuilder {
            conn,
            txn_builder: |conn| Box::pin(async move { conn.transaction().await.map(Some) }),
        }
        .try_build()
        .await?)
    }

    async fn ping(&self) -> anyhow::Result<()> {
        let conn = self.pool.get().await?;
        conn.query_one("select 1", &[])
            .await
            .map_err(Into::into)
            .map(|row| row.get::<_, i32>(0) == 1)
            .and_then(|ok| {
                ok.then_some(())
                    .ok_or_else(|| anyhow!("Failed to ping database"))
            })
    }
}

#[self_referencing]
pub struct PostgresTransaction {
    conn: PgPooledConnection,
    #[borrows(mut conn)]
    #[covariant]
    txn: Option<PgTransaction<'this>>,
}

impl PostgresTransaction {
    fn txn(&self) -> &PgTransaction<'_> {
        self.borrow_txn().as_ref().unwrap()
    }
}

impl Transaction for PostgresTransaction {
    async fn commit(mut self) -> anyhow::Result<()> {
        self.with_txn_mut(|txn| txn.take())
            .unwrap()
            .commit()
            .await
            .map_err(Into::into)
    }

    async fn rollback(mut self) -> anyhow::Result<()> {
        self.with_txn_mut(|txn| txn.take())
            .unwrap()
            .rollback()
            .await
            .map_err(Into::into)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Migration {
    pub name: &'static str,
    pub up: &'static str,
    pub down: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct MigrationStatus {
    pub migration: Migration,
    pub applied: bool,
}

pub const MIGRATIONS: &[Migration] = include!(env!("MIGRATIONS"));

async fn create_migrations_table(conn: &PgClient) -> anyhow::Result<()> {
    conn.execute(
        "create table if not exists _migrations (name text primary key);",
        &[],
    )
    .await?;
    Ok(())
}

async fn list_migrations(conn: &PgClient) -> anyhow::Result<Vec<MigrationStatus>> {
    let applied = conn
        .query("select name from _migrations;", &[])
        .await?
        .into_iter()
        .map(|row| row.get(0))
        .collect::<HashSet<String>>();

    Ok(MIGRATIONS
        .iter()
        .map(|&migration| MigrationStatus {
            migration,
            applied: applied.contains(migration.name),
        })
        .collect())
}

fn decode_sha256hash(hash: Vec<u8>) -> anyhow::Result<Sha256Hash> {
    hash.try_into()
        .map(Sha256Hash)
        .map_err(|x| anyhow!("Failed to decode SHA256 hash {x:?}"))
}

macro_rules! columns {
    ($vis:vis $ident:ident as $alias:literal: $fst:literal $(, $col:literal)* $(,)?) => {
        ::paste::paste! {
            #[allow(unused)]
            $vis const [< $ident:snake:upper _CNT >]: usize = [ $fst $(, $col)* ].len();
            $vis const [< $ident:snake:upper _COLS >]: &str = ::core::concat!( '"', $alias, "\".\"", $fst, '"' $(, ", \"" , $alias, "\".\"", $col, '"' )* );
            #[allow(unused)]
            $vis const [< $ident:snake:upper _COL_NAMES >]: &str = ::core::concat!( '"', $fst, '"' $(, ", \"", $col, '"' )* );
        }
    };
}
use columns;

fn arg_indices(indices: impl IntoIterator<Item = usize>) -> String {
    let mut it = indices.into_iter();
    let mut out = String::new();
    if let Some(x) = it.next() {
        write!(&mut out, "${x}").unwrap();
    }
    for x in it {
        write!(&mut out, ", ${x}").unwrap();
    }
    out
}
