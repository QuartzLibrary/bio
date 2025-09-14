pub mod enumerable {
    pub trait Enumerable {
        const N: u128;
    }

    impl Enumerable for bool {
        const N: u128 = 2;
    }

    impl Enumerable for u8 {
        const N: u128 = Self::MAX as u128;
    }
    impl Enumerable for u16 {
        const N: u128 = Self::MAX as u128;
    }
    impl Enumerable for u32 {
        const N: u128 = Self::MAX as u128;
    }
    impl Enumerable for u64 {
        const N: u128 = Self::MAX as u128;
    }
    impl Enumerable for u128 {
        const N: u128 = Self::MAX;
    }
    impl Enumerable for usize {
        const N: u128 = Self::MAX as u128;
    }

    impl Enumerable for i8 {
        const N: u128 = u8::MAX as u128;
    }
    impl Enumerable for i16 {
        const N: u128 = u16::MAX as u128;
    }
    impl Enumerable for i32 {
        const N: u128 = u32::MAX as u128;
    }
    impl Enumerable for i64 {
        const N: u128 = u64::MAX as u128;
    }
    impl Enumerable for i128 {
        const N: u128 = u128::MAX;
    }
    impl Enumerable for isize {
        const N: u128 = usize::MAX as u128;
    }

    impl<T> Enumerable for Option<T>
    where
        T: Enumerable,
    {
        const N: u128 = T::N + 1;
    }
    impl<T, E> Enumerable for Result<T, E>
    where
        T: Enumerable,
        E: Enumerable,
    {
        const N: u128 = T::N + E::N;
    }
    impl<T, const N: usize> Enumerable for [T; N]
    where
        T: Enumerable,
    {
        const N: u128 = T::N.pow(N as u32);
    }
}
