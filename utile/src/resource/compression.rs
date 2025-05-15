use std::{
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;

use super::{Compression, RawResource, RawResourceExt, ResourceRef};

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
        let key = self.resource.key();
        let key = match self.resource.compression() {
            Some(compression) => compression.trim_extension(&key),
            None => &key,
        };
        format!("{}/{key}", R::NAMESPACE)
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
        match self.compression.or(self.resource.compression()) {
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
        match self.compression.or(self.resource.compression()) {
            None => Ok(AsyncDecompressedReader::None(
                self.resource.read_async().await?,
            )),
            Some(Compression::Gzip) => Ok(AsyncDecompressedReader::Gzip(
                async_compression::tokio::bufread::GzipDecoder::new(
                    ResourceRef::new(&self.resource)
                        .buffered()
                        .read_async()
                        .await?,
                ),
            )),
            Some(Compression::MultiGzip) => todo!(),
        }
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompressedResource<R> {
    resource: R,
    compression: Compression,
}
impl<R: RawResource> CompressedResource<R> {
    pub fn new(resource: R, compression: Compression) -> Self {
        Self {
            resource,
            compression,
        }
    }
}
impl<R: RawResource> RawResource for CompressedResource<R> {
    const NAMESPACE: &'static str = "compressed";
    fn key(&self) -> String {
        format!(
            "{}/{}.{}",
            R::NAMESPACE,
            self.resource.key(),
            self.compression.extension()
        )
    }
    fn compression(&self) -> Option<Compression> {
        Some(self.compression)
    }

    type Reader = CompressedReader<R::Reader>;

    fn size(&self) -> std::io::Result<u64> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Compressed resources do not have a size.",
        ))
    }

    fn read(&self) -> std::io::Result<Self::Reader> {
        match self.compression {
            Compression::Gzip => Ok(CompressedReader::Gzip(flate2::bufread::GzEncoder::new(
                ResourceRef::new(&self.resource).buffered().read()?,
                flate2::Compression::default(),
            ))),
            Compression::MultiGzip => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Multi-gzip compression is not supported.",
            )),
        }
    }

    type AsyncReader = AsyncCompressedReader<R::AsyncReader>;

    async fn size_async(&self) -> std::io::Result<u64> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Compressed resources do not have a size.",
        ))
    }

    async fn read_async(&self) -> std::io::Result<Self::AsyncReader> {
        match self.compression {
            Compression::Gzip => Ok(AsyncCompressedReader::Gzip(
                async_compression::tokio::bufread::GzipEncoder::new(
                    ResourceRef::new(&self.resource)
                        .buffered()
                        .read_async()
                        .await?,
                ),
            )),
            Compression::MultiGzip => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Multi-gzip compression is not supported.",
            )),
        }
    }
}
#[derive(Debug)]
pub enum CompressedReader<R> {
    Gzip(flate2::bufread::GzEncoder<std::io::BufReader<R>>),
    // MultiGzip(flate2::read::GzEncoder<std::io::BufReader<R>>), //
}
impl<R: std::io::Read> std::io::Read for CompressedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Gzip(r) => r.read(buf),
        }
    }
}

#[derive(Debug)]
#[pin_project(project = AsyncCompressedReaderProj)]
pub enum AsyncCompressedReader<R> {
    Gzip(#[pin] async_compression::tokio::bufread::GzipEncoder<tokio::io::BufReader<R>>),
    // MultiGzip(#[pin] async_compression::tokio::bufread::GzipDecoder<tokio::io::BufReader<R>>), //
}
impl<R: tokio::io::AsyncRead> tokio::io::AsyncRead for AsyncCompressedReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.project() {
            AsyncCompressedReaderProj::Gzip(reader) => reader.poll_read(cx, buf),
        }
    }
}
