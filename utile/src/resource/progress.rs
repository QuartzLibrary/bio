use std::pin::Pin;

use indicatif::ProgressStyle;

use super::{Compression, RawResource};

const PROGRESS_BAR_STYLE: &str =
    "{spinner} {bytes} ({percent}%) of {total_bytes} | {bytes_per_sec} {wide_bar} {eta}";

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
