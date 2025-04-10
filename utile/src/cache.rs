use std::{
    fmt, io,
    path::{Path, PathBuf},
    pin::pin,
    sync::LazyLock,
};

use directories::ProjectDirs;
use reqwest::IntoUrl;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use url::Url;

use crate::resource::RawResource;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FsCache {
    path: PathBuf,
}
impl FsCache {
    pub fn global() -> Self {
        static PROJECT_DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
            let cache = directories::ProjectDirs::from("", "bio_data", "bio_data").unwrap();
            log::info!("Using global cache at {}", cache.cache_dir().display());
            cache
        });

        Self::new(PROJECT_DIRS.cache_dir())
    }
    pub fn new(path: impl AsRef<Path>) -> Self {
        assert!(path.as_ref().is_absolute());
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
    pub fn entry(&self, key: impl AsRef<Path>) -> FsCacheEntry {
        FsCacheEntry::new(self, key)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FsCacheEntry {
    path: PathBuf,
}
impl AsRef<Path> for FsCacheEntry {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
impl fmt::Display for FsCacheEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}
impl FsCacheEntry {
    pub fn new(cache: &FsCache, key: impl AsRef<Path>) -> Self {
        assert!(cache.path.is_absolute());
        let path = cache.path.join(key.as_ref());
        assert!(path.is_absolute());
        Self { path }
    }

    pub fn exists(&self) -> std::io::Result<bool> {
        match std::fs::File::open(self) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }
    pub async fn exists_async(&self) -> std::io::Result<bool> {
        match tokio::fs::File::open(&self).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn write_file(&self, mut data: impl std::io::BufRead) -> std::io::Result<()> {
        std::fs::create_dir_all(self.path.parent().unwrap())?;

        let mut tmp_file = tempfile::Builder::new().tempfile()?;
        std::io::copy(&mut data, &mut tmp_file)?;

        rename_or_copy(tmp_file, self)?;

        Ok(())
    }
    pub async fn write_file_async(
        &self,
        data: impl tokio::io::AsyncBufRead,
    ) -> std::io::Result<()> {
        tokio::fs::create_dir_all(self.path.parent().unwrap()).await?;

        let tmp_file = tempfile::Builder::new().tempfile()?;
        tokio::io::copy(
            &mut pin!(data),
            &mut tokio::fs::File::create(tmp_file.path()).await?,
        )
        .await?;

        rename_or_copy_async(tmp_file, &self).await?;

        Ok(())
    }

    pub fn write_json<T: serde::Serialize>(&self, data: T) -> std::io::Result<()> {
        self.write_file(serde_json::to_string(&data)?.as_bytes())
    }
    pub fn write_json_lines<T: serde::Serialize>(
        &self,
        data: impl IntoIterator<Item = T>,
    ) -> std::io::Result<()> {
        self.write_file(io::BufReader::new(crate::jsonl::JsonLinesReader::new(
            data.into_iter(),
        )))
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
impl RawResource for FsCacheEntry {
    const NAMESPACE: &'static str = "fs_cache";
    fn key(&self) -> String {
        self.path.to_string_lossy().as_ref().to_owned()
    }

    fn compression(&self) -> Option<crate::resource::Compression> {
        None
    }

    type Reader = std::fs::File;
    fn size(&self) -> std::io::Result<u64> {
        std::fs::metadata(self).map(|m| m.len())
    }
    fn read(&self) -> std::io::Result<Self::Reader> {
        std::fs::File::open(self)
    }

    type AsyncReader = tokio::fs::File;
    async fn size_async(&self) -> std::io::Result<u64> {
        tokio::fs::metadata(self).await.map(|m| m.len())
    }
    async fn read_async(&self) -> std::io::Result<Self::AsyncReader> {
        tokio::fs::File::open(self).await
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
