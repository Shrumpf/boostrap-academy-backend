use academy_di::Build;
use academy_models::{
    oauth2::{OAuth2Link, OAuth2LinkId, OAuth2UserInfo},
    user::UserId,
};
use academy_persistence_contracts::oauth2::{OAuth2RepoError, OAuth2Repository};
use bb8_postgres::tokio_postgres::{self, Row};
use uuid::Uuid;

use crate::{arg_indices, columns, ColumnCounter, PostgresTransaction};

#[derive(Debug, Clone, Build)]
pub struct PostgresOAuth2Repository;

columns!(oauth2_links as "ol": "id", "user_id", "provider_id", "created_at", "remote_user_id", "remote_user_name");

impl OAuth2Repository<PostgresTransaction> for PostgresOAuth2Repository {
    async fn list_links_by_user(
        &self,
        txn: &mut PostgresTransaction,
        user_id: UserId,
    ) -> anyhow::Result<Vec<OAuth2Link>> {
        txn.txn()
            .query(
                &format!("select {OAUTH2_LINKS_COLS} from oauth2_links ol where user_id=$1"),
                &[&*user_id],
            )
            .await
            .map_err(Into::into)
            .and_then(|rows| {
                rows.into_iter()
                    .map(|row| decode_oauth2_link(&row, &mut Default::default()))
                    .collect()
            })
    }

    async fn get_link(
        &self,
        txn: &mut PostgresTransaction,
        link_id: OAuth2LinkId,
    ) -> anyhow::Result<Option<OAuth2Link>> {
        txn.txn()
            .query_opt(
                &format!("select {OAUTH2_LINKS_COLS} from oauth2_links ol where id=$1"),
                &[&*link_id],
            )
            .await
            .map_err(Into::into)
            .and_then(|row| {
                row.map(|row| decode_oauth2_link(&row, &mut Default::default()))
                    .transpose()
            })
    }

    async fn create_link(
        &self,
        txn: &mut PostgresTransaction,
        oauth2_link: &OAuth2Link,
    ) -> Result<(), OAuth2RepoError> {
        txn.txn()
            .execute(
                &format!(
                    "insert into oauth2_links ({OAUTH2_LINKS_COL_NAMES}) values ({})",
                    arg_indices(1..=OAUTH2_LINKS_CNT)
                ),
                &[
                    &*oauth2_link.id,
                    &*oauth2_link.user_id,
                    &*oauth2_link.provider_id,
                    &oauth2_link.created_at,
                    &*oauth2_link.remote_user.id,
                    &*oauth2_link.remote_user.name,
                ],
            )
            .await
            .map(|_| ())
            .map_err(map_oauth2_repo_error)
    }

    async fn delete_link(
        &self,
        txn: &mut PostgresTransaction,
        link_id: OAuth2LinkId,
    ) -> anyhow::Result<bool> {
        txn.txn()
            .execute("delete from oauth2_links where id=$1", &[&*link_id])
            .await
            .map(|n| n != 0)
            .map_err(Into::into)
    }
}

fn decode_oauth2_link(row: &Row, cnt: &mut ColumnCounter) -> anyhow::Result<OAuth2Link> {
    Ok(OAuth2Link {
        id: row.get::<_, Uuid>(cnt.idx()).into(),
        user_id: row.get::<_, Uuid>(cnt.idx()).into(),
        provider_id: row.get::<_, String>(cnt.idx()).into(),
        created_at: row.get(cnt.idx()),
        remote_user: OAuth2UserInfo {
            id: row.get::<_, String>(cnt.idx()).try_into()?,
            name: row.get::<_, String>(cnt.idx()).try_into()?,
        },
    })
}

fn map_oauth2_repo_error(err: tokio_postgres::Error) -> OAuth2RepoError {
    match err.as_db_error() {
        Some(err) if err.constraint() == Some("oauth2_links_provider_id_remote_user_id_idx") => {
            OAuth2RepoError::Conflict
        }
        _ => OAuth2RepoError::Other(err.into()),
    }
}
