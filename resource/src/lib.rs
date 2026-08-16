#![expect(async_fn_in_trait)] // TODO

pub mod buffered;
pub mod cache;
pub mod cached;
pub mod compression;
pub mod iter;
pub mod progress;
pub mod uri;

use std::{
    fmt::Debug,
    io::{self, Read, Write as _},
    pin::{Pin, pin},
};

use futures::{Stream, StreamExt as _, stream};
use serde::de::DeserializeOwned;
use serde_json::StreamDeserializer;
use tokio::io::{AsyncReadExt, AsyncWriteExt as _};

use utile::io::read_ext::AsyncReadInto;

pub use self::{
    buffered::BufferedResource,
    compression::{CompressedResource, DecompressedResource},
    progress::ProgressResource,
    uri::UrlResource,
};

pub use self::cached::FsCacheResource;

type JsonStreamDeserializer<R, T> =
    StreamDeserializer<'static, serde_json::de::IoRead<io::BufReader<R>>, T>;

pub trait Resource {
    const NAMESPACE: &'static str;
    fn key(&self) -> String;

    fn compression(&self) -> Option<Compression>;
}
pub trait ReadResource: Resource {
    type Reader: io::Read;
    fn size(&self) -> io::Result<u64>;
    fn read(&self) -> io::Result<Self::Reader>;

    type AsyncReader: tokio::io::AsyncRead;
    async fn size_async(&self) -> io::Result<u64>;
    async fn read_async(&self) -> io::Result<Self::AsyncReader>;

    fn read_vec(&self) -> io::Result<Vec<u8>> {
        let mut reader = ResourceRef::new(self).read()?;
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(data)
    }
    fn read_string(&self) -> io::Result<String> {
        let mut reader = ResourceRef::new(self).read()?;
        let mut data = String::new();
        reader.read_to_string(&mut data)?;
        Ok(data)
    }
    fn read_json<T: DeserializeOwned>(&self) -> io::Result<T> {
        serde_json::from_reader(ResourceRef::new(self).buffered().read()?).map_err(io::Error::from)
    }
    fn read_json_lines<T: DeserializeOwned>(
        &self,
    ) -> io::Result<JsonStreamDeserializer<Self::Reader, T>> {
        let reader = ResourceRef::new(self).buffered().read()?;
        Ok(serde_json::Deserializer::from_reader(reader).into_iter())
    }

    async fn read_vec_async(&self) -> io::Result<Vec<u8>> {
        let reader = ResourceRef::new(self).read_async().await?;
        let mut data = Vec::new();
        pin!(reader).read_to_end(&mut data).await?;
        Ok(data)
    }
    async fn read_string_async(&self) -> io::Result<String> {
        let reader = ResourceRef::new(self).read_async().await?;
        let mut data = String::new();
        pin!(reader).read_to_string(&mut data).await?;
        Ok(data)
    }
    async fn read_json_async<T: DeserializeOwned>(&self) -> io::Result<T> {
        let data = self.read_async().await?.read_into_vec().await?;
        serde_json::from_slice(&data).map_err(io::Error::from)
    }
    async fn read_json_lines_async<T: DeserializeOwned>(
        &self,
    ) -> io::Result<impl Stream<Item = io::Result<T>>> {
        Ok(stream::try_unfold((), |()| async move { todo!() }))
    }
}
pub trait WriteResource: Resource {
    type Writer: io::Write;
    fn write_with(&self, f: impl FnOnce(&mut Self::Writer) -> io::Result<()>) -> io::Result<()>;

    type AsyncWriter: tokio::io::AsyncWrite;
    async fn write_async_with(
        &self,
        f: impl AsyncFnOnce(Pin<&mut Self::AsyncWriter>) -> io::Result<()>,
    ) -> io::Result<()>;

    fn write_resource(&self, resource: &impl ReadResource) -> io::Result<()> {
        self.write_with(|writer| std::io::copy(&mut resource.read()?, writer).map(drop))
    }
    async fn write_resource_async(&self, resource: &impl ReadResource) -> io::Result<()> {
        self.write_async_with(async |mut writer| {
            tokio::io::copy(&mut pin!(resource.read_async().await?), &mut writer)
                .await
                .map(drop)
        })
        .await
    }

    fn write_slice(&self, data: &[u8]) -> io::Result<()> {
        self.write_with(|writer| writer.write_all(data))
    }
    async fn write_slice_async(&self, data: &[u8]) -> io::Result<()> {
        self.write_async_with(async |mut writer| writer.write_all(data).await)
            .await
    }

    fn write_json<T: serde::Serialize>(&self, data: &T) -> std::io::Result<()> {
        self.write_with(|writer| Ok(serde_json::to_writer(writer, data)?))
    }
    async fn write_json_async<T: serde::Serialize>(&self, data: &T) -> std::io::Result<()> {
        // TODO: avoid buffering in memory
        let data = serde_json::to_vec(data)?;
        self.write_slice_async(&data).await
    }

    fn write_json_lines<T: serde::Serialize>(
        &self,
        data: impl IntoIterator<Item = T>,
    ) -> std::io::Result<()> {
        self.write_with(|writer| {
            std::io::copy(
                &mut utile::jsonl::JsonLinesReader::new(data.into_iter()),
                writer,
            )
            .map(drop)
        })
    }
    async fn write_json_lines_async<T: serde::Serialize>(
        &self,
        data: impl IntoIterator<Item = T>,
    ) -> io::Result<()> {
        self.write_async_with(async |mut writer| {
            let items = stream::iter(data);
            let mut items = pin!(items);

            let mut vec = Vec::new();
            while let Some(item) = items.next().await {
                vec.clear();
                serde_json::to_writer(&mut vec, &item)?;
                vec.push(b'\n');
                writer.write_all(&vec).await?;
            }
            Ok(())
        })
        .await
    }
}
pub trait ResourceExt: Resource + Sized {
    fn buffered(self) -> BufferedResource<Self> {
        BufferedResource::new(self)
    }

    fn with_fs_cache(self, cache: &crate::cache::fs::FsCache) -> FsCacheResource<Self> {
        FsCacheResource::new(cache, self)
    }
    fn with_global_fs_cache(self) -> FsCacheResource<Self> {
        FsCacheResource::new(&crate::cache::fs::FsCache::global(), self)
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
}
impl<T: Resource> ResourceExt for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Compression {
    Gzip,
    MultiGzip,
    // TODO: nested
    Brotli,
}
impl Compression {
    pub fn infer_strict(filename: &str) -> Option<Self> {
        if filename.ends_with(".gz") {
            Some(Self::Gzip)
        } else if filename.ends_with(".bgz") {
            Some(Self::MultiGzip)
        } else if filename.ends_with(".br") {
            Some(Self::Brotli)
        } else {
            None
        }
    }
    pub fn infer(filename: &str) -> Option<Self> {
        if filename.ends_with(".gz") || filename.ends_with(".bgz") {
            // We default to multi-gzip because it doesn't fail silently.
            Some(Self::MultiGzip)
        } else if filename.ends_with(".br") {
            Some(Self::Brotli)
        } else {
            None
        }
    }
    pub fn extension(self) -> &'static str {
        match self {
            Self::Gzip => "gz",
            Self::MultiGzip => "bgz",
            Self::Brotli => "br",
        }
    }
    pub fn trim_extension(self, filename: &str) -> &str {
        match self {
            Compression::Gzip => filename.strip_suffix(".gz").unwrap_or(filename),
            Compression::MultiGzip => filename
                .strip_suffix(".bgz")
                .unwrap_or_else(|| filename.strip_suffix(".gz").unwrap_or(filename)),
            Compression::Brotli => filename.strip_suffix(".br").unwrap_or(filename),
        }
    }
}

/// Just a helper struct to avoid a blanket `impl RawResource for &R`
/// or requiring a `Clone` bound in some places.
/// (The blanket impl would allow the builder api to take a reference
/// which in practice can cause annoying lifetime issues.)
struct ResourceRef<'a, R: ?Sized> {
    resource: &'a R,
}
impl<'a, R: Resource + ?Sized> ResourceRef<'a, R> {
    pub fn new(resource: &'a R) -> Self {
        Self { resource }
    }
}
impl<'a, R: Resource + ?Sized> Resource for ResourceRef<'a, R> {
    const NAMESPACE: &'static str = R::NAMESPACE;
    fn key(&self) -> String {
        R::key(self.resource)
    }
    fn compression(&self) -> Option<Compression> {
        R::compression(self.resource)
    }
}
impl<'a, R: ReadResource + ?Sized> ReadResource for ResourceRef<'a, R> {
    type Reader = R::Reader;
    fn size(&self) -> io::Result<u64> {
        R::size(self.resource)
    }
    fn read(&self) -> io::Result<Self::Reader> {
        R::read(self.resource)
    }

    type AsyncReader = R::AsyncReader;
    async fn size_async(&self) -> io::Result<u64> {
        R::size_async(self.resource).await
    }
    async fn read_async(&self) -> io::Result<Self::AsyncReader> {
        R::read_async(self.resource).await
    }
}
