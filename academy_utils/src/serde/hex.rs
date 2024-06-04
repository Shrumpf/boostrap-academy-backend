use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<S, T>(data: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    let encoded = hex::encode(data);
    encoded.serialize(serializer)
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: TryFrom<Vec<u8>>,
{
    let input = String::deserialize(deserializer)?;
    hex::decode(input)
        .map_err(serde::de::Error::custom)?
        .try_into()
        .map_err(|_| serde::de::Error::custom("Failed to deserialize hex data"))
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[test]
    fn test() {
        let hash = serde_json::Value::String(
            "bceb5208d5d8d93c389575fdd1aae9898787ae3893790c531324bc3be0e82993".into(),
        );
        let data = serde_json::from_value::<Data>(hash.clone()).unwrap();
        let serialized = serde_json::to_value(data).unwrap();
        assert_eq!(serialized, hash);
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Data(#[serde(with = "super")] [u8; 32]);
}
