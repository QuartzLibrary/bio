use std::{fmt, io::BufRead, str::FromStr};

use hyperx::header::{ContentDisposition, DispositionParam, DispositionType, Header};
use reqwest::header::{HeaderMap, CONTENT_DISPOSITION, CONTENT_LENGTH};

pub mod read_ext {
    use std::{
        io::{Error, Read},
        pin::pin,
    };

    use tokio::io::{AsyncRead, AsyncReadExt};

    pub trait ReadInto: Read {
        fn read_into_vec(&mut self) -> Result<Vec<u8>, Error>;
        fn read_into_string(&mut self) -> Result<String, Error>;
    }
    impl<R: Read> ReadInto for R {
        fn read_into_vec(&mut self) -> Result<Vec<u8>, Error> {
            let mut buf = Vec::new();
            self.read_to_end(&mut buf)?;
            Ok(buf)
        }
        fn read_into_string(&mut self) -> Result<String, Error> {
            let mut buf = String::new();
            self.read_to_string(&mut buf)?;
            Ok(buf)
        }
    }

    #[allow(async_fn_in_trait)] // TODO
    pub trait AsyncReadInto: AsyncRead {
        async fn read_into_vec(self) -> Result<Vec<u8>, Error>;
        async fn read_into_string(self) -> Result<String, Error>;
    }
    impl<R: AsyncRead> AsyncReadInto for R {
        async fn read_into_vec(self) -> Result<Vec<u8>, Error> {
            let mut buf = Vec::new();
            pin!(self).read_to_end(&mut buf).await?;
            Ok(buf)
        }
        async fn read_into_string(self) -> Result<String, Error> {
            let mut buf = String::new();
            pin!(self).read_to_string(&mut buf).await?;
            Ok(buf)
        }
    }
}

pub mod read {
    use std::{io::BufRead, str::FromStr};

    pub fn line<T>(buf: &mut Vec<u8>, reader: &mut impl BufRead) -> Result<T, std::io::Error>
    where
        T: FromStr,
        <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    {
        buf.clear();
        reader.read_until(b'\n', buf)?;
        let mut buf = &buf[..buf.len() - 1];
        if buf.last() == Some(&b'\r') {
            buf = &buf[..buf.len() - 1];
        }
        super::parse::buf(buf)
    }
    pub fn from_str<T>(
        buf: &mut Vec<u8>,
        reader: &mut impl BufRead,
        separator: u8,
    ) -> Result<T, std::io::Error>
    where
        T: FromStr,
        <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    {
        buf.clear();
        reader.read_until(separator, buf)?;
        assert_eq!(buf.last(), Some(&separator));
        let buf = &buf[..buf.len() - 1];
        super::parse::buf(buf)
    }
    pub fn string(
        buf: &mut Vec<u8>,
        reader: &mut impl BufRead,
        separator: u8,
    ) -> Result<String, std::io::Error> {
        buf.clear();
        reader.read_until(separator, buf)?;
        assert_eq!(buf.last(), Some(&separator));
        let buf = &buf[..buf.len() - 1];
        Ok(String::from_utf8(buf.to_vec()).unwrap())
    }
}

pub mod parse {
    use std::{str, str::FromStr};

    pub fn buf<T>(buf: &[u8]) -> Result<T, std::io::Error>
    where
        T: FromStr,
        <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    {
        str::from_utf8(buf)
            .map_err(super::invalid_data)?
            .parse()
            .map_err(super::invalid_data)
    }

    pub fn numeric_id(s: &str, prefix: &str, expected: &str) -> Result<u64, std::io::Error> {
        let Some(i) = s.strip_prefix(prefix) else {
            return Err(crate::io::invalid_data(format!(
                "{expected}, but the value does not start with '{prefix}'. Value found: '{s}'.",
            )));
        };

        if !i.chars().next().as_ref().is_some_and(char::is_ascii_digit) {
            return Err(crate::io::invalid_data(format!(
                "{expected}, but found invalid value. Value found: '{s}'.",
            )));
        }
        let Ok(id) = i.parse() else {
            return Err(crate::io::invalid_data(format!(
                "{expected}, but the value is not an integer after '{prefix}'. Value found: '{s}'.",
            )));
        };
        Ok(id)
    }

    pub mod string_sequence {
        use std::{str, str::FromStr};

        pub fn str<T>(string: &str, separator: char) -> Result<Vec<T>, std::io::Error>
        where
            T: FromStr,
            <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
        {
            string
                .split(separator)
                .map(FromStr::from_str)
                .map(|v| v.map_err(crate::io::invalid_data))
                .collect()
        }
        pub fn buf<T>(buf: &[u8], separator: u8) -> Result<Vec<T>, std::io::Error>
        where
            T: FromStr,
            <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
        {
            buf.split(|&v| v == separator)
                .map(str::from_utf8)
                .map(|v| {
                    v.map_err(crate::io::invalid_data)?
                        .parse()
                        .map_err(crate::io::invalid_data)
                })
                .collect()
        }
    }
}

pub trait BufReadExt: BufRead {
    fn read_line_buf_no_delimter(&mut self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        self.read_line_buf(buf)?;
        buf.pop();
        if buf.last() == Some(&b'\r') {
            buf.pop();
        }
        Ok(())
    }
    fn read_line_buf(&mut self, buf: &mut Vec<u8>) -> Result<usize, std::io::Error> {
        self.read_until(b'\n', buf)
    }
}
impl<T: BufRead> BufReadExt for T {}

pub fn invalid_data(e: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e)
}
pub fn reqwest_error(e: reqwest::Error) -> std::io::Error {
    let kind = if e.is_timeout() {
        std::io::ErrorKind::TimedOut
    } else if e.is_connect() {
        std::io::ErrorKind::ConnectionRefused
    } else if e.is_request() {
        std::io::ErrorKind::InvalidInput
    } else if e.is_body() || e.is_decode() {
        std::io::ErrorKind::InvalidData
    } else {
        std::io::ErrorKind::Other
    };
    std::io::Error::new(kind, e)
}
pub fn ftp_error(e: suppaftp::FtpError) -> std::io::Error {
    let kind = match &e {
        suppaftp::FtpError::ConnectionError(error) => error.kind(),
        // suppaftp::FtpError::SecureError(_) => std::io::ErrorKind::Other,
        suppaftp::FtpError::UnexpectedResponse(_) => std::io::ErrorKind::InvalidData,
        suppaftp::FtpError::BadResponse => std::io::ErrorKind::InvalidData,
        suppaftp::FtpError::InvalidAddress(_) => std::io::ErrorKind::Other,
    };
    std::io::Error::new(kind, e)
}

pub fn get_filename_from_headers(headers: &HeaderMap) -> Option<String> {
    let header_value = headers.get(CONTENT_DISPOSITION)?;
    let mut content_disposition = ContentDisposition::parse_header(&header_value).ok()?;

    if content_disposition.disposition == DispositionType::Ext("attachement".to_owned()) {
        content_disposition.disposition = DispositionType::Attachment;
    }

    if content_disposition.disposition != DispositionType::Attachment {
        return None;
    }

    content_disposition.parameters.iter().find_map(|param| {
        if let DispositionParam::Filename(_, _, bytes) = param {
            String::from_utf8(bytes.clone()).ok()
        } else {
            None
        }
    })
}
pub fn get_filesize_from_headers(headers: &HeaderMap) -> Option<u64> {
    headers.get(CONTENT_LENGTH)?.to_str().ok()?.parse().ok()
}

pub trait FromUtf8Bytes: Sized {
    type Err: fmt::Debug; // Require Debug for convenience on bounds.

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err>;
}
impl FromUtf8Bytes for String {
    type Err = std::io::Error;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err> {
        String::from_utf8(bytes.to_vec())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "Failed to parse value. Raw bytes: {raw:?}. UTF8 value: {utf8:?}. Parse error: {parse_error:?}"
)]
pub struct FromAsciiBytesError<E> {
    raw: Vec<u8>,
    utf8: Result<String, std::string::FromUtf8Error>,
    parse_error: Option<E>,
}
impl<E: fmt::Debug> From<FromAsciiBytesError<E>> for std::io::Error {
    fn from(value: FromAsciiBytesError<E>) -> Self {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{value}"))
    }
}
macro_rules! from_bytes_ascii {
    ($t:ty) => {
        impl FromUtf8Bytes for $t {
            type Err = FromAsciiBytesError<<$t as FromStr>::Err>;

            fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err> {
                let Some(ascii) = bytes.as_ascii() else {
                    Err(FromAsciiBytesError {
                        raw: bytes.to_vec(),
                        utf8: String::from_utf8(bytes.to_vec()),
                        parse_error: None,
                    })?
                };
                ascii.as_str().parse().map_err(|e| FromAsciiBytesError {
                    raw: bytes.to_vec(),
                    utf8: String::from_utf8(bytes.to_vec()),
                    parse_error: Some(e),
                })
            }
        }
    };
}
from_bytes_ascii!(u8);
from_bytes_ascii!(u16);
from_bytes_ascii!(u32);
from_bytes_ascii!(u64);
from_bytes_ascii!(u128);
from_bytes_ascii!(i8);
from_bytes_ascii!(i16);
from_bytes_ascii!(i32);
from_bytes_ascii!(i64);
from_bytes_ascii!(i128);
from_bytes_ascii!(f32);
from_bytes_ascii!(f64);
