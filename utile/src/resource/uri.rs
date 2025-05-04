use std::{fmt, sync::LazyLock};

use bytes::Bytes;
use futures::{Stream, TryStreamExt};
use reqwest::IntoUrl;
use url::Url;

use crate::io::{get_filesize_from_headers, reqwest_error};

use super::{Compression, RawResource};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UrlResource(Url);
impl fmt::Display for UrlResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl UrlResource {
    pub fn new(url: impl IntoUrl) -> std::io::Result<Self> {
        let mut url = url.into_url().map_err(reqwest_error)?;

        // Convert s3:// URLs to https URLs for AWS S3
        if url.scheme() == "s3"
            && let Some(bucket) = url.host_str()
        {
            let path = url.path();
            let new = Url::parse(&format!("https://{bucket}.s3.amazonaws.com{path}")).unwrap();
            log::info!("Converted {url} to {new}.");
            url = new;
        }

        // Seems like this doesn't work, requires login.
        // if url.scheme() == "gs"
        //     && let Some(bucket) = url.host_str()
        // {
        //     let path = url.path();
        //     let new =
        //         Url::parse(&format!("https://storage.cloud.google.com/{bucket}{path}")).unwrap();
        //     log::info!("Converted {url} to {new}.");
        //     url = new;
        // }

        Ok(Self(url))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn exists(&self) -> reqwest::Result<bool> {
        let response = reqwest::blocking::Client::new()
            .head(self.0.clone())
            .send()?;
        Ok(response.status() == reqwest::StatusCode::OK)
    }
    #[cfg(target_arch = "wasm32")]
    pub fn exists(&self) -> reqwest::Result<bool> {
        panic!("UrlResource::exists is not supported on wasm32, use the non-blocking version.");
    }

    pub async fn exists_async(&self) -> reqwest::Result<bool> {
        let response = reqwest::Client::new().head(self.0.clone()).send().await?;
        Ok(response.status() == reqwest::StatusCode::OK)
    }

    pub async fn read_retry_async(
        &self,
        retries: u64,
    ) -> std::io::Result<impl tokio::io::AsyncRead> {
        for i in 0.. {
            match self.read_async().await {
                Ok(ok) => return Ok(ok),
                Err(e) if i == retries => return Err(e),
                Err(_) => {
                    let delay = 1000 * (1 << i); // 1s, 2s, 4s, 8s, 16s, ...
                    crate::time::sleep(std::time::Duration::from_millis(delay)).await;
                }
            }
        }
        unreachable!()
    }
}
impl RawResource for UrlResource {
    const NAMESPACE: &'static str = "url";
    fn key(&self) -> String {
        self.0.to_string()
    }

    fn compression(&self) -> Option<Compression> {
        None
    }

    #[cfg(not(target_arch = "wasm32"))]
    type Reader = reqwest::blocking::Response;
    #[cfg(not(target_arch = "wasm32"))]
    fn size(&self) -> std::io::Result<u64> {
        let response = reqwest::blocking::Client::new()
            .head(self.0.clone())
            .send()
            .map_err(reqwest_error)?
            .error_for_status()
            .map_err(reqwest_error)?;
        match get_filesize_from_headers(response.headers()) {
            Some(size) => Ok(size),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Content length not found.",
            )),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn read(&self) -> std::io::Result<Self::Reader> {
        log::info!("Downloading {self}");
        static CLIENT: LazyLock<reqwest::blocking::Client> =
            LazyLock::new(reqwest::blocking::Client::new);
        let response = CLIENT
            .get(self.0.clone())
            .send()
            .map_err(reqwest_error)?
            .error_for_status()
            .map_err(reqwest_error)?;

        let status = response.status();
        if !status.is_success() {
            return Err(std::io::Error::other(format!(
                "Request unsuccessful, failed with status code: {status}."
            )));
        }

        if let Some(content_length) = response.content_length() {
            indicatif::ProgressBar::new(content_length);
        }

        Ok(response)
    }
    #[cfg(target_arch = "wasm32")]
    type Reader = std::io::Cursor<&'static [u8]>;
    #[cfg(target_arch = "wasm32")]
    fn size(&self) -> std::io::Result<u64> {
        panic!("UrlResource::size is not supported on wasm32, use the non-blocking version.");
    }
    #[cfg(target_arch = "wasm32")]
    fn read(&self) -> std::io::Result<Self::Reader> {
        panic!("UrlResource::read is not supported on wasm32, use the non-blocking version.");
    }

    type AsyncReader =
        tokio_util::io::StreamReader<impl Stream<Item = std::io::Result<Bytes>>, Bytes>;
    async fn size_async(&self) -> std::io::Result<u64> {
        static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);
        let response = CLIENT
            .head(self.0.clone())
            .send()
            .await
            .map_err(reqwest_error)?
            .error_for_status()
            .map_err(reqwest_error)?;
        match get_filesize_from_headers(response.headers()) {
            Some(size) => Ok(size),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Content length not found.",
            )),
        }
    }
    async fn read_async(&self) -> std::io::Result<Self::AsyncReader> {
        let response = reqwest::Client::new()
            .get(self.0.clone())
            .send()
            .await
            .map_err(reqwest_error)?
            .error_for_status()
            .map_err(reqwest_error)?;

        let status = response.status();
        if !status.is_success() {
            return Err(std::io::Error::other(format!(
                "Request unsuccessful, failed with status code: {status}."
            )));
        }

        let stream = response.bytes_stream().map_err(std::io::Error::other);
        Ok(tokio_util::io::StreamReader::new(stream))
    }
}

// TODO: integrate into main resource.
#[cfg(not(target_arch = "wasm32"))]
pub mod ftp {
    use std::fmt;

    use reqwest::IntoUrl;
    use tokio_util::compat::FuturesAsyncReadCompatExt;
    use url::Url;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FtpEntry(pub Url);
    impl fmt::Display for FtpEntry {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl FtpEntry {
        pub fn new(url: impl IntoUrl) -> reqwest::Result<Self> {
            Ok(Self(url.into_url()?))
        }

        fn connect(&self) -> suppaftp::FtpResult<suppaftp::FtpStream> {
            let Some(host) = self.0.host_str() else {
                return Err(suppaftp::FtpError::ConnectionError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid FTP host",
                )));
            };
            let port = self.0.port().unwrap_or(21);

            let mut ftp = suppaftp::FtpStream::connect(format!("{host}:{port}"))?;
            ftp.login("anonymous", "anonymous")?;

            Ok(ftp)
        }
        async fn connect_async(&self) -> suppaftp::FtpResult<suppaftp::AsyncFtpStream> {
            let Some(host) = self.0.host_str() else {
                return Err(suppaftp::FtpError::ConnectionError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid FTP host",
                )));
            };
            let port = self.0.port().unwrap_or(21);

            let mut ftp = suppaftp::AsyncFtpStream::connect(format!("{host}:{port}")).await?;
            ftp.login("anonymous", "anonymous").await?;

            Ok(ftp)
        }

        pub fn exists(&self) -> suppaftp::FtpResult<bool> {
            let mut ftp = self.connect()?;
            let path = self.0.path();
            Ok(ftp.size(path).is_ok())
        }
        pub async fn exists_async(&self) -> suppaftp::FtpResult<bool> {
            let mut ftp = self.connect_async().await?;
            let path = self.0.path();
            Ok(ftp.size(path).await.is_ok())
        }

        pub fn get(&self) -> suppaftp::FtpResult<impl std::io::Read> {
            let mut ftp = self.connect()?;

            let path = self.0.path();
            let reader = ftp.retr_as_stream(path)?;

            Ok(reader)
        }
        pub async fn get_async(&self) -> suppaftp::FtpResult<impl tokio::io::AsyncRead> {
            let mut ftp = self.connect_async().await?;
            let path = self.0.path();
            // TODO: we should clean-up the connection after the stream is done.
            // Though it might be fine if we use a different connection for every file for now?
            Ok(ftp.retr_as_stream(path).await?.compat(/* tokio compat */))
        }
        pub async fn get_retry_async(
            &self,
            retries: u64,
        ) -> suppaftp::FtpResult<impl tokio::io::AsyncRead> {
            for i in 0.. {
                match self.get_async().await {
                    Ok(ok) => return Ok(ok),
                    Err(e) if i == retries => return Err(e),
                    Err(_) => {
                        let delay = 1000 * (1 << i); // 1s, 2s, 4s, 8s, 16s, ...
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    }
                }
            }
            unreachable!()
        }

        pub async fn list_all(&self) -> suppaftp::FtpResult<Vec<String>> {
            let mut ftp = self.connect_async().await?;
            ftp.list(Some(self.0.path())).await
        }
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
}
