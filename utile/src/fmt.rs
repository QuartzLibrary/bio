use std::fmt;

pub struct FmtClosure<F> {
    f: F,
}
impl<F> FmtClosure<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    pub fn new(f: F) -> Self {
        Self { f }
    }
}
impl<F> fmt::Debug for FmtClosure<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.f)(f)
    }
}
impl<F> fmt::Display for FmtClosure<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.f)(f)
    }
}
