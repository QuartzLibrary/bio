use std::{
    fmt,
    path::{Path, PathBuf},
    sync::LazyLock,
};
#[cfg(not(target_arch = "wasm32"))]
use std::{io, pin::Pin};

use directories::ProjectDirs;

use utile::io::not_found_error;

#[cfg(not(target_arch = "wasm32"))]
use crate::WriteResource;
use crate::{ReadResource, Resource};

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
        assert!(path.as_ref().is_absolute(), "{}", path.as_ref().display());
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
    pub fn new_temp() -> (Self, tempfile::TempDir) {
        let temp = tempfile::Builder::new()
            .suffix("bio_data")
            .tempdir()
            .unwrap();
        (
            Self {
                path: temp.path().to_path_buf(),
            },
            temp,
        )
    }

    pub fn entry(&self, key: impl AsRef<Path>) -> FsCacheEntry {
        FsCacheEntry::new(self, key)
    }

    pub fn subfolder(&self, key: impl AsRef<Path>) -> Self {
        Self::new(self.path.join(key))
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

    pub fn try_exists(&self) -> std::io::Result<bool> {
        self.as_ref().try_exists()
    }

    #[cfg(not(target_arch = "wasm32"))] // TODO
    pub async fn try_exists_async(&self) -> std::io::Result<bool> {
        tokio::fs::try_exists(&self).await
    }

    /// Unfortunately some sources aren't pure.
    pub fn invalidate(&self) -> std::io::Result<()> {
        std::fs::remove_file(self)
    }
    /// Unfortunately some sources aren't pure.
    #[cfg(not(target_arch = "wasm32"))] // TODO
    pub async fn invalidate_async(&self) -> std::io::Result<()> {
        tokio::fs::remove_file(&self).await
    }
}
impl Resource for FsCacheEntry {
    const NAMESPACE: &'static str = "fs_cache";
    fn key(&self) -> String {
        self.path.to_string_lossy().as_ref().to_owned()
    }

    fn compression(&self) -> Option<crate::Compression> {
        None
    }
}
impl ReadResource for FsCacheEntry {
    type Reader = std::fs::File;
    fn size(&self) -> std::io::Result<u64> {
        std::fs::metadata(self).map(|m| m.len())
    }
    fn read(&self) -> std::io::Result<Self::Reader> {
        std::fs::File::open(self).map_err(|e| not_found_error(e, self))
    }

    #[cfg(not(target_arch = "wasm32"))] // TODO
    type AsyncReader = tokio::fs::File;
    #[cfg(not(target_arch = "wasm32"))] // TODO
    async fn size_async(&self) -> std::io::Result<u64> {
        tokio::fs::metadata(self).await.map(|m| m.len())
    }
    #[cfg(not(target_arch = "wasm32"))] // TODO
    async fn read_async(&self) -> std::io::Result<Self::AsyncReader> {
        tokio::fs::File::open(self)
            .await
            .map_err(|e| not_found_error(e, self))
    }
    #[cfg(target_arch = "wasm32")]
    type AsyncReader = std::io::Cursor<&'static [u8]>;
    #[cfg(target_arch = "wasm32")]
    async fn size_async(&self) -> std::io::Result<u64> {
        panic!("FsCacheEntry is not supported on wasm32");
    }
    #[cfg(target_arch = "wasm32")]
    async fn read_async(&self) -> std::io::Result<Self::AsyncReader> {
        panic!("FsCacheEntry is not supported on wasm32");
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl WriteResource for FsCacheEntry {
    type Writer = tempfile::NamedTempFile;
    fn write_with(&self, f: impl FnOnce(&mut Self::Writer) -> io::Result<()>) -> io::Result<()> {
        std::fs::create_dir_all(self.path.parent().unwrap())?;

        let mut tmp_file = tempfile::Builder::new()
            .prefix("tempfile_")
            .tempfile_in(self.path.parent().unwrap())?;
        f(&mut tmp_file)?;

        std::fs::rename(tmp_file.path(), self)
    }

    type AsyncWriter = tokio::fs::File;
    async fn write_async_with(
        &self,
        f: impl AsyncFnOnce(Pin<&mut Self::AsyncWriter>) -> io::Result<()>,
    ) -> io::Result<()> {
        tokio::fs::create_dir_all(self.path.parent().unwrap()).await?;

        let tmp_file = tempfile::Builder::new()
            .prefix("tempfile_")
            .tempfile_in(self.path.parent().unwrap())?;
        let mut writer = tokio::fs::File::create(tmp_file.path()).await?;
        f(Pin::new(&mut writer)).await?;
        drop(writer);

        tokio::fs::rename(tmp_file.path(), self).await
    }
}
