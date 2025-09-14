#![cfg_attr(target_family = "wasm", allow(unused_variables))] // TODO

pub struct Task {
    #[cfg(not(target_family = "wasm"))]
    task: tokio_task::Task,
}
impl Drop for Task {
    fn drop(&mut self) {
        // Deferred to inner types, here just to document.
    }
}

impl Task {
    pub fn new(f: impl Future<Output = ()> + Send + 'static) -> Self {
        assert!(cfg!(not(target_family = "wasm"))); // TODO
        Self {
            #[cfg(not(target_family = "wasm"))]
            task: tokio_task::Task::new(f),
        }
    }
    pub fn new_local(f: impl Future<Output = ()> + 'static) -> Self {
        assert!(cfg!(not(target_family = "wasm"))); // TODO
        Self {
            #[cfg(not(target_family = "wasm"))]
            task: tokio_task::Task::new_local(f),
        }
    }
}

#[cfg(not(target_family = "wasm"))]
mod tokio_task {
    use tokio::task::JoinHandle;

    #[derive(Debug)]
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

mod boilerplate {
    use super::Task;

    impl std::fmt::Debug for Task {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            #[cfg(not(target_family = "wasm"))]
            {
                self.task.fmt(f)
            }
            #[cfg(target_family = "wasm")]
            {
                f.write_str("Task { .. }")
            }
        }
    }
}
