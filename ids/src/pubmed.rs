use std::{fmt, num::NonZero, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub struct PubmedId(NonZero<u64>);
impl PubmedId {
    pub fn new(id: u64) -> Self {
        Self::try_new(id).unwrap()
    }
    pub fn try_new(id: u64) -> Result<Self, PubmedIdError> {
        Ok(Self(NonZero::new(id).ok_or(PubmedIdError::Zero)?))
    }
    pub fn inner(self) -> u64 {
        self.0.get()
    }
    pub fn url(self) -> String {
        format!("https://pubmed.ncbi.nlm.nih.gov/{}/", self.0)
    }
}
impl fmt::Display for PubmedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl FromStr for PubmedId {
    type Err = PubmedIdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id: u64 = s
            .parse()
            .map_err(|_| PubmedIdError::UnexpectedValue(s.to_owned()))?;
        Self::try_new(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PubmedIdError {
    #[error("Pubmed IDs cannot have value 0.")]
    Zero,
    #[error("Expected a Pubmed ID (e.g. '123'), found: '{0}'.")]
    UnexpectedValue(String),
}
impl From<PubmedIdError> for std::io::Error {
    fn from(value: PubmedIdError) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, value)
    }
}
