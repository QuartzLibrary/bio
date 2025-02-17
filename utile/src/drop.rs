pub struct ExecuteOnDrop<F: FnOnce()>(Option<F>);
impl<F: FnOnce()> ExecuteOnDrop<F> {
    pub fn new(f: F) -> Self {
        Self(Some(f))
    }
}
impl<F: FnOnce()> Drop for ExecuteOnDrop<F> {
    fn drop(&mut self) {
        self.0.take().unwrap()();
    }
}

pub struct ExecuteOnPanic<F: FnOnce()>(Option<F>);
impl<F: FnOnce()> ExecuteOnPanic<F> {
    pub fn new(f: F) -> Self {
        Self(Some(f))
    }
}
impl<F: FnOnce()> Drop for ExecuteOnPanic<F> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.0.take().unwrap()();
        }
    }
}
