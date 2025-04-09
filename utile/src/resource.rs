#![allow(async_fn_in_trait)] // TODO

use std::{
    fmt::{self, Debug},
    io::Read,
    path::{Path, PathBuf},
    pin::{pin, Pin},
    sync::LazyLock,
    task::{Context, Poll},
};

use bytes::Bytes;
use directories::ProjectDirs;
use futures::{stream, Stream, TryStreamExt};
use indicatif::ProgressStyle;
use pin_project::pin_project;
use reqwest::IntoUrl;
use serde::de::DeserializeOwned;
use serde_json::StreamDeserializer;
use tokio::io::AsyncReadExt;
use url::Url;

use crate::io::{get_filesize_from_headers, read_ext::AsyncReadInto, reqwest_error};

type JsonStreamDeserializer<R, T> =
    StreamDeserializer<'static, serde_json::de::IoRead<std::io::BufReader<R>>, T>;

const PROGRESS_BAR_STYLE: &str =
    "{spinner} {bytes} ({percent}%) of {total_bytes} | {bytes_per_sec} {wide_bar} {eta}";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Compression {
    Gzip,
    MultiGzip,
    // TODO: nested
}

pub trait RawResource {
    const NAMESPACE: &'static str;
    fn key(&self) -> String;

    fn compression(&self) -> Option<Compression>;

    type Reader: std::io::Read;
    fn size(&self) -> std::io::Result<u64>;
    fn read(&self) -> std::io::Result<Self::Reader>;

    type AsyncReader: tokio::io::AsyncRead;
    async fn size_async(&self) -> std::io::Result<u64>;
    async fn read_async(&self) -> std::io::Result<Self::AsyncReader>;
}
pub trait RawResourceExt: RawResource + Sized {
    fn buffered(self) -> BufferedResource<Self> {
        BufferedResource::new(self)
    }

    fn with_fs_cache(self, cache: &FsCache) -> FsCacheResource<Self> {
        cache.entry(self)
    }
    fn with_global_fs_cache(self) -> FsCacheResource<Self> {
        FsCache::global().entry(self)
    }

    fn log_progress(self) -> ProgressResource<Self> {
        ProgressResource::new(self)
    }

    fn decompressed(self) -> DecompressedResource<Self> {
        DecompressedResource::new(self)
    }
    fn decompressed_with(self, compression: Compression) -> DecompressedResource<Self> {
        DecompressedResource::new_with(self, compression)
    }

    fn read_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut reader = ResourceRef::new(self).read()?;
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(data)
    }
    fn read_json<T: DeserializeOwned>(&self) -> std::io::Result<T> {
        serde_json::from_reader(ResourceRef::new(self).buffered().read()?)
            .map_err(std::io::Error::from)
    }
    fn read_json_lines<T: DeserializeOwned>(
        &self,
    ) -> std::io::Result<JsonStreamDeserializer<Self::Reader, T>> {
        let reader = ResourceRef::new(self).buffered().read()?;
        Ok(serde_json::Deserializer::from_reader(reader).into_iter())
    }

    async fn read_vec_async(&self) -> std::io::Result<Vec<u8>> {
        let mut reader = ResourceRef::new(self).read_async().await?;
        let mut data = Vec::new();
        pin!(reader).read_to_end(&mut data).await?;
        Ok(data)
    }
    async fn read_json_async<T: DeserializeOwned>(&self) -> std::io::Result<T> {
        let data = self.read_async().await?.read_into_vec().await?;
        serde_json::from_slice(&data).map_err(std::io::Error::from)
    }
    async fn read_json_lines_async<T: DeserializeOwned>(
        &self,
    ) -> std::io::Result<impl Stream<Item = std::io::Result<T>>> {
        Ok(stream::try_unfold((), |()| async move { todo!() }))
    }
}
impl<T: RawResource> RawResourceExt for T {}

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

    pub fn entry<R: RawResource>(&self, resource: R) -> FsCacheResource<R> {
        FsCacheResource::new(self, resource)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FsCacheResource<R> {
    entry: FsCacheEntry,
    resource: R,
}
impl<R> fmt::Display for FsCacheResource<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.entry)
    }
}
impl<R> FsCacheResource<R> {
    pub fn new(cache: &FsCache, resource: R) -> Self
    where
        R: RawResource,
    {
        Self {
            entry: FsCacheEntry::new(cache, cache.path.join(R::NAMESPACE).join(resource.key())),
            resource,
        }
    }

    pub fn exists(&self) -> std::io::Result<bool> {
        self.entry.exists()
    }
    pub async fn exists_async(&self) -> std::io::Result<bool> {
        self.entry.exists_async().await
    }

    pub fn ensure_cached(self) -> std::io::Result<Self>
    where
        R: RawResource,
    {
        self.cache()?;
        Ok(self)
    }
    pub async fn ensure_cached_async(self) -> std::io::Result<Self>
    where
        R: RawResource,
    {
        self.cache_async().await?;
        Ok(self)
    }

    pub fn cache(&self) -> std::io::Result<FsCacheEntry>
    where
        R: RawResource,
    {
        self.read()?;
        Ok(self.entry.clone())
    }
    pub async fn cache_async(&self) -> std::io::Result<FsCacheEntry>
    where
        R: RawResource,
    {
        self.read_async().await?;
        Ok(self.entry.clone())
    }
}
impl<R: RawResource> RawResource for FsCacheResource<R> {
    const NAMESPACE: &'static str = R::NAMESPACE;
    fn key(&self) -> String {
        R::key(&self.resource)
    }

    fn compression(&self) -> Option<Compression> {
        self.resource.compression()
    }

    type Reader = std::fs::File;
    fn size(&self) -> std::io::Result<u64> {
        if let Ok(size) = self.entry.size() {
            Ok(size)
        } else {
            self.resource.size()
        }
    }
    fn read(&self) -> std::io::Result<Self::Reader> {
        if self.exists()? {
            log::info!("Cache hit at {self}");
            return self.entry.read_file();
        }

        log::info!("Downloading {self} from {self}");

        self.entry
            .write_file(ResourceRef::new(&self.resource).buffered().read()?)?;

        log::info!("Downloaded {self}");

        self.entry.read_file()
    }

    type AsyncReader = tokio::fs::File;
    async fn size_async(&self) -> std::io::Result<u64> {
        if let Ok(size) = self.entry.size_async().await {
            Ok(size)
        } else {
            self.resource.size_async().await
        }
    }
    async fn read_async(&self) -> std::io::Result<Self::AsyncReader> {
        if self.exists_async().await? {
            log::info!("Cache hit at {self}");
            return self.entry.read_file_async().await;
        }

        log::info!("Downloading {self} from {self}");

        self.entry
            .set_file_async(
                ResourceRef::new(&self.resource)
                    .buffered()
                    .read_async()
                    .await?,
            )
            .await?;

        log::info!("Downloaded {self}");

        self.entry.read_file_async().await
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

    pub fn size(&self) -> std::io::Result<u64> {
        std::fs::metadata(self).map(|m| m.len())
    }
    pub async fn size_async(&self) -> std::io::Result<u64> {
        tokio::fs::metadata(self).await.map(|m| m.len())
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

    pub fn read_file(&self) -> std::io::Result<std::fs::File> {
        std::fs::File::open(self)
    }
    pub fn write_file(&self, mut data: impl std::io::BufRead) -> std::io::Result<()> {
        std::fs::create_dir_all(self.path.parent().unwrap())?;

        let mut tmp_file = tempfile::Builder::new().tempfile()?;
        std::io::copy(&mut data, &mut tmp_file)?;

        rename_or_copy(tmp_file, self)?;

        Ok(())
    }

    pub async fn read_file_async(&self) -> std::io::Result<tokio::fs::File> {
        tokio::fs::File::open(&self).await
    }
    pub async fn set_file_async(&self, data: impl tokio::io::AsyncBufRead) -> std::io::Result<()> {
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
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecompressedResource<R> {
    resource: R,
    compression: Option<Compression>,
}
impl<R: RawResource> DecompressedResource<R> {
    pub fn new(resource: R) -> Self {
        Self {
            resource,
            compression: None,
        }
    }
    pub fn new_with(resource: R, compression: Compression) -> Self {
        Self {
            resource,
            compression: Some(compression),
        }
    }
}
impl<R: RawResource> RawResource for DecompressedResource<R> {
    const NAMESPACE: &'static str = "decompressed";
    fn key(&self) -> String {
        format!("{}/{}", R::NAMESPACE, self.resource.key())
    }
    fn compression(&self) -> Option<Compression> {
        None
    }

    type Reader = DecompressedReader<R::Reader>;
    fn size(&self) -> std::io::Result<u64> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Decompressed resources do not have a size.",
        ))
    }
    fn read(&self) -> std::io::Result<Self::Reader> {
        match self.resource.compression() {
            None => Ok(DecompressedReader::None(self.resource.read()?)),
            Some(Compression::Gzip) => {
                Ok(DecompressedReader::Gzip(flate2::bufread::GzDecoder::new(
                    ResourceRef::new(&self.resource).buffered().read()?,
                )))
            }
            Some(Compression::MultiGzip) => Ok(DecompressedReader::MultiGzip(
                flate2::bufread::MultiGzDecoder::new(
                    ResourceRef::new(&self.resource).buffered().read()?,
                ),
            )),
        }
    }

    type AsyncReader = AsyncDecompressedReader<R::AsyncReader>;
    async fn size_async(&self) -> std::io::Result<u64> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Decompressed resources do not have a size.",
        ))
    }
    async fn read_async(&self) -> std::io::Result<Self::AsyncReader> {
        match self.resource.compression() {
            None => Ok(AsyncDecompressedReader::None(
                self.resource.read_async().await?,
            )),
            Some(Compression::Gzip) => Ok(AsyncDecompressedReader::Gzip(
                async_compression::tokio::bufread::GzipDecoder::new(tokio::io::BufReader::new(
                    self.resource.read_async().await?,
                )),
            )),
            Some(Compression::MultiGzip) => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferedResource<R> {
    resource: R,
}
impl<R: RawResource> BufferedResource<R> {
    pub fn new(resource: R) -> Self {
        Self { resource }
    }
}
impl<R: RawResource> RawResource for BufferedResource<R> {
    const NAMESPACE: &'static str = R::NAMESPACE;
    fn key(&self) -> String {
        R::key(&self.resource)
    }
    fn compression(&self) -> Option<Compression> {
        self.resource.compression()
    }

    type Reader = std::io::BufReader<R::Reader>;
    fn size(&self) -> std::io::Result<u64> {
        self.resource.size()
    }
    fn read(&self) -> std::io::Result<Self::Reader> {
        Ok(std::io::BufReader::new(self.resource.read()?))
    }

    type AsyncReader = tokio::io::BufReader<R::AsyncReader>;
    async fn size_async(&self) -> std::io::Result<u64> {
        self.resource.size_async().await
    }
    async fn read_async(&self) -> std::io::Result<Self::AsyncReader> {
        Ok(tokio::io::BufReader::new(self.resource.read_async().await?))
    }
}

#[derive(Debug)]
pub enum DecompressedReader<R> {
    None(R),
    Gzip(flate2::bufread::GzDecoder<std::io::BufReader<R>>),
    // GzipTrailingGarbage(flate2::bufread::GzDecoder<std::io::BufReader<R>>),
    MultiGzip(flate2::bufread::MultiGzDecoder<std::io::BufReader<R>>),
}
impl<R: std::io::Read> std::io::Read for DecompressedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::None(r) => r.read(buf),
            Self::Gzip(r) => r.read(buf),
            Self::MultiGzip(r) => r.read(buf),
        }
    }
}

#[derive(Debug)]
#[pin_project(project = AsyncDecompressedReaderProj)]
pub enum AsyncDecompressedReader<R> {
    None(#[pin] R),
    Gzip(#[pin] async_compression::tokio::bufread::GzipDecoder<tokio::io::BufReader<R>>),
    // // TODO: async multi-gz?
    // MultiGzip(#[pin] async_compression::tokio::bufread::GzipDecoder<tokio::io::BufReader<R>>),
}
impl<R: tokio::io::AsyncRead> tokio::io::AsyncRead for AsyncDecompressedReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.project() {
            AsyncDecompressedReaderProj::None(reader) => reader.poll_read(cx, buf),
            AsyncDecompressedReaderProj::Gzip(decoder) => decoder.poll_read(cx, buf),
            // AsyncDecompressedReaderProj::MultiGzip(decoder) => decoder.poll_read(cx, buf),
        }
    }
}

impl Compression {
    pub fn infer(filename: &str) -> Option<Self> {
        if filename.ends_with(".gz") || filename.ends_with(".bgz") {
            // We default to multi-gzip because it doesn't fail silently.
            Some(Self::MultiGzip)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
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

    type Reader = reqwest::blocking::Response;
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
    fn read(&self) -> std::io::Result<Self::Reader> {
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProgressResource<R> {
    resource: R,
}
impl<R: RawResource> ProgressResource<R> {
    pub fn new(resource: R) -> Self {
        Self { resource }
    }
}
impl<R: RawResource> RawResource for ProgressResource<R> {
    const NAMESPACE: &'static str = R::NAMESPACE;
    fn key(&self) -> String {
        R::key(&self.resource)
    }

    fn compression(&self) -> Option<Compression> {
        self.resource.compression()
    }

    type Reader = indicatif::ProgressBarIter<R::Reader>;
    fn size(&self) -> std::io::Result<u64> {
        self.resource.size()
    }
    fn read(&self) -> std::io::Result<Self::Reader> {
        let style = ProgressStyle::with_template(PROGRESS_BAR_STYLE).unwrap();
        let reader = self.resource.read()?;
        Ok(match self.size() {
            Ok(size) => indicatif::ProgressBar::new(size),
            Err(_) => indicatif::ProgressBar::no_length(),
        }
        .with_style(style)
        .wrap_read(reader))
    }

    type AsyncReader = indicatif::ProgressBarIter<Pin<Box<R::AsyncReader>>>;
    async fn size_async(&self) -> std::io::Result<u64> {
        self.resource.size_async().await
    }
    async fn read_async(&self) -> std::io::Result<Self::AsyncReader> {
        let style = ProgressStyle::with_template(PROGRESS_BAR_STYLE).unwrap();
        let reader = self.resource.read_async().await?;
        Ok(match self.size_async().await {
            Ok(size) => indicatif::ProgressBar::new(size),
            Err(_) => indicatif::ProgressBar::no_length(),
        }
        .with_style(style)
        .wrap_async_read(Box::pin(reader)))
    }
}

fn rename_or_copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> std::io::Result<()> {
    match std::fs::rename(from.as_ref(), to.as_ref()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
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
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
            tokio::fs::copy(from.as_ref(), to.as_ref()).await?;
            tokio::fs::remove_file(from.as_ref()).await?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Just a helper struct to avoid a blanket `impl RawResource for &R`
/// or requiring a `Clone` bound in some places.
/// (The blanket impl would allow the builder api to take a reference
/// which in practice can cause annoying lifetime issues.)
struct ResourceRef<'a, R> {
    resource: &'a R,
}
impl<'a, R: RawResource> ResourceRef<'a, R> {
    pub fn new(resource: &'a R) -> Self {
        Self { resource }
    }
}
impl<'a, R: RawResource> RawResource for ResourceRef<'a, R> {
    const NAMESPACE: &'static str = R::NAMESPACE;
    fn key(&self) -> String {
        R::key(self.resource)
    }
    fn compression(&self) -> Option<Compression> {
        R::compression(self.resource)
    }

    type Reader = R::Reader;
    fn size(&self) -> std::io::Result<u64> {
        R::size(self.resource)
    }
    fn read(&self) -> std::io::Result<Self::Reader> {
        R::read(self.resource)
    }

    type AsyncReader = R::AsyncReader;
    async fn size_async(&self) -> std::io::Result<u64> {
        R::size_async(self.resource).await
    }
    async fn read_async(&self) -> std::io::Result<Self::AsyncReader> {
        R::read_async(self.resource).await
    }
}
