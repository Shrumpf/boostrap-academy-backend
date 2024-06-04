use std::{sync::LazyLock, time::Duration};

use academy_models::mfa::{TotpDevice, TotpSecret};
use academy_persistence_contracts::mfa::MfaRepository;
use uuid::uuid;

use crate::user::{ADMIN, FOO};

pub static ALL_TOTP_DEVICES: LazyLock<Vec<&TotpDevice>> =
    LazyLock::new(|| vec![&ADMIN_TOTP_1, &FOO_TOTP_1]);

pub static ADMIN_TOTP_1: LazyLock<TotpDevice> = LazyLock::new(|| TotpDevice {
    id: uuid!("75a9def2-688f-4211-9fc7-750eb89600cd").into(),
    user_id: ADMIN.user.id,
    enabled: true,
    created_at: ADMIN.user.created_at + Duration::from_secs(600),
});

pub static FOO_TOTP_1: LazyLock<TotpDevice> = LazyLock::new(|| TotpDevice {
    id: uuid!("24532ed6-9126-4b8a-b0b3-c6979ff0549e").into(),
    user_id: FOO.user.id,
    enabled: false,
    created_at: FOO.user.created_at + Duration::from_secs(2 * 24 * 3600),
});

pub fn totp_secret_of(device: &TotpDevice) -> TotpSecret {
    TotpSecret::try_new(device.id.into_bytes().into_iter().rev().collect()).unwrap()
}

pub async fn create<Txn: Send + Sync + 'static>(
    txn: &mut Txn,
    repo: impl MfaRepository<Txn>,
) -> anyhow::Result<()> {
    for &totp_device in &*ALL_TOTP_DEVICES {
        let secret = totp_secret_of(totp_device);
        repo.create_totp_device(txn, totp_device, &secret)
            .await
            .unwrap();
    }
    Ok(())
}
