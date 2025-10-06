pub trait TryU64: Sized {
    type Error: std::fmt::Debug;

    fn try_u64(self) -> Result<u64, Self::Error>;
    #[track_caller]
    fn u64_unwrap(self) -> u64 {
        self.try_u64().unwrap()
    }
}
impl<T> TryU64 for T
where
    T: TryInto<u64>,
    <Self as TryInto<u64>>::Error: std::fmt::Debug,
{
    type Error = <Self as TryInto<u64>>::Error;

    fn try_u64(self) -> Result<u64, Self::Error> {
        self.try_into()
    }
}

pub trait TryI64: Sized {
    type Error: std::fmt::Debug;

    fn try_i64(self) -> Result<i64, Self::Error>;
    #[track_caller]
    fn i64_unwrap(self) -> i64 {
        self.try_i64().unwrap()
    }
}
impl<T> TryI64 for T
where
    T: TryInto<i64>,
    <Self as TryInto<i64>>::Error: std::fmt::Debug,
{
    type Error = <Self as TryInto<i64>>::Error;

    fn try_i64(self) -> Result<i64, Self::Error> {
        self.try_into()
    }
}

pub trait TryUsize: Sized {
    type Error: std::fmt::Debug;

    fn try_usize(self) -> Result<usize, Self::Error>;
    #[track_caller]
    fn usize_unwrap(self) -> usize {
        self.try_usize().unwrap()
    }
}
impl<T> TryUsize for T
where
    T: TryInto<usize>,
    <Self as TryInto<usize>>::Error: std::fmt::Debug,
{
    type Error = <Self as TryInto<usize>>::Error;

    fn try_usize(self) -> Result<usize, Self::Error> {
        self.try_into()
    }
}
