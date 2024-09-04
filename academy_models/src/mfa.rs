use std::sync::LazyLock;

use academy_utils::patch::Patch;
use chrono::{DateTime, Utc};
use nutype::nutype;
use regex::Regex;

use crate::{
    hyphenated_code_regex,
    macros::{id, nutype_string},
    user::UserId,
    Sha256Hash,
};

id!(TotpDeviceId);

#[derive(Debug, Clone, PartialEq, Eq, Patch)]
pub struct TotpDevice {
    #[no_patch]
    pub id: TotpDeviceId,
    #[no_patch]
    pub user_id: UserId,
    pub enabled: bool,
    #[no_patch]
    pub created_at: DateTime<Utc>,
}

nutype_string!(TotpCode(validate(regex = TOTP_CODE_REGEX)));
pub static TOTP_CODE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[0-9]{6}$").unwrap());

#[nutype(validate(predicate = |x| x.len() >= 16), derive(Debug, Clone, PartialEq, Eq, Deref, TryFrom))]
pub struct TotpSecret(Vec<u8>);

#[nutype(
    validate(greater_or_equal = 16),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        Deref,
        TryFrom,
        Serialize,
        Deserialize
    )
)]
pub struct TotpSecretLength(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TotpSetup {
    /// The base32 encoded totp secret.
    pub secret: String,
}

nutype_string!(MfaRecoveryCode(
    sanitize(uppercase),
    validate(regex = MFA_RECOVERY_CODE_REGEX),
));

pub static MFA_RECOVERY_CODE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    hyphenated_code_regex(MfaRecoveryCode::CHUNK_COUNT, MfaRecoveryCode::CHUNK_SIZE)
});

impl MfaRecoveryCode {
    pub const CHUNK_COUNT: usize = 4;
    pub const CHUNK_SIZE: usize = 6;
}

#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, Deref, From, Serialize, Deserialize,))]
pub struct MfaRecoveryCodeHash(Sha256Hash);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MfaAuthenticateCommand {
    pub totp_code: Option<TotpCode>,
    pub recovery_code: Option<MfaRecoveryCode>,
}
