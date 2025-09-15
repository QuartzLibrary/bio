/// Useful to avoid breaking method chains when a conditional or
/// slightly more involved manipulation is required.
pub trait AnyMap: Sized {
    fn any_map<O>(self, f: impl FnOnce(Self) -> O) -> O {
        f(self)
    }
    fn any_map_if(self, if_: bool, f: impl FnOnce(Self) -> Self) -> Self {
        if if_ { f(self) } else { self }
    }
}

impl<T> AnyMap for T {}
