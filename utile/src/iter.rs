use std::ops::Range;

pub struct SplicedIterator<Base, Insert> {
    inner: Option<SplicedIteratorInner<Base, Insert>>,
}
enum SplicedIteratorInner<Base, Insert> {
    Start {
        start: usize,
        remove: usize,
        base: Base,
        insert: Insert,
    },
    Insert {
        insert: Insert,
        base: Base,
    },
    End {
        base: Base,
    },
}
impl<Base: ExactSizeIterator, Insert: Iterator<Item = Base::Item>> SplicedIterator<Base, Insert> {
    pub fn new(base: Base, range: Range<usize>, insert: Insert) -> Self {
        Self::new_checked(base, range, insert).unwrap()
    }
    pub fn new_checked(base: Base, range: Range<usize>, insert: Insert) -> Option<Self> {
        let base_len = base.len();

        if range.end < range.start {
            return None;
        }

        if range.end > base_len {
            return None;
        }

        Some(Self {
            inner: Some(SplicedIteratorInner::Start {
                start: range.start,
                remove: range.end - range.start,
                base,
                insert,
            }),
        })
    }
}
impl<Base: Iterator, Insert: Iterator<Item = Base::Item>> Iterator
    for SplicedIterator<Base, Insert>
{
    type Item = Base::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let (self_, next) = self.inner.take()?.next();
        self.inner = Some(self_);
        next
    }
}
impl<Base: Iterator, Insert: Iterator<Item = Base::Item>> SplicedIteratorInner<Base, Insert> {
    pub fn next(self) -> (Self, Option<Base::Item>) {
        match self {
            SplicedIteratorInner::Start {
                start: start @ 1..,
                remove,
                mut base,
                insert,
            } => {
                let item = base.next();
                (
                    SplicedIteratorInner::Start {
                        start: start - 1,
                        remove,
                        base,
                        insert,
                    },
                    item,
                )
            }
            SplicedIteratorInner::Start {
                start: 0,
                remove,
                mut base,
                insert,
            } => {
                for _ in 0..remove {
                    base.next();
                }
                SplicedIteratorInner::Insert { insert, base }.next()
            }
            SplicedIteratorInner::Insert { mut insert, base } => match insert.next() {
                Some(item) => (SplicedIteratorInner::Insert { insert, base }, Some(item)),
                None => SplicedIteratorInner::End { base }.next(),
            },
            SplicedIteratorInner::End { mut base } => {
                let item = base.next();
                (SplicedIteratorInner::End { base }, item)
            }
        }
    }
}
impl<Base: ExactSizeIterator, Insert: Iterator<Item = Base::Item> + ExactSizeIterator>
    ExactSizeIterator for SplicedIterator<Base, Insert>
{
    fn len(&self) -> usize {
        match self.inner.as_ref().unwrap() {
            SplicedIteratorInner::Start {
                start: _,
                remove,
                base,
                insert,
            } => base.len() - remove + insert.len(),
            SplicedIteratorInner::Insert { insert, base } => insert.len() + base.len(),
            SplicedIteratorInner::End { base } => base.len(),
        }
    }
}
