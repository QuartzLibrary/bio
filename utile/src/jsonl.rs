use crate::io::messagepack_error;

pub struct JsonLinesReader<I, T> {
    iter: I,
    data: Vec<u8>,
    pos: usize,
    _marker: std::marker::PhantomData<T>,
}
impl<I, T> JsonLinesReader<I, T>
where
    I: Iterator<Item = T>,
    T: serde::Serialize,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            data: Vec::new(),
            pos: 0,
            _marker: std::marker::PhantomData,
        }
    }
}
impl<I, T> std::io::Read for JsonLinesReader<I, T>
where
    I: Iterator<Item = T>,
    T: serde::Serialize,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut total_read = 0;
        while total_read < buf.len() {
            if self.data_left().is_empty() {
                let Some(item) = self.iter.next() else {
                    break;
                };
                self.fill_data_with(item)?;
            }
            total_read += self.copy_data(&mut buf[total_read..]);
        }
        Ok(total_read)
    }
}
impl<I, T> JsonLinesReader<I, T>
where
    I: Iterator<Item = T>,
    T: serde::Serialize,
{
    fn data_left(&self) -> &[u8] {
        &self.data[self.pos..]
    }
    fn fill_data_with(&mut self, item: T) -> Result<(), serde_json::Error> {
        debug_assert_eq!(self.data.len(), self.pos);
        self.data.clear();
        serde_json::to_writer(&mut self.data, &item)?;
        self.data.push(b'\n');
        self.pos = 0;
        Ok(())
    }
    fn copy_data(&mut self, buf: &mut [u8]) -> usize {
        let data_left = self.data_left();
        let span = std::cmp::min(buf.len(), data_left.len());
        buf[..span].copy_from_slice(&data_left[..span]);
        self.pos += span;
        span
    }
}

pub struct MessagePackLinesReader<I, T> {
    iter: I,
    data: Vec<u8>,
    pos: usize,
    _marker: std::marker::PhantomData<T>,
}
impl<I, T> MessagePackLinesReader<I, T>
where
    I: Iterator<Item = T>,
    T: serde::Serialize,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            data: Vec::new(),
            pos: 0,
            _marker: std::marker::PhantomData,
        }
    }
}
impl<I, T> std::io::Read for MessagePackLinesReader<I, T>
where
    I: Iterator<Item = T>,
    T: serde::Serialize,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut total_read = 0;
        while total_read < buf.len() {
            if self.data_left().is_empty() {
                let Some(item) = self.iter.next() else {
                    break;
                };
                self.fill_data_with(item).map_err(messagepack_error)?;
            }
            total_read += self.copy_data(&mut buf[total_read..]);
        }
        Ok(total_read)
    }
}
impl<I, T> MessagePackLinesReader<I, T>
where
    I: Iterator<Item = T>,
    T: serde::Serialize,
{
    fn data_left(&self) -> &[u8] {
        &self.data[self.pos..]
    }
    fn fill_data_with(&mut self, item: T) -> Result<(), rmp_serde::encode::Error> {
        debug_assert_eq!(self.data.len(), self.pos);
        self.data.clear();
        rmp_serde::encode::write(&mut self.data, &item)?;
        self.data.push(b'\n');
        self.pos = 0;
        Ok(())
    }
    fn copy_data(&mut self, buf: &mut [u8]) -> usize {
        let data_left = self.data_left();
        let span = std::cmp::min(buf.len(), data_left.len());
        buf[..span].copy_from_slice(&data_left[..span]);
        self.pos += span;
        span
    }
}
