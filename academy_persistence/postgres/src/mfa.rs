use std::fmt::Write;

use academy_di::Build;
use academy_models::{
    mfa::{MfaRecoveryCodeHash, TotpDevice, TotpDeviceId, TotpDevicePatchRef, TotpSecret},
    user::UserId,
};
use academy_persistence_contracts::mfa::MfaRepository;
use academy_utils::patch::PatchValue;
use bb8_postgres::tokio_postgres::{types::ToSql, Row};
use uuid::Uuid;

use crate::{arg_indices, columns, decode_sha256hash, ColumnCounter, PostgresTransaction};

#[derive(Debug, Clone, Build)]
pub struct PostgresMfaRepository;

columns!(totp_device as "td": "id", "user_id", "enabled", "created_at");

impl MfaRepository<PostgresTransaction> for PostgresMfaRepository {
    async fn list_totp_devices_by_user(
        &self,
        txn: &mut PostgresTransaction,
        user_id: UserId,
    ) -> anyhow::Result<Vec<TotpDevice>> {
        txn.txn()
            .query(
                &format!("select {TOTP_DEVICE_COLS} from totp_devices td where user_id=$1"),
                &[&*user_id],
            )
            .await
            .map_err(Into::into)
            .and_then(|rows| {
                rows.into_iter()
                    .map(|row| decode_totp_device(&row, &mut Default::default()))
                    .collect()
            })
    }

    async fn create_totp_device(
        &self,
        txn: &mut PostgresTransaction,
        totp_device: &TotpDevice,
        secret: &TotpSecret,
    ) -> anyhow::Result<()> {
        txn.txn()
            .execute(
                &format!(
                    "insert into totp_devices ({TOTP_DEVICE_COL_NAMES}) values ({})",
                    arg_indices(1..=TOTP_DEVICE_CNT)
                ),
                &[
                    &*totp_device.id,
                    &*totp_device.user_id,
                    &totp_device.enabled,
                    &totp_device.created_at,
                ],
            )
            .await?;

        self.save_totp_device_secret(txn, totp_device.id, secret)
            .await?;

        Ok(())
    }

    async fn update_totp_device<'a>(
        &self,
        txn: &mut PostgresTransaction,
        totp_device_id: TotpDeviceId,
        TotpDevicePatchRef { enabled }: TotpDevicePatchRef<'a>,
    ) -> anyhow::Result<bool> {
        let mut query = "update totp_devices set id=id".to_owned();
        let mut params: Vec<&(dyn ToSql + Sync)> = vec![&*totp_device_id];

        if let PatchValue::Update(enabled) = enabled {
            params.push(enabled);
            write!(&mut query, ", enabled=${}", params.len()).unwrap();
        }

        query.push_str(" where id=$1");

        txn.txn()
            .execute(&query, &params)
            .await
            .map(|n| n != 0)
            .map_err(Into::into)
    }

    async fn delete_totp_devices_by_user(
        &self,
        txn: &mut PostgresTransaction,
        user_id: UserId,
    ) -> anyhow::Result<()> {
        txn.txn()
            .execute("delete from totp_devices where user_id=$1", &[&*user_id])
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    async fn list_enabled_totp_device_secrets_by_user(
        &self,
        txn: &mut PostgresTransaction,
        user_id: UserId,
    ) -> anyhow::Result<Vec<TotpSecret>> {
        txn.txn()
            .query(
                "select secret from totp_device_secrets inner join totp_devices using(id) where \
                 user_id=$1 and enabled",
                &[&*user_id],
            )
            .await
            .map_err(Into::into)
            .and_then(|rows| {
                rows.into_iter()
                    .map(|row| decode_totp_device_secret(row.get(0)))
                    .collect()
            })
    }

    async fn get_totp_device_secret(
        &self,
        txn: &mut PostgresTransaction,
        totp_device_id: TotpDeviceId,
    ) -> anyhow::Result<TotpSecret> {
        txn.txn()
            .query_one(
                "select secret from totp_device_secrets where id=$1",
                &[&*totp_device_id],
            )
            .await
            .map_err(Into::into)
            .and_then(|row| decode_totp_device_secret(row.get(0)))
    }

    async fn save_totp_device_secret(
        &self,
        txn: &mut PostgresTransaction,
        totp_device_id: TotpDeviceId,
        secret: &TotpSecret,
    ) -> anyhow::Result<()> {
        txn.txn()
            .execute(
                "insert into totp_device_secrets (id, secret) values ($1, $2) on conflict (id) do \
                 update set secret=$2",
                &[&*totp_device_id, &**secret],
            )
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    async fn get_mfa_recovery_code_hash(
        &self,
        txn: &mut PostgresTransaction,
        user_id: UserId,
    ) -> anyhow::Result<Option<MfaRecoveryCodeHash>> {
        txn.txn()
            .query_opt(
                "select code from mfa_recovery_codes where user_id=$1",
                &[&*user_id],
            )
            .await
            .map_err(Into::into)
            .and_then(|row| {
                row.map(|row| decode_sha256hash(row.get(0)).map(Into::into))
                    .transpose()
            })
    }

    async fn save_mfa_recovery_code_hash(
        &self,
        txn: &mut PostgresTransaction,
        user_id: UserId,
        recovery_code_hash: MfaRecoveryCodeHash,
    ) -> anyhow::Result<()> {
        txn.txn()
            .execute(
                "insert into mfa_recovery_codes (user_id, code) values ($1, $2) on conflict \
                 (user_id) do update set code=$2",
                &[&*user_id, &recovery_code_hash.0.as_slice()],
            )
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    async fn delete_mfa_recovery_code_hash(
        &self,
        txn: &mut PostgresTransaction,
        user_id: UserId,
    ) -> anyhow::Result<()> {
        txn.txn()
            .execute(
                "delete from mfa_recovery_codes where user_id=$1",
                &[&*user_id],
            )
            .await
            .map(|_| ())
            .map_err(Into::into)
    }
}

fn decode_totp_device(row: &Row, cnt: &mut ColumnCounter) -> anyhow::Result<TotpDevice> {
    Ok(TotpDevice {
        id: row.get::<_, Uuid>(cnt.idx()).into(),
        user_id: row.get::<_, Uuid>(cnt.idx()).into(),
        enabled: row.get(cnt.idx()),
        created_at: row.get(cnt.idx()),
    })
}

fn decode_totp_device_secret(data: Vec<u8>) -> anyhow::Result<TotpSecret> {
    data.try_into().map_err(Into::into)
}
