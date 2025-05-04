use std::{fmt, path::PathBuf};

use crate::cache::{FsCache, FsCacheEntry};

use super::{Compression, RawResource, RawResourceExt, ResourceRef};

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
            entry: FsCacheEntry::new(cache, PathBuf::from(R::NAMESPACE).join(resource.key())),
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
            return self.entry.read();
        }

        log::info!("Downloading {self} from {self}");

        self.entry
            .write_file(ResourceRef::new(&self.resource).buffered().read()?)?;

        log::info!("Downloaded {self}");

        self.entry.read()
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
            return self.entry.read_async().await;
        }

        log::info!("Downloading {self} from {self}");

        self.entry
            .write_file_async(
                ResourceRef::new(&self.resource)
                    .buffered()
                    .read_async()
                    .await?,
            )
            .await?;

        log::info!("Downloaded {self}");

        self.entry.read_async().await
    }
}
