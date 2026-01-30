use utile::jsonl::JsonLinesReader;

use super::RawResource;

pub struct IterToJsonLinesResource<I, T> {
    key: String,
    iter: JsonLinesReader<I, T>,
}
impl<I, T> IterToJsonLinesResource<I, T>
where
    I: Iterator<Item = T>,
    T: serde::Serialize,
{
    pub fn new(key: String, iter: I) -> Self {
        Self {
            key,
            iter: JsonLinesReader::new(iter),
        }
    }
}
impl<I, T> RawResource for IterToJsonLinesResource<I, T>
where
    I: Iterator<Item = T> + Clone,
    T: serde::Serialize,
{
    const NAMESPACE: &'static str = "jsonl";
    fn key(&self) -> String {
        self.key.clone()
    }

    fn compression(&self) -> Option<super::Compression> {
        None
    }

    type Reader = JsonLinesReader<I, T>;

    fn size(&self) -> std::io::Result<u64> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "JsonLinesResource do not have a size.",
        ))
    }

    fn read(&self) -> std::io::Result<Self::Reader> {
        Ok(self.iter.clone()) // TODO: avoid clone
    }

    type AsyncReader = std::io::Cursor<&'static [u8]>;

    async fn size_async(&self) -> std::io::Result<u64> {
        panic!("JsonLinesResource::size_async is not supported.");
    }

    async fn read_async(&self) -> std::io::Result<Self::AsyncReader> {
        panic!("JsonLinesResource::read_async is not supported.");
    }
}
