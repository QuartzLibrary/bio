#![allow(async_fn_in_trait)] // TODO

pub mod buffered;
pub mod cached;
pub mod compression;
pub mod progress;
pub mod uri;

use std::{fmt::Debug, io::Read, pin::pin};

use futures::{stream, Stream};
use serde::de::DeserializeOwned;
use serde_json::StreamDeserializer;
use tokio::io::AsyncReadExt;

use crate::{cache::FsCache, io::read_ext::AsyncReadInto};

pub use self::{
    buffered::BufferedResource,
    cached::FsCacheResource,
    compression::{CompressedResource, DecompressedResource},
    progress::ProgressResource,
    uri::UrlResource,
};

type JsonStreamDeserializer<R, T> =
    StreamDeserializer<'static, serde_json::de::IoRead<std::io::BufReader<R>>, T>;

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
        FsCacheResource::new(cache, self)
    }
    fn with_global_fs_cache(self) -> FsCacheResource<Self> {
        FsCacheResource::new(&FsCache::global(), self)
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

    fn compressed(self) -> CompressedResource<Self> {
        CompressedResource::new(self, Compression::Gzip)
    }
    fn compressed_with(self, compression: Compression) -> CompressedResource<Self> {
        CompressedResource::new(self, compression)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Compression {
    Gzip,
    MultiGzip,
    // TODO: nested
}
impl Compression {
    pub fn infer_strict(filename: &str) -> Option<Self> {
        if filename.ends_with(".gz") {
            Some(Self::Gzip)
        } else if filename.ends_with(".bgz") {
            Some(Self::MultiGzip)
        } else {
            None
        }
    }
    pub fn infer(filename: &str) -> Option<Self> {
        if filename.ends_with(".gz") || filename.ends_with(".bgz") {
            // We default to multi-gzip because it doesn't fail silently.
            Some(Self::MultiGzip)
        } else {
            None
        }
    }
    pub fn extension(self) -> &'static str {
        match self {
            Self::Gzip => "gz",
            Self::MultiGzip => "bgz",
        }
    }
    pub fn trim_extension(self, filename: &str) -> &str {
        match self {
            Compression::Gzip => filename.strip_suffix(".gz").unwrap_or(filename),
            Compression::MultiGzip => filename
                .strip_suffix(".bgz")
                .unwrap_or_else(|| filename.strip_suffix(".gz").unwrap_or(filename)),
        }
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
