use std::borrow::Cow;

use academy_models::{
    email_address::EmailAddress,
    user::{
        UserBio, UserCity, UserComposite, UserCountry, UserDisplayName, UserFilter, UserFirstName,
        UserId, UserIdOrSelf, UserLastName, UserName, UserPassword, UserStreet, UserTags,
        UserVatId, UserZipCode,
    },
    SearchTerm,
};
use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApiUser {
    pub id: UserId,
    pub name: UserName,
    pub display_name: UserDisplayName,
    pub email: Option<EmailAddress>,
    pub email_verified: bool,
    pub registration: i64,
    pub last_login: Option<i64>,
    pub last_name_change: Option<i64>,
    pub enabled: bool,
    pub admin: bool,
    pub password: bool,
    pub mfa_enabled: bool,
    pub description: UserBio,
    pub tags: UserTags,
    pub newsletter: bool,
    pub business: Option<bool>,
    pub first_name: Option<UserFirstName>,
    pub last_name: Option<UserLastName>,
    pub street: Option<UserStreet>,
    pub zip_code: Option<UserZipCode>,
    pub city: Option<UserCity>,
    pub country: Option<UserCountry>,
    pub vat_id: Option<UserVatId>,
    pub can_buy_coins: bool,
    pub can_receive_coins: bool,
    pub avatar_url: Option<Url>,
}

impl From<UserComposite> for ApiUser {
    fn from(user_composite: UserComposite) -> Self {
        let can_buy_coins = user_composite.can_buy_coins();
        let can_receive_coins = user_composite.can_receive_coins();

        let UserComposite {
            user,
            profile,
            details,
            invoice_info,
        } = user_composite;

        let avatar_url = user.email.as_ref().map(get_avatar_url);

        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            email_verified: user.email_verified,
            registration: user.created_at.timestamp(),
            last_login: user.last_login.map(|x| x.timestamp()),
            last_name_change: user.last_name_change.map(|x| x.timestamp()),
            enabled: user.enabled,
            admin: user.admin,
            newsletter: user.newsletter,

            display_name: profile.display_name,
            description: profile.bio,
            tags: profile.tags,

            mfa_enabled: details.mfa_enabled,
            password: details.password_login,

            business: invoice_info.business,
            first_name: invoice_info.first_name,
            last_name: invoice_info.last_name,
            street: invoice_info.street,
            zip_code: invoice_info.zip_code,
            city: invoice_info.city,
            country: invoice_info.country,
            vat_id: invoice_info.vat_id,

            avatar_url,
            can_buy_coins,
            can_receive_coins,
        }
    }
}

#[derive(Deserialize)]
pub struct ApiUserFilter {
    pub name: Option<SearchTerm>,
    pub email: Option<SearchTerm>,
    pub enabled: Option<bool>,
    pub admin: Option<bool>,
    pub mfa_enabled: Option<bool>,
    pub email_verified: Option<bool>,
    pub newsletter: Option<bool>,
}

impl From<ApiUserFilter> for UserFilter {
    fn from(value: ApiUserFilter) -> Self {
        Self {
            name: value.name,
            email: value.email,
            enabled: value.enabled,
            admin: value.admin,
            mfa_enabled: value.mfa_enabled,
            email_verified: value.email_verified,
            newsletter: value.newsletter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiUserIdOrSelf {
    UserId(UserId),
    Slf,
}

impl<'de> Deserialize<'de> for ApiUserIdOrSelf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        pub enum UserIdOrString {
            UserId(UserId),
            String(String),
        }

        let result = UserIdOrString::deserialize(deserializer)?;
        match result {
            UserIdOrString::UserId(user_id) => Ok(ApiUserIdOrSelf::UserId(user_id)),
            UserIdOrString::String(s) if matches!(s.to_lowercase().as_str(), "me" | "self") => {
                Ok(ApiUserIdOrSelf::Slf)
            }
            _ => Err(serde::de::Error::custom("Invalid user id")),
        }
    }
}

impl JsonSchema for ApiUserIdOrSelf {
    fn schema_name() -> String {
        UserId::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        UserId::json_schema(gen)
    }

    fn is_referenceable() -> bool {
        UserId::is_referenceable()
    }

    fn schema_id() -> Cow<'static, str> {
        UserId::schema_id()
    }
}

impl From<ApiUserIdOrSelf> for UserIdOrSelf {
    fn from(value: ApiUserIdOrSelf) -> Self {
        match value {
            ApiUserIdOrSelf::UserId(user_id) => Self::UserId(user_id),
            ApiUserIdOrSelf::Slf => Self::Slf,
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PathUserId {
    pub user_id: UserId,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PathUserIdOrSelf {
    pub user_id: ApiUserIdOrSelf,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum ApiUserPasswordOrEmpty {
    #[serde(rename = "")]
    Empty,
    #[serde(untagged)]
    Password(UserPassword),
}

fn get_avatar_url(email: &EmailAddress) -> Url {
    let hash = Sha256::new()
        .chain_update(email.as_str().trim().to_lowercase())
        .finalize();
    format!("https://gravatar.com/avatar/{hash:x}")
        .parse()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_api_user_id_or_self() {
        enum Kind {
            UserId,
            Slf,
            Invalid,
        }

        for (input, kind) in [
            ("3b1c09f9-4971-4376-89e0-ccc478fcd213", Kind::UserId),
            ("self", Kind::Slf),
            ("me", Kind::Slf),
            ("SELF", Kind::Slf),
            ("ME", Kind::Slf),
            ("asdf", Kind::Invalid),
            ("", Kind::Invalid),
        ] {
            let result =
                serde_json::from_value::<ApiUserIdOrSelf>(serde_json::Value::String(input.into()));
            match kind {
                Kind::UserId => assert_eq!(
                    result.unwrap(),
                    ApiUserIdOrSelf::UserId(UserId::new(input.parse().unwrap()))
                ),
                Kind::Slf => assert_eq!(result.unwrap(), ApiUserIdOrSelf::Slf),
                Kind::Invalid => assert!(result.is_err()),
            }
        }
    }

    #[test]
    fn deserialize_api_user_password_or_empty() {
        let result =
            serde_json::from_value::<ApiUserPasswordOrEmpty>(serde_json::Value::String("".into()));
        assert_eq!(result.unwrap(), ApiUserPasswordOrEmpty::Empty);

        let result =
            serde_json::from_value::<ApiUserPasswordOrEmpty>(serde_json::Value::String("a".into()));
        assert_eq!(
            result.unwrap(),
            ApiUserPasswordOrEmpty::Password("a".try_into().unwrap())
        );

        let input = "a".repeat(UserPassword::MAX_LENGTH + 1);
        let result =
            serde_json::from_value::<ApiUserPasswordOrEmpty>(serde_json::Value::String(input));
        assert!(result.is_err());
    }

    #[test]
    fn get_avatar_url() {
        let result = super::get_avatar_url(&"Test@Example.com".parse().unwrap());
        assert_eq!(result.as_str(), "https://gravatar.com/avatar/973dfe463ec85785f5f95af5ba3906eedb2d931c24e69824a89ea65dba4e813b");
    }
}
