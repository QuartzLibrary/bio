use super::{Compression, RawResource};

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
