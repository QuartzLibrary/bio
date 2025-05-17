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
            total_read += self.read_inner(&mut buf[total_read..]);
        }
        Ok(total_read)
    }
}
impl<I, T> std::io::BufRead for JsonLinesReader<I, T>
where
    I: Iterator<Item = T>,
    T: serde::Serialize,
{
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if self.data_left().is_empty() {
            let Some(item) = self.iter.next() else {
                return Ok(&[]);
            };
            self.fill_data_with(item)?;
        }
        Ok(self.data_left())
    }

    fn consume(&mut self, amt: usize) {
        self.pos += amt;
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
    fn read_inner(&mut self, buf: &mut [u8]) -> usize {
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
                self.fill_data_with(item).map_err(std::io::Error::other)?;
            }
            total_read += self.read_inner(&mut buf[total_read..]);
        }
        Ok(total_read)
    }
}
impl<I, T> std::io::BufRead for MessagePackLinesReader<I, T>
where
    I: Iterator<Item = T>,
    T: serde::Serialize,
{
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if self.data_left().is_empty() {
            let Some(item) = self.iter.next() else {
                return Ok(&[]);
            };
            self.fill_data_with(item).map_err(std::io::Error::other)?;
        }
        Ok(self.data_left())
    }

    fn consume(&mut self, amt: usize) {
        self.pos += amt;
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
        rmp_serde::encode::write_named(&mut self.data, &item)?;
        self.data.push(b'\n');
        self.pos = 0;
        Ok(())
    }
    fn read_inner(&mut self, buf: &mut [u8]) -> usize {
        let data_left = self.data_left();
        let span = std::cmp::min(buf.len(), data_left.len());
        buf[..span].copy_from_slice(&data_left[..span]);
        self.pos += span;
        span
    }
}

mod boilerplate {
    use super::*;

    impl<I: std::fmt::Debug, T> std::fmt::Debug for JsonLinesReader<I, T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("JsonLinesReader")
                .field("iter", &self.iter)
                .field("data", &self.data)
                .field("pos", &self.pos)
                .field("_marker", &self._marker)
                .finish()
        }
    }
    impl<I: Clone, T> Clone for JsonLinesReader<I, T> {
        fn clone(&self) -> Self {
            Self {
                iter: self.iter.clone(),
                data: self.data.clone(),
                pos: self.pos,
                _marker: self._marker,
            }
        }
    }
}
