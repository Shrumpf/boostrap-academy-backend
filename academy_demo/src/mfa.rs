use std::{collections::HashMap, sync::LazyLock, time::Duration};

use academy_models::mfa::{TotpDevice, TotpDeviceId, TotpSecret};
use academy_persistence_contracts::mfa::MfaRepository;
use uuid::uuid;

use crate::user::{ADMIN2, FOO};

pub static ALL_TOTP_DEVICES: LazyLock<Vec<&TotpDevice>> =
    LazyLock::new(|| vec![&ADMIN2_TOTP_1, &FOO_TOTP_1]);

pub static ADMIN2_TOTP_1: LazyLock<TotpDevice> = LazyLock::new(|| TotpDevice {
    id: uuid!("75a9def2-688f-4211-9fc7-750eb89600cd").into(),
    user_id: ADMIN2.user.id,
    enabled: true,
    created_at: ADMIN2.user.created_at + Duration::from_secs(600),
});

pub static FOO_TOTP_1: LazyLock<TotpDevice> = LazyLock::new(|| TotpDevice {
    id: uuid!("24532ed6-9126-4b8a-b0b3-c6979ff0549e").into(),
    user_id: FOO.user.id,
    enabled: false,
    created_at: FOO.user.created_at + Duration::from_secs(2 * 24 * 3600),
});

pub static TOTP_SECRETS: LazyLock<HashMap<TotpDeviceId, TotpSecret>> = LazyLock::new(|| {
    [
        (ADMIN2_TOTP_1.id, "CF3ABXI2PIN5AIKTFBWHTSMA24"),
        (FOO_TOTP_1.id, "6MKF3WY2IYYEVEKW4O4W6NNUBY"),
    ]
    .map(|(id, secret)| (id, decode_secret(secret)))
    .into()
});

fn decode_secret(secret: &str) -> TotpSecret {
    base32::decode(base32::Alphabet::Rfc4648 { padding: false }, secret)
        .unwrap()
        .try_into()
        .unwrap()
}

pub async fn create<Txn: Send + Sync + 'static>(
    txn: &mut Txn,
    repo: impl MfaRepository<Txn>,
) -> anyhow::Result<()> {
    for &totp_device in &*ALL_TOTP_DEVICES {
        repo.create_totp_device(txn, totp_device, &TOTP_SECRETS[&totp_device.id])
            .await
            .unwrap();
    }
    Ok(())
}
