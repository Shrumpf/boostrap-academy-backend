use academy_models::pagination::{PaginationLimit, PaginationSlice};
use serde::{Deserialize, Serialize};

pub mod contact;
pub mod oauth2;
pub mod session;
pub mod user;

#[derive(Serialize)]
pub struct ApiError {
    pub detail: &'static str,
}

#[derive(Deserialize)]
pub struct ApiPaginationSlice {
    #[serde(default)]
    pub limit: PaginationLimit,
    #[serde(default)]
    pub offset: u64,
}

impl From<ApiPaginationSlice> for PaginationSlice {
    fn from(value: ApiPaginationSlice) -> Self {
        Self {
            limit: value.limit,
            offset: value.offset,
        }
    }
}

pub enum StringOption<T> {
    Some(T),
    None,
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for StringOption<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        enum Inner<T> {
            #[serde(rename = "")]
            Empty,
            #[serde(untagged)]
            Some(T),
        }

        match Option::<Inner<T>>::deserialize(deserializer)? {
            Some(Inner::Some(x)) => Ok(Self::Some(x)),
            Some(Inner::Empty) | None => Ok(Self::None),
        }
    }
}

impl<T> From<StringOption<T>> for Option<T> {
    fn from(value: StringOption<T>) -> Self {
        match value {
            StringOption::Some(x) => Some(x),
            StringOption::None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use academy_models::user::UserPassword;

    use super::*;

    #[test]
    fn string_option() {
        let x = serde_json::Value::String("test".into());
        let y = Option::<UserPassword>::from(
            serde_json::from_value::<StringOption<UserPassword>>(x).unwrap(),
        );
        assert_eq!(y.unwrap().into_inner(), "test");

        let x = serde_json::Value::String("".into());
        let y = Option::<UserPassword>::from(
            serde_json::from_value::<StringOption<UserPassword>>(x).unwrap(),
        );
        assert_eq!(y, None);

        let x = serde_json::Value::Null;
        let y = Option::<UserPassword>::from(
            serde_json::from_value::<StringOption<UserPassword>>(x).unwrap(),
        );
        assert_eq!(y, None);
    }
}
