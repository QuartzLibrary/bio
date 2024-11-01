use flate2::read::MultiGzDecoder;
use std::{
    collections::HashMap,
    io::{BufReader, Read},
};

use super::{download_and_cache_latest_contig, parse, AltGenotype, B38Contig};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SerdeRecord {
    #[serde(rename = "CHROM")]
    pub contig: B38Contig,
    #[serde(rename = "POS")]
    pub position: u64,
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "REF")]
    pub reference_allele: String,
    #[serde(rename = "ALT")]
    #[serde(with = "string_sequence")]
    pub alternate_alleles: Vec<AltGenotype>,
    #[serde(rename = "QUAL")]
    pub quality: u8,
    #[serde(rename = "FILTER")]
    pub filter: String,
    #[serde(rename = "INFO")]
    pub info: String,
    #[serde(rename = "FORMAT")]
    pub format: String,
    #[serde(flatten)]
    pub samples: HashMap<String, String>,
}

impl SerdeRecord {
    #[allow(dead_code)]
    async fn load_contig_(
        contig: B38Contig,
    ) -> Result<impl Iterator<Item = Result<SerdeRecord, csv::Error>>, std::io::Error> {
        let fs_entry = download_and_cache_latest_contig(contig.download_url()).await?;

        let mut reader = BufReader::new(MultiGzDecoder::new(fs_entry.get()?));
        parse::comments::skip(&mut reader)?;

        {
            let mut buf = [0u8];
            reader.read_exact(&mut buf).unwrap();
            assert_eq!(buf, [b'#']);
        }

        Ok(csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(reader)
            .into_deserialize())
    }
}

mod string_sequence {
    use core::fmt;
    use std::{fmt::Write, str::FromStr};

    use serde::{de::Error, Deserialize};

    const SEPARATOR: char = ',';

    pub fn serialize<S, T>(s: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: fmt::Display,
    {
        let mut string = String::new();
        let mut first = true;
        for item in s {
            if !first {
                string.push(SEPARATOR);
            }
            string.write_fmt(format_args!("{item}")).unwrap();
            first = false;
        }
        serializer.serialize_str(&string)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: FromStr,
        <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    {
        let string: &str = Deserialize::deserialize(deserializer)?;
        utile::io::parse::string_sequence::str(string, SEPARATOR).map_err(D::Error::custom)
    }
}
