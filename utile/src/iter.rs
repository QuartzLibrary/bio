use std::{cmp::Ordering, collections::VecDeque, iter::Peekable, ops::Range};

pub trait IteratorExt: Iterator + Sized {
    fn spliced(
        self,
        range: Range<usize>,
        insert: impl Iterator<Item = Self::Item>,
    ) -> SplicedIterator<Self, impl Iterator<Item = Self::Item>>
    where
        Self: ExactSizeIterator,
    {
        SplicedIterator::new(self, range, insert)
    }

    /// Sort an iterator already sorted by the first stage by the second stage.
    fn staged_sorted_by<S1, S2>(self, s1: S1, s2: S2) -> StagedSortedIterator<Self, S1, S2>
    where
        S1: FnMut(&Self::Item, &Self::Item) -> Ordering,
        S2: FnMut(&Self::Item, &Self::Item) -> Ordering,
    {
        StagedSortedIterator::new(self, s1, s2)
    }
}
impl<T: Iterator> IteratorExt for T {}

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

pub struct StagedSortedIterator<Base, S1, S2>
where
    Base: Iterator,
{
    base: Peekable<Base>,
    s1: S1,
    s2: S2,
    stage: VecDeque<Base::Item>,
}
impl<Base, S1, S2> StagedSortedIterator<Base, S1, S2>
where
    Base: Iterator,
    S1: FnMut(&Base::Item, &Base::Item) -> Ordering,
    S2: FnMut(&Base::Item, &Base::Item) -> Ordering,
{
    pub fn new(base: Base, s1: S1, s2: S2) -> Self {
        Self {
            base: base.peekable(),
            s1,
            s2,
            stage: VecDeque::new(),
        }
    }
}
impl<Base, S1, S2> Iterator for StagedSortedIterator<Base, S1, S2>
where
    Base: Iterator,
    S1: FnMut(&Base::Item, &Base::Item) -> Ordering,
    S2: FnMut(&Base::Item, &Base::Item) -> Ordering,
    Base::Item: std::fmt::Debug,
{
    type Item = Base::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.stage.pop_front() {
            return Some(item);
        }

        self.stage.push_back(self.base.next()?);
        let mut multiple = false;
        while let Some(item) = self.base.peek() {
            match (self.s1)(&self.stage[0], item) {
                Ordering::Less => break,
                Ordering::Equal => {
                    multiple = true;
                    self.stage.push_back(self.base.next()?);
                }
                Ordering::Greater => unreachable!(
                    "Iterator is not sorted by first stage {:?}, {:?}",
                    self.stage[0], item
                ),
            }
        }

        if multiple {
            self.stage.make_contiguous().sort_by(&mut self.s2);
            self.next()
        } else {
            Some(self.stage.pop_front().unwrap())
        }
    }
}
