use std::sync::LazyLock;

use academy_utils::patch::Patch;
use chrono::{DateTime, Utc};
use email_address::EmailAddress;
use nutype::nutype;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    macros::{id, nutype_string},
    SearchTerm,
};

pub static USER_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^[a-zA-Z0-9_-]{1,32}$").unwrap());

id!(UserId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UserIdOrSelf {
    UserId(UserId),
    Slf,
}

impl From<UserId> for UserIdOrSelf {
    fn from(value: UserId) -> Self {
        Self::UserId(value)
    }
}

impl UserIdOrSelf {
    pub fn unwrap_or(self, self_user_id: UserId) -> UserId {
        match self {
            UserIdOrSelf::UserId(user_id) => user_id,
            UserIdOrSelf::Slf => self_user_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserComposite {
    pub user: User,
    pub profile: UserProfile,
    pub details: UserDetails,
}

#[derive(Debug, Clone, PartialEq, Eq, Patch)]
pub struct User {
    #[no_patch]
    pub id: UserId,
    pub name: UserName,
    pub email: Option<EmailAddress>,
    pub email_verified: bool,
    #[no_patch]
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub last_name_change: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub admin: bool,
    pub newsletter: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Patch)]
pub struct UserProfile {
    pub display_name: UserDisplayName,
    pub bio: UserBio,
    pub tags: UserTags,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserDetails {
    pub mfa_enabled: bool,
}

nutype_string!(UserName(validate(regex = USER_NAME_REGEX)));
nutype_string!(UserDisplayName(validate(
    len_char_min = 1,
    len_char_max = 64
)));

nutype_string!(UserPassword(validate(
    len_char_min = 1,
    len_char_max = UserPassword::MAX_LENGTH
)));
impl UserPassword {
    pub const MAX_LENGTH: usize = 4096;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserNameOrEmailAddress {
    Name(UserName),
    Email(EmailAddress),
}

nutype_string!(UserBio(
    validate(len_char_max = 1024),
    derive(Default),
    default = ""
));

nutype_string!(UserTag(validate(len_char_min = 1, len_char_max = 64)));

#[nutype(
    validate(predicate = |x| x.len() <= 8),
    derive(Debug, Clone, PartialEq, Eq, Deref, Default, TryFrom, Serialize, Deserialize),
    default = Vec::new(),
)]
pub struct UserTags(Vec<UserTag>);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserFilter {
    pub name: Option<SearchTerm>,
    pub email: Option<SearchTerm>,
    pub enabled: Option<bool>,
    pub admin: Option<bool>,
    pub mfa_enabled: Option<bool>,
    pub email_verified: Option<bool>,
    pub newsletter: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_name_or_email() {
        enum Kind {
            Name,
            Email,
            Invalid,
        }
        use Kind::{Email, Invalid, Name};

        for (input, kind) in [
            ("foobar", Name),
            ("foo@bar.com", Email),
            ("@", Invalid),
            ("", Invalid),
            ("foo bar", Invalid),
        ] {
            let result = serde_json::from_value::<UserNameOrEmailAddress>(
                serde_json::Value::String(input.into()),
            );

            match kind {
                Name => assert_eq!(
                    result.unwrap(),
                    UserNameOrEmailAddress::Name(input.try_into().unwrap())
                ),
                Email => assert_eq!(
                    result.unwrap(),
                    UserNameOrEmailAddress::Email(input.parse().unwrap())
                ),
                Invalid => assert!(result.is_err()),
            }
        }
    }
}
