use std::{fmt, io::Read, path::Path, str::FromStr};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A sha256 hash, serialized as lowercase hex.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sha256Hash([u8; 32]);

impl Sha256Hash {
    pub fn of_bytes(bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Self(hasher.finalize().into())
    }
    pub fn of_file(path: &Path) -> std::io::Result<Self> {
        let mut hasher = Sha256::new();
        let mut file = std::fs::File::open(path)?;
        std::io::copy(&mut file, &mut hasher)?;
        Ok(Self(hasher.finalize().into()))
    }
    pub fn of_reader(mut reader: impl Read) -> std::io::Result<Self> {
        let mut hasher = Sha256::new();
        std::io::copy(&mut reader, &mut hasher)?;
        Ok(Self(hasher.finalize().into()))
    }
    pub fn to_hex(self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Sha256Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in self.0 {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}
impl fmt::Debug for Sha256Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl FromStr for Sha256Hash {
    type Err = std::io::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("expected 32 bytes, got {}", s.len() / 2),
            ));
        }
        let mut bytes = [0u8; 32];
        for (i, chunk) in s.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            bytes[i] = u8::from_str_radix(
                std::str::from_utf8(chunk).map_err(crate::io::invalid_data)?,
                16,
            )
            .map_err(crate::io::invalid_data)?;
        }
        Ok(Self(bytes))
    }
}
impl Serialize for Sha256Hash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}
impl<'de> Deserialize<'de> for Sha256Hash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prints_lowercase_hex() {
        let mut bytes = [0u8; 32];
        bytes[0] = 0xff;
        bytes[1] = 0xaa;
        assert_eq!(
            Sha256Hash(bytes).to_string(),
            "ffaa000000000000000000000000000000000000000000000000000000000000"
        );
    }
}
