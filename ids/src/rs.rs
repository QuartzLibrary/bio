use std::{fmt, num::NonZero, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RsId(NonZero<u64>);
impl RsId {
    pub fn new(id: u64) -> Self {
        Self::try_new(id).unwrap()
    }
    pub fn try_new(id: u64) -> Result<Self, RsIdError> {
        Ok(Self(NonZero::new(id).ok_or(RsIdError::Zero)?))
    }
}
impl fmt::Display for RsId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rs{}", self.0)
    }
}
impl FromStr for RsId {
    type Err = RsIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const EXPECTED: &str = "Expected a rsID (e.g. 'rs123')";
        let i = utile::io::parse::numeric_id(s, "rs", EXPECTED)
            .map_err(|_| RsIdError::UnexpectedValue(s.to_owned()))?;
        Self::try_new(i)
    }
}

impl Serialize for RsId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for RsId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = RsId;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a rsID (e.g. 'rs123')")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(|e| serde::de::Error::custom(e))
            }
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(|e| serde::de::Error::custom(e))
            }
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(|e| serde::de::Error::custom(e))
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RsIdError {
    #[error("rsIDs cannot have value 0.")]
    Zero,
    #[error("Expected a rsID (e.g. 'rs123'), found: '{0}'.")]
    UnexpectedValue(String),
}
impl From<RsIdError> for std::io::Error {
    fn from(value: RsIdError) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, value)
    }
}
