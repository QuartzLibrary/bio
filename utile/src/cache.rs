use std::{
    fmt,
    io::{self, Cursor, Read},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use directories::ProjectDirs;
use futures::TryStreamExt;
use reqwest::IntoUrl;
use tokio::io::AsyncReadExt;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use url::Url;

static PROJECT_DIRS: LazyLock<ProjectDirs> =
    LazyLock::new(|| directories::ProjectDirs::from("", "bio_data", "bio_data").unwrap());

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cache {
    root: PathBuf,
    prefix: PathBuf,
}
impl Cache {
    pub fn global(prefix: impl AsRef<Path>) -> Self {
        Cache {
            root: PROJECT_DIRS.cache_dir().to_path_buf(),
            prefix: prefix.as_ref().to_path_buf(),
        }
    }
    pub fn new(root: impl AsRef<Path>, prefix: impl AsRef<Path>) -> Self {
        Cache {
            root: root.as_ref().to_path_buf(),
            prefix: prefix.as_ref().to_path_buf(),
        }
    }

    // TODO: Check paths to avoid paths that exit the cache.
    pub fn entry(&self, key: impl AsRef<Path>) -> CacheEntry {
        let key = key.as_ref();
        assert!(key.is_relative(), "Path is not relative: {key:?}");
        CacheEntry::new(self.root.clone(), self.prefix.clone(), key.to_path_buf())
    }
    pub async fn list_all(&self) -> Result<Vec<CacheEntry>, io::Error> {
        let base = self.dir();
        let mut raw_entries = tokio::fs::read_dir(&base).await?;
        let mut entries = vec![];
        for _ in 0.. {
            let Some(entry) = raw_entries.next_entry().await? else {
                break;
            };
            entries.push(self.entry(entry.path().strip_prefix(&base).unwrap()));
        }
        Ok(entries)
    }

    fn dir(&self) -> PathBuf {
        self.root.join(&self.prefix)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    root: PathBuf,
    prefix: PathBuf,
    key: PathBuf,
    full: PathBuf,
}
impl fmt::Display for CacheEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path().display())
    }
}
impl CacheEntry {
    fn new(root: PathBuf, prefix: PathBuf, key: PathBuf) -> Self {
        let full = root.join(&prefix).join(&key);
        Self {
            root,
            prefix,
            key,
            full,
        }
    }
    pub fn path(&self) -> &Path {
        &self.full
    }

    pub fn exists(&self) -> std::io::Result<bool> {
        match std::fs::File::open(self) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }
    pub async fn exists_async(&self) -> std::io::Result<bool> {
        match tokio::fs::File::open(&self).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn get(&self) -> std::io::Result<impl std::io::Read + std::io::Seek> {
        std::fs::File::open(self)
    }
    pub fn get_as_string(&self) -> std::io::Result<String> {
        let mut reader = self.get()?;
        let mut data = String::new();
        reader.read_to_string(&mut data)?;
        Ok(data)
    }
    pub fn get_gz(&self) -> std::io::Result<impl std::io::Read> {
        struct AssertingGzDecoder<R> {
            inner: flate2::bufread::GzDecoder<R>,
        }

        impl<R: std::io::BufRead> std::io::Read for AssertingGzDecoder<R> {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                let read = self.inner.read(buf)?;
                if read == 0 && !buf.is_empty() {
                    assert_ne!(read, 0);
                }
                Ok(0)
            }
        }

        Ok(AssertingGzDecoder {
            inner: flate2::bufread::GzDecoder::new(std::io::BufReader::new(self.get()?)),
        })
    }
    pub fn get_gz_multi(&self) -> std::io::Result<impl std::io::Read> {
        Ok(flate2::bufread::MultiGzDecoder::new(
            std::io::BufReader::new(self.get()?),
        ))
    }
    pub fn get_as_json<T: serde::de::DeserializeOwned>(&self) -> std::io::Result<T> {
        Ok(serde_json::from_reader(self.get()?)?)
    }
    pub fn get_as_json_lines<T: serde::de::DeserializeOwned>(&self) -> std::io::Result<Vec<T>> {
        let file = self.get()?;
        let reader = std::io::BufReader::new(file);
        Ok(serde_json::Deserializer::from_reader(reader)
            .into_iter()
            .try_collect()?)
    }
    pub async fn get_async(
        &self,
    ) -> std::io::Result<impl tokio::io::AsyncRead + tokio::io::AsyncSeek> {
        tokio::fs::File::open(&self).await
    }
    pub async fn get_as_string_async(&self) -> std::io::Result<String> {
        let mut reader = self.get_async().await?;
        let mut data = String::new();
        reader.read_to_string(&mut data).await?;
        Ok(data)
    }
    // pub async fn get_as_json_async<T: serde::de::DeserializeOwned>(&self) -> std::io::Result<T> {
    //    todo!()
    // }
    // pub async fn get_as_json_lines_async<T: serde::de::DeserializeOwned>(&self) -> std::io::Result<Vec<T>> {
    //     todo!()
    // }

    pub fn set(&self, data: impl std::io::Read) -> std::io::Result<()> {
        let mut data = std::io::BufReader::new(data);

        let mut tmp_file = tempfile::Builder::new().tempfile()?;
        std::io::copy(&mut data, &mut tmp_file)?;

        std::fs::create_dir_all(self.path().parent().unwrap())?;
        rename_or_copy(tmp_file, self)?;

        Ok(())
    }
    pub async fn set_async(&self, data: impl tokio::io::AsyncRead) -> std::io::Result<()> {
        let data = tokio::io::BufReader::new(data);

        let tmp_file = tempfile::Builder::new().tempfile().unwrap();
        tokio::io::copy(
            &mut std::pin::pin!(data),
            &mut tokio::fs::File::create(tmp_file.path()).await.unwrap(),
        )
        .await?;

        tokio::fs::create_dir_all(self.path().parent().unwrap())
            .await
            .unwrap();
        rename_or_copy_async(tmp_file, &self).await.unwrap();
        Ok(())
    }
    pub fn set_json<T: serde::Serialize>(&self, data: T) -> std::io::Result<()> {
        self.set(serde_json::to_string(&data)?.as_bytes())
    }
    pub fn set_json_lines<T: serde::Serialize>(
        &self,
        data: impl IntoIterator<Item = T>,
    ) -> std::io::Result<()> {
        self.set(crate::jsonl::JsonLinesReader::new(data.into_iter()))
    }

    /// Unfortunately some sources aren't pure.
    pub fn invalidate(&self) -> std::io::Result<()> {
        std::fs::remove_file(self)
    }
    /// Unfortunately some sources aren't pure.
    pub async fn invalidate_async(&self) -> std::io::Result<()> {
        tokio::fs::remove_file(&self).await
    }
}
impl AsRef<Path> for CacheEntry {
    fn as_ref(&self) -> &Path {
        &self.full
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlEntry(pub Url);
impl fmt::Display for UrlEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl UrlEntry {
    pub fn new(url: impl IntoUrl) -> reqwest::Result<Self> {
        let mut url = url.into_url()?;

        // Convert s3:// URLs to https URLs for AWS S3
        if url.scheme() == "s3"
            && let Some(bucket) = url.host_str()
        {
            let path = url.path();
            let new = Url::parse(&format!("https://{bucket}.s3.amazonaws.com{path}")).unwrap();
            log::info!("Converted {url} to {new}.");
            url = new;
        }

        Ok(Self(url))
    }

    pub fn exists(&self) -> reqwest::Result<bool> {
        let response = reqwest::blocking::Client::new()
            .head(self.0.clone())
            .send()?;
        Ok(response.status() == reqwest::StatusCode::OK)
    }
    pub async fn exists_async(&self) -> reqwest::Result<bool> {
        let response = reqwest::Client::new().head(self.0.clone()).send().await?;
        Ok(response.status() == reqwest::StatusCode::OK)
    }

    pub fn get(&self) -> reqwest::Result<impl std::io::Read> {
        let response = reqwest::blocking::Client::new()
            .get(self.0.clone())
            .send()?
            .error_for_status()?;
        assert!(response.status().is_success());
        Ok(Cursor::new(response.bytes()?))
    }
    pub async fn get_async(&self) -> reqwest::Result<impl tokio::io::AsyncRead> {
        let response = reqwest::Client::new()
            .get(self.0.clone())
            .send()
            .await?
            .error_for_status()?;

        let stream = response.bytes_stream().map_err(std::io::Error::other);
        Ok(tokio_util::io::StreamReader::new(stream))
    }
    pub async fn get_retry_async(
        &self,
        retries: u64,
    ) -> reqwest::Result<impl tokio::io::AsyncRead> {
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
}
impl UrlEntry {
    pub fn get_and_cache(&self, log_prefix: &str, fs_entry: CacheEntry) -> std::io::Result<()> {
        if fs_entry.exists()? {
            log::info!("{log_prefix} Cache hit at {fs_entry}");
            return Ok(());
        }

        log::info!("{log_prefix} Downloading {fs_entry} from {self}");

        fs_entry.set(self.get().map_err(crate::io::reqwest_error)?)?;

        log::info!("{log_prefix} Downloaded {fs_entry}");

        Ok(())
    }
    pub async fn get_and_cache_async(
        &self,
        log_prefix: &str,
        fs_entry: CacheEntry,
    ) -> std::io::Result<()> {
        if fs_entry.exists_async().await? {
            log::info!("{log_prefix} Cache hit at {fs_entry}");
            return Ok(());
        }

        log::info!("{log_prefix} Downloading {fs_entry} from {self}");

        fs_entry
            .set_async(
                self.get_retry_async(5)
                    .await
                    .map_err(crate::io::reqwest_error)?,
            )
            .await?;

        log::info!("{log_prefix} Downloaded {fs_entry}");

        Ok(())
    }
}

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
impl FtpEntry {
    pub fn get_and_cache(&self, log_prefix: &str, fs_entry: CacheEntry) -> std::io::Result<()> {
        if fs_entry.exists()? {
            log::info!("{log_prefix} Cache hit at {fs_entry}");
            return Ok(());
        }

        log::info!("{log_prefix} Downloading {fs_entry} from {self}");

        fs_entry.set(self.get().map_err(crate::io::ftp_error)?)?;

        log::info!("{log_prefix} Downloaded {fs_entry}");

        Ok(())
    }
    pub async fn get_and_cache_async(
        &self,
        log_prefix: &str,
        fs_entry: CacheEntry,
    ) -> std::io::Result<()> {
        if fs_entry.exists_async().await? {
            log::info!("{log_prefix} Cache hit at {fs_entry}");
            return Ok(());
        }

        log::info!("{log_prefix} Downloading {fs_entry} from {self}");

        fs_entry
            .set_async(
                self.get_retry_async(5)
                    .await
                    .map_err(crate::io::ftp_error)?,
            )
            .await?;

        log::info!("{log_prefix} Downloaded {fs_entry}");

        Ok(())
    }
}

// Add these new helper functions
fn rename_or_copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> std::io::Result<()> {
    match std::fs::rename(from.as_ref(), to.as_ref()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
            std::fs::copy(from.as_ref(), to.as_ref())?;
            std::fs::remove_file(from.as_ref())?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

async fn rename_or_copy_async(from: impl AsRef<Path>, to: impl AsRef<Path>) -> std::io::Result<()> {
    match tokio::fs::rename(from.as_ref(), to.as_ref()).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
            tokio::fs::copy(from.as_ref(), to.as_ref()).await?;
            tokio::fs::remove_file(from.as_ref()).await?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}
