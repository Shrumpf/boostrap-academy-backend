use std::fmt::Write;

use academy_di::Build;
use academy_models::{
    session::{Session, SessionId, SessionPatchRef, SessionRefreshTokenHash},
    user::UserId,
};
use academy_persistence_contracts::session::SessionRepository;
use academy_utils::patch::PatchValue;
use bb8_postgres::tokio_postgres::{types::ToSql, Row};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{arg_indices, columns, decode_sha256hash, PostgresTransaction};

#[derive(Debug, Clone, Build)]
pub struct PostgresSessionRepository;

columns!(session as "s": "id", "user_id", "device_name", "created_at", "updated_at");

impl SessionRepository<PostgresTransaction> for PostgresSessionRepository {
    async fn get(
        &self,
        txn: &mut PostgresTransaction,
        session_id: SessionId,
    ) -> anyhow::Result<Option<Session>> {
        txn.txn()
            .query_opt(
                &format!("select {SESSION_COLS} from sessions s where id=$1",),
                &[&*session_id],
            )
            .await
            .map_err(Into::into)
            .and_then(|row| row.map(|row| decode_session(&row, &mut 0)).transpose())
    }

    async fn get_by_refresh_token_hash(
        &self,
        txn: &mut PostgresTransaction,
        refresh_token_hash: SessionRefreshTokenHash,
    ) -> anyhow::Result<Option<Session>> {
        txn.txn()
            .query_opt(
                &format!(
                    "select {SESSION_COLS} from sessions s inner join session_refresh_tokens rt \
                     on s.id=rt.session_id where rt.refresh_token_hash=$1"
                ),
                &[&refresh_token_hash.0.as_slice()],
            )
            .await
            .map_err(Into::into)
            .and_then(|row| row.map(|row| decode_session(&row, &mut 0)).transpose())
    }

    async fn list_by_user(
        &self,
        txn: &mut PostgresTransaction,
        user_id: UserId,
    ) -> anyhow::Result<Vec<Session>> {
        txn.txn()
            .query(
                &format!("select {SESSION_COLS} from sessions s where user_id=$1"),
                &[&*user_id],
            )
            .await
            .map_err(Into::into)
            .and_then(|rows| {
                rows.into_iter()
                    .map(|row| decode_session(&row, &mut 0))
                    .collect()
            })
    }

    async fn create(&self, txn: &mut PostgresTransaction, session: &Session) -> anyhow::Result<()> {
        txn.txn()
            .execute(
                &format!(
                    "insert into sessions ({SESSION_COL_NAMES}) values ({})",
                    arg_indices(1..=SESSION_CNT)
                ),
                &[
                    &*session.id,
                    &*session.user_id,
                    &session.device_name.as_deref(),
                    &session.created_at,
                    &session.updated_at,
                ],
            )
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    async fn update(
        &self,
        txn: &mut PostgresTransaction,
        session_id: SessionId,
        SessionPatchRef {
            device_name,
            updated_at,
        }: SessionPatchRef<'_>,
    ) -> anyhow::Result<bool> {
        let mut query = "update sessions set id=id".to_owned();
        let mut params: Vec<&(dyn ToSql + Sync)> = vec![&*session_id];

        let device_name = device_name.map(|x| x.as_ref().map(|x| x.as_str()));

        if let PatchValue::Update(device_name) = &device_name {
            params.push(device_name);
            write!(&mut query, ", device_name=${}", params.len()).unwrap();
        }
        if let PatchValue::Update(updated_at) = updated_at {
            params.push(updated_at);
            write!(&mut query, ", updated_at=${}", params.len()).unwrap();
        }

        query.push_str(" where id=$1");

        txn.txn()
            .execute(&query, &params)
            .await
            .map(|n| n != 0)
            .map_err(Into::into)
    }

    async fn delete(
        &self,
        txn: &mut PostgresTransaction,
        session_id: SessionId,
    ) -> anyhow::Result<bool> {
        txn.txn()
            .execute("delete from sessions where id=$1", &[&*session_id])
            .await
            .map(|n| n != 0)
            .map_err(Into::into)
    }

    async fn delete_by_user(
        &self,
        txn: &mut PostgresTransaction,
        user_id: UserId,
    ) -> anyhow::Result<()> {
        txn.txn()
            .execute("delete from sessions where user_id=$1", &[&*user_id])
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    async fn delete_by_updated_at(
        &self,
        txn: &mut PostgresTransaction,
        updated_at: DateTime<Utc>,
    ) -> anyhow::Result<u64> {
        txn.txn()
            .execute("delete from sessions where updated_at<$1", &[&updated_at])
            .await
            .map_err(Into::into)
    }

    async fn list_refresh_token_hashes_by_user(
        &self,
        txn: &mut PostgresTransaction,
        user_id: UserId,
    ) -> anyhow::Result<Vec<SessionRefreshTokenHash>> {
        txn.txn()
            .query(
                "select rt.refresh_token_hash from session_refresh_tokens rt inner join sessions \
                 s on s.id=rt.session_id where s.user_id=$1",
                &[&*user_id],
            )
            .await
            .map_err(Into::into)
            .and_then(|rows| {
                rows.into_iter()
                    .map(|row| decode_sha256hash(row.get(0)).map(Into::into))
                    .collect()
            })
    }

    async fn get_refresh_token_hash(
        &self,
        txn: &mut PostgresTransaction,
        session_id: SessionId,
    ) -> anyhow::Result<Option<SessionRefreshTokenHash>> {
        txn.txn()
            .query_opt(
                "select refresh_token_hash from session_refresh_tokens where session_id=$1",
                &[&*session_id],
            )
            .await
            .map_err(Into::into)
            .and_then(|row| {
                row.map(|row| decode_sha256hash(row.get(0)).map(Into::into))
                    .transpose()
            })
    }

    async fn save_refresh_token_hash(
        &self,
        txn: &mut PostgresTransaction,
        session_id: SessionId,
        refresh_token_hash: SessionRefreshTokenHash,
    ) -> anyhow::Result<()> {
        txn.txn()
            .execute(
                "insert into session_refresh_tokens (session_id, refresh_token_hash) values ($1, \
                 $2) on conflict (session_id) do update set refresh_token_hash=$2",
                &[&*session_id, &refresh_token_hash.0.as_slice()],
            )
            .await
            .map(|_| ())
            .map_err(Into::into)
    }
}

fn decode_session(row: &Row, offset: &mut usize) -> anyhow::Result<Session> {
    let mut idx = || {
        *offset += 1;
        *offset - 1
    };

    Ok(Session {
        id: row.get::<_, Uuid>(idx()).into(),
        user_id: row.get::<_, Uuid>(idx()).into(),
        device_name: row
            .get::<_, Option<String>>(idx())
            .map(TryInto::try_into)
            .transpose()?,
        created_at: row.get(idx()),
        updated_at: row.get(idx()),
    })
}
