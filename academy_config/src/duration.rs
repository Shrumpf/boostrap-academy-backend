use std::ops::Deref;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration(pub std::time::Duration);

impl From<Duration> for std::time::Duration {
    fn from(value: Duration) -> Self {
        value.0
    }
}

impl Deref for Duration {
    type Target = std::time::Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut out = std::time::Duration::default();
        for part in s.split_whitespace() {
            let mut bytes = part.bytes();
            let mut seconds = 0;
            for b in bytes.by_ref() {
                match b {
                    b'0'..=b'9' => seconds = seconds * 10 + (b - b'0') as u64,
                    b's' => break,
                    b'm' => {
                        seconds *= 60;
                        break;
                    }
                    b'h' => {
                        seconds *= 3600;
                        break;
                    }
                    b'd' => {
                        seconds *= 24 * 3600;
                        break;
                    }
                    _ => return Err(serde::de::Error::custom("Invalid duration")),
                }
            }
            if bytes.next().is_some() {
                return Err(serde::de::Error::custom("Invalid duration"));
            }
            out += std::time::Duration::from_secs(seconds);
        }
        Ok(Self(out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_duration() {
        for (input, expected) in [
            ("13s", Some(13)),
            ("42m", Some(42 * 60)),
            ("7h", Some(7 * 60 * 60)),
            ("20d", Some(20 * 24 * 60 * 60)),
            ("", Some(0)),
            ("1d 2h 3m 4s", Some(((24 + 2) * 60 + 3) * 60 + 4)),
            ("xyz", None),
            ("7dd", None),
        ] {
            let input = serde_json::Value::String(input.into());
            let output = serde_json::from_value::<Duration>(input)
                .ok()
                .map(|x| x.0.as_secs());
            assert_eq!(output, expected);
        }
    }
}
