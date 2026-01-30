#[expect(dead_code)]
pub struct Task {
    #[cfg(not(target_family = "wasm"))]
    task: tokio_task::Task,
    #[cfg(target_family = "wasm")]
    task: wasm_task::Task,
}
impl Drop for Task {
    fn drop(&mut self) {
        // Deferred to inner types, here just to document.
    }
}

impl Task {
    pub fn new(f: impl Future<Output = ()> + Send + 'static) -> Self {
        Self {
            #[cfg(not(target_family = "wasm"))]
            task: tokio_task::Task::new(f),
            #[cfg(target_family = "wasm")]
            task: wasm_task::Task::new_local(f), // TODO: forbid instead?
        }
    }
    pub fn new_local(f: impl Future<Output = ()> + 'static) -> Self {
        Self {
            #[cfg(not(target_family = "wasm"))]
            task: tokio_task::Task::new_local(f),
            #[cfg(target_family = "wasm")]
            task: wasm_task::Task::new_local(f),
        }
    }
}

#[cfg(not(target_family = "wasm"))]
mod tokio_task {
    use tokio::task::JoinHandle;

    pub struct Task {
        join_handle: JoinHandle<()>,
    }
    impl Drop for Task {
        fn drop(&mut self) {
            self.join_handle.abort();
        }
    }
    impl Task {
        pub fn new(f: impl Future<Output = ()> + Send + 'static) -> Self {
            Self {
                join_handle: tokio::spawn(f),
            }
        }
        pub fn new_local(f: impl Future<Output = ()> + 'static) -> Self {
            Self {
                join_handle: tokio::task::spawn_local(f),
            }
        }
    }
}

#[cfg(target_family = "wasm")]
mod wasm_task {
    use futures::stream::{AbortHandle, Abortable};

    pub struct Task {
        abort_handle: AbortHandle,
    }

    impl Task {
        pub fn new_local(f: impl Future<Output = ()> + 'static) -> Self {
            let (abort_handle, abort_registration) = AbortHandle::new_pair();
            let f = Abortable::new(f, abort_registration);
            let f = async move {
                let _ = f.await;
            };
            wasm_bindgen_futures::spawn_local(f);
            Self { abort_handle }
        }
    }

    impl Drop for Task {
        fn drop(&mut self) {
            self.abort_handle.abort();
        }
    }
}

mod boilerplate {
    use super::Task;

    impl std::fmt::Debug for Task {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("Task { .. }")
        }
    }
}
