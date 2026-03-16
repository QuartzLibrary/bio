use std::{
    pin::{Pin, pin},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
    time::Duration,
};

use web_time::Instant;

use crate::{any::AnyMap, task::Task};

pub async fn sleep(duration: Duration) {
    #[cfg(not(target_family = "wasm"))]
    tokio::time::sleep(duration).await;

    #[cfg(target_family = "wasm")]
    {
        // The usage of the channel makes this `Send` + `Sync`.
        let (send, recv) = futures::channel::oneshot::channel();
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::sleep(duration).await;
            let _ = send.send(());
        });
        recv.await.unwrap();
    }
}

pub async fn sleep_until(until: Instant) {
    #[cfg(not(target_family = "wasm"))]
    tokio::time::sleep_until(tokio::time::Instant::from_std(until)).await;
    #[cfg(target_family = "wasm")]
    {
        while let now = Instant::now()
            && now < until
        {
            sleep(until - now).await;
        }
    }
}

#[track_caller]
pub fn time<O>(f: impl FnOnce() -> O) -> (O, Duration) {
    let start = Instant::now();
    let output = f();
    let duration = start.elapsed();
    (output, duration)
}

pub trait TimedFuture: Future {
    #[track_caller]
    fn check_time(self) -> TimeCheckFuture<Self>
    where
        Self: Sized,
    {
        TimeCheckFuture::new(self)
    }
    #[track_caller]
    fn assert_1ms(self) -> TimeCheckFuture<Self>
    where
        Self: Sized,
    {
        self.check_time()
            .max_wallclock(Duration::from_millis(1))
            .heartbeat()
            .assert()
    }
    #[track_caller]
    fn assert_1s(self) -> TimeCheckFuture<Self>
    where
        Self: Sized,
    {
        self.check_time()
            .max_wallclock(Duration::from_secs(1))
            .heartbeat()
            .assert()
    }
    /// Asserts that the total execution (blocking) time for this future is less than 1ms.
    /// Since it only check execution time, this can be used for future that use non-blocking I/O.
    #[track_caller]
    fn assert_blocking_1ms(self) -> TimeCheckFuture<Self>
    where
        Self: Sized,
    {
        self.check_time()
            .max(Duration::from_millis(1))
            // TODO: non-local [Task] not implemented on wasm yet.
            .any_map_if(cfg!(not(target_family = "wasm")), |f| f.heartbeat())
            .assert()
    }
}
impl<F: Future> TimedFuture for F {}

// TODO: maybe split into multiple futures?
// TODO: add proper timeout to wallclock time.
#[derive(Debug)]
#[pin_project::pin_project]
pub struct TimeCheckFuture<F> {
    #[pin]
    future: F,

    blocking: Arc<Mutex<(Duration, Option<Instant>)>>,
    blocking_max: Option<Duration>,

    wallclock_start: Option<Instant>,
    wallclock_max: Option<Duration>,

    heartbeat: Option<Heartbeat>,
    heartbeat_spacing: Duration,

    assert: bool,

    done: Arc<AtomicBool>,

    loc: &'static std::panic::Location<'static>,
}
#[derive(Debug)]
enum Heartbeat {
    ToSpawn,
    #[expect(dead_code)]
    Spawned(Task),
    Done,
}
impl<F> TimeCheckFuture<F> {
    #[track_caller]
    pub fn new(future: F) -> Self {
        Self {
            future,
            blocking: Arc::new(Mutex::new((Duration::ZERO, None))),
            blocking_max: None,
            wallclock_start: None,
            wallclock_max: None,
            heartbeat: None,
            heartbeat_spacing: Duration::from_secs(1),
            assert: false,
            done: Arc::new(AtomicBool::new(false)),
            loc: std::panic::Location::caller(),
        }
    }
    pub fn max(mut self, max: Duration) -> Self {
        self.blocking_max = Some(max);
        self
    }
    pub fn max_wallclock(mut self, max: Duration) -> Self {
        self.wallclock_max = Some(max);
        self
    }
    /// Spawns a heartbeat task on the first poll which will warn or panic
    /// if the future exceeds the allotted time.
    pub fn heartbeat(mut self) -> Self {
        self.heartbeat = Some(Heartbeat::ToSpawn);
        self
    }
    pub fn heartbeat_spacing(mut self, spacing: Duration) -> Self {
        self.heartbeat_spacing = spacing;
        self
    }
    pub fn assert(mut self) -> Self {
        self.assert = true;
        self
    }
}

macro_rules! time_exceeded {
    ($assert:expr, $($arg:tt)*) => {
        if $assert {
            panic!($($arg)*);
        } else {
            log::warn!($($arg)*);
        }
    };
}

impl<F: Future> Future for TimeCheckFuture<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let self_ = self.project();

        let loc = *self_.loc;
        let assert = *self_.assert;

        let poll_start = Instant::now();
        let start = *self_.wallclock_start.get_or_insert(poll_start);

        // Setup the heartbeat
        if matches!(self_.heartbeat, Some(Heartbeat::ToSpawn)) {
            let done = self_.done.clone();
            let wallclock = match *self_.wallclock_max {
                Some(max) => {
                    let done = done.clone();
                    let until = start + max;
                    Some(async move {
                        sleep_until(until).await;
                        if done.load(Ordering::Acquire) {
                            return;
                        }
                        time_exceeded!(
                            assert,
                            "Heartbeat exceeded allotted wallclock time: {max:?}.\n\
                            Future defined at: {loc}"
                        );
                    })
                }
                None => None,
            };
            let blocking = match *self_.blocking_max {
                Some(max) => {
                    let spacing = *self_.heartbeat_spacing;
                    let current_blocking = self_.blocking.clone();
                    Some(async move {
                        let keep_going = move || -> bool {
                            let (duration, start) = *current_blocking.lock().unwrap();
                            let in_progress =
                                start.as_ref().map(Instant::elapsed).unwrap_or_default();
                            duration + in_progress < max
                        };
                        while keep_going() && !done.load(Ordering::Acquire) {
                            sleep(spacing).await;
                        }
                        if done.load(Ordering::Acquire) {
                            return;
                        }
                        time_exceeded!(
                            assert,
                            "Heartbeat exceeded allotted blocking time: {max:?}.\n\
                            Future defined at: {loc}"
                        );
                    })
                }
                None => None,
            };

            let task = match (wallclock, blocking) {
                (Some(wallclock), Some(blocking)) => Some(Task::new(async move {
                    futures::future::select(pin!(wallclock), pin!(blocking)).await;
                })),
                (Some(wallclock), None) => Some(Task::new(wallclock)),
                (None, Some(blocking)) => Some(Task::new(blocking)),
                (None, None) => None,
            };

            if let Some(task) = task {
                *self_.heartbeat = Some(Heartbeat::Spawned(task));
            } else {
                log::warn!("No limits set for heartbeat for future defined at: {loc}");
                *self_.heartbeat = Some(Heartbeat::Done);
            }
        }

        {
            // Start the blocking timer.
            let (_duration, start) = &mut *self_.blocking.lock().unwrap();
            debug_assert!(start.is_none());
            *start = Some(poll_start);
        }

        let output = self_.future.poll(cx);

        let poll_end = Instant::now();

        if output.is_ready() {
            // We are done, so either way we can issue any warning or panic here.
            *self_.heartbeat = Some(Heartbeat::Done);
        }

        let is_done = self_.done.load(Ordering::Acquire);

        // Check the wallclock timer.
        if let Some(max) = *self_.wallclock_max
            && start + max < poll_end
            && !is_done
        {
            let actual = poll_end - start;
            time_exceeded!(
                assert,
                "Future exceeded allotted wallclock time: {max:?} < {actual:?}.\n\
                Future defined at: {loc}"
            );
            *self_.heartbeat = Some(Heartbeat::Done);
        }

        // Advance the blocking timer.
        let blocking = {
            let (duration, start) = &mut *self_.blocking.lock().unwrap();
            debug_assert!(start.is_some());
            *duration += poll_end - poll_start;
            *start = None;

            *duration
        };

        // Check the blocking timer.
        if let Some(max) = *self_.blocking_max
            && max < blocking
            && !is_done
        {
            let actual = blocking;
            time_exceeded!(
                assert,
                "Future exceeded allotted blocking time: {max:?} < {actual:?}.\n\
                Future defined at: {loc}"
            );
            *self_.heartbeat = Some(Heartbeat::Done);
        }

        if output.is_ready() {
            self_.done.store(true, Ordering::Release);
        }

        output
    }
}
