use std::{fmt, num::NonZero, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Polygenic Score ID (e.g. 'PGS000001')
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgsId(NonZero<u64>);
impl PgsId {
    pub fn new(id: u64) -> Self {
        Self::try_new(id).unwrap()
    }
    pub fn try_new(id: u64) -> Result<Self, PgsIdError> {
        Ok(Self(NonZero::new(id).ok_or(PgsIdError::Zero)?))
    }

    pub fn inner(self) -> u64 {
        self.0.get()
    }

    pub fn url(self) -> String {
        format!("https://www.pgscatalog.org/score/{self}")
    }

    pub fn is_missing(self) -> bool {
        #[allow(clippy::match_like_matches_macro)] // Formats better
        (match self.0.get() {
            4255 | 4256 | 4258 | 4910 | 4911 | 4928 | 4946 | 4947 | 4948 | 4949 | 4950 | 4951
            | 5198 | 5199 | 5200 | 5201 | 5202 | 5203 | 5204 | 5218 | 5221 | 5222 | 5223 | 5224
            | 5225 | 5226 | 5227 | 5228 | 5229 => true,
            _ => false,
        } || self.is_retired())
    }
    /// Missing from the full metadata file.
    pub fn is_retired(self) -> bool {
        #[allow(clippy::match_like_matches_macro)] // Formats better
        match self.0.get() {
            85 | 715 | 915 | 916 | 917 | 918 | 919 | 920 | 968 | 970 | 971 | 973 | 974 | 975
            | 979 | 981 | 983 | 985 | 986 | 992 | 1035 | 1083 | 1084 | 1089 | 1121 | 1122
            | 1151 | 1170 | 1171 | 1175 | 1176 | 1177 | 1178 | 1183 | 1184 | 1186 | 1187 | 1188
            | 1189 | 1190 | 1191 | 1193 | 1194 | 1195 | 1196 | 1197 | 1198 | 1201 | 1202 | 1203
            | 1204 | 1205 | 1206 | 1207 | 1208 | 1209 | 1210 | 1211 | 1212 | 1213 | 1214 | 1215
            | 1216 | 1217 | 1221 | 1222 | 1223 | 1224 | 1231 | 1269 | 1325 | 1342 | 3892 | 4939
            | 4940 | 5093 | 5094 | 5102 | 5103 | 5109 => true,

            _ => false,
        }
    }
    /// A convenience method to iterate most of the available ids for testing.
    pub fn iter_test() -> impl Iterator<Item = Self> {
        (1..=5231).map(Self::new).filter(|id| !id.is_missing())
    }
}
impl PgsId {
    pub fn retired(self) -> bool {
        self.is_missing() || self.is_retired()
    }
}

impl fmt::Display for PgsId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PGS{:06}", self.0)
    }
}
impl FromStr for PgsId {
    type Err = PgsIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const EXPECTED: &str = "Expected a PSG ID (e.g. 'PGS000001')";
        let i = utile::io::parse::numeric_id(s, "PGS", EXPECTED)
            .map_err(|_| PgsIdError::UnexpectedValue(s.to_owned()))?;
        Self::try_new(i)
    }
}

impl Serialize for PgsId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for PgsId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = PgsId;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a PSG ID (e.g. 'PGS000001')")
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
                self.visit_str(v)
            }
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&v)
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PgsIdError {
    #[error("PSG IDs cannot have value PGS000000.")]
    Zero,
    #[error("Expected a PSG ID (e.g. 'PGS000001'), found: '{0}'.")]
    UnexpectedValue(String),
}
impl From<PgsIdError> for std::io::Error {
    fn from(value: PgsIdError) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, value)
    }
}

pub mod pss {
    use std::{fmt, num::NonZero, str::FromStr};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::PgsId;

    /// Polygenic Score Sample Set ID (e.g. 'PSS000001')
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PssId(NonZero<u64>);
    impl PssId {
        pub fn new(id: u64) -> Self {
            Self::try_new(id).unwrap()
        }
        pub fn try_new(id: u64) -> Result<Self, PssIdError> {
            Ok(Self(NonZero::new(id).ok_or(PssIdError::Zero)?))
        }

        pub fn inner(self) -> u64 {
            self.0.get()
        }

        pub fn url(self, pgs_id: PgsId) -> String {
            format!("https://www.pgscatalog.org/score/{pgs_id}#{self}")
        }
    }

    impl fmt::Display for PssId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "PSS{:06}", self.0)
        }
    }
    impl FromStr for PssId {
        type Err = PssIdError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            const EXPECTED: &str = "Expected a PSS ID (e.g. 'PSS000001')";
            let i = utile::io::parse::numeric_id(s, "PSS", EXPECTED)
                .map_err(|_| PssIdError::UnexpectedValue(s.to_owned()))?;
            Self::try_new(i)
        }
    }

    impl Serialize for PssId {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.to_string().serialize(serializer)
        }
    }
    impl<'de> Deserialize<'de> for PssId {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct Visitor;
            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = PssId;
                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a PSS ID (e.g. 'PSS000001')")
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
                    self.visit_str(v)
                }
                fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    self.visit_str(&v)
                }
            }
            deserializer.deserialize_str(Visitor)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    pub enum PssIdError {
        #[error("PSS IDs cannot have value PSS000000.")]
        Zero,
        #[error("Expected a PSS ID (e.g. 'PSS000001'), found: '{0}'.")]
        UnexpectedValue(String),
    }
    impl From<PssIdError> for std::io::Error {
        fn from(value: PssIdError) -> Self {
            std::io::Error::new(std::io::ErrorKind::InvalidData, value)
        }
    }
}

pub mod ppm {
    use std::{fmt, num::NonZero, str::FromStr};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Polygenic Score Performance Metric ID (e.g. 'PPM000001')
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PpmId(NonZero<u64>);
    impl PpmId {
        pub fn new(id: u64) -> Self {
            Self::try_new(id).unwrap()
        }
        pub fn try_new(id: u64) -> Result<Self, PpmIdError> {
            Ok(Self(NonZero::new(id).ok_or(PpmIdError::Zero)?))
        }

        pub fn inner(self) -> u64 {
            self.0.get()
        }
    }

    impl fmt::Display for PpmId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "PPM{:06}", self.0)
        }
    }
    impl FromStr for PpmId {
        type Err = PpmIdError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            const EXPECTED: &str = "Expected a PPM ID (e.g. 'PPM000001')";
            let i = utile::io::parse::numeric_id(s, "PPM", EXPECTED)
                .map_err(|_| PpmIdError::UnexpectedValue(s.to_owned()))?;
            Self::try_new(i)
        }
    }

    impl Serialize for PpmId {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.to_string().serialize(serializer)
        }
    }
    impl<'de> Deserialize<'de> for PpmId {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct Visitor;
            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = PpmId;
                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a PPM ID (e.g. 'PPM000001')")
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
                    self.visit_str(v)
                }
                fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    self.visit_str(&v)
                }
            }
            deserializer.deserialize_str(Visitor)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    pub enum PpmIdError {
        #[error("PPM IDs cannot have value PPM000000.")]
        Zero,
        #[error("Expected a PPM ID (e.g. 'PPM000001'), found: '{0}'.")]
        UnexpectedValue(String),
    }
    impl From<PpmIdError> for std::io::Error {
        fn from(value: PpmIdError) -> Self {
            std::io::Error::new(std::io::ErrorKind::InvalidData, value)
        }
    }
}

pub mod pgp {
    use std::{fmt, num::NonZero, str::FromStr};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Polygenic Score Publication ID (e.g. 'PGP000001')
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PgpId(NonZero<u64>);
    impl PgpId {
        pub fn new(id: u64) -> Self {
            Self::try_new(id).unwrap()
        }
        pub fn try_new(id: u64) -> Result<Self, PgpIdError> {
            Ok(Self(NonZero::new(id).ok_or(PgpIdError::Zero)?))
        }

        pub fn inner(self) -> u64 {
            self.0.get()
        }

        pub fn url(self) -> String {
            format!("https://www.pgscatalog.org/publication/{self}")
        }
    }

    impl fmt::Display for PgpId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "PGP{:06}", self.0)
        }
    }
    impl FromStr for PgpId {
        type Err = PgpIdError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            const EXPECTED: &str = "Expected a PGP ID (e.g. 'PGP000001')";
            let i = utile::io::parse::numeric_id(s, "PGP", EXPECTED)
                .map_err(|_| PgpIdError::UnexpectedValue(s.to_owned()))?;
            Self::try_new(i)
        }
    }

    impl Serialize for PgpId {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.to_string().serialize(serializer)
        }
    }
    impl<'de> Deserialize<'de> for PgpId {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct Visitor;
            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = PgpId;
                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a PGP ID (e.g. 'PGP000001')")
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
                    self.visit_str(v)
                }
                fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    self.visit_str(&v)
                }
            }
            deserializer.deserialize_str(Visitor)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    pub enum PgpIdError {
        #[error("PGP IDs cannot have value PGP000000.")]
        Zero,
        #[error("Expected a PGP ID (e.g. 'PGP000001'), found: '{0}'.")]
        UnexpectedValue(String),
    }
    impl From<PgpIdError> for std::io::Error {
        fn from(value: PgpIdError) -> Self {
            std::io::Error::new(std::io::ErrorKind::InvalidData, value)
        }
    }
}
