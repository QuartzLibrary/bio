use std::{
    pin::Pin,
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

// TODO
// pub async fn sleep_until(i: Instant) {
//     #[cfg(not(target_family = "wasm"))]
//     tokio::time::sleep_until(tokio::time::Instant::from_std(i)).await;
//     #[cfg(target_family = "wasm")]
//     gloo_timers::future::sleep_until(i).await;
// }

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
            // TODO: [Task] not implementes on wasm yet.
            .any_map_if(cfg!(not(target_family = "wasm")), |f| f.heartbeat())
            .assert()
    }
    #[track_caller]
    fn assert_1s(self) -> TimeCheckFuture<Self>
    where
        Self: Sized,
    {
        self.check_time()
            .max_wallclock(Duration::from_secs(1))
            // TODO: [Task] not implementes on wasm yet.
            .any_map_if(cfg!(not(target_family = "wasm")), |f| f.heartbeat())
            .assert()
    }
    /// Asserts that the total execution (blocking) time for this future is less than 1ms.
    /// Since it only check execution time, this can be used for future that use non-blocking I/O.
    #[track_caller]
    fn assert_blocking_1ms(self) -> TimeCheckFuture<Self>
    where
        Self: Sized,
    {
        self.check_time().max(Duration::from_millis(1)).assert()
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

    blocking: Duration,
    blocking_max: Duration,

    wallclock_start: Instant,
    wallclock_max: Option<Instant>,

    heartbeat: Option<Option<Task>>,

    assert: bool,

    loc: &'static std::panic::Location<'static>,
}
impl<F> TimeCheckFuture<F> {
    #[track_caller]
    pub fn new(future: F) -> Self {
        Self {
            future,
            blocking: Duration::ZERO,
            blocking_max: Duration::MAX,
            wallclock_start: Instant::now(),
            wallclock_max: None,
            heartbeat: None,
            assert: false,
            loc: std::panic::Location::caller(),
        }
    }
    pub fn max(mut self, max: Duration) -> Self {
        self.blocking_max = max;
        self
    }
    pub fn max_wallclock(mut self, max: Duration) -> Self {
        self.wallclock_max = Some(self.wallclock_start + max);
        self
    }
    /// Spwans an heartbeat task on the first poll which will warn or panic
    /// if the future is not dropped by the time it's done.
    pub fn heartbeat(mut self) -> Self {
        self.heartbeat = Some(None);
        self
    }
    pub fn assert(mut self) -> Self {
        self.assert = true;
        self
    }
}
impl<F: Future> Future for TimeCheckFuture<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let self_ = self.project();

        let loc = *self_.loc;

        if matches!(self_.heartbeat, Some(None))
            && let Some(max) = self_.wallclock_max
        {
            let nominal_duration = *max - *self_.wallclock_start;
            let sleep_for = *max - Instant::now();

            let assert = *self_.assert;

            *self_.heartbeat = Some(Some(Task::new(async move {
                sleep(sleep_for).await;
                if assert {
                    panic!(
                        "Heartbeat exceeded alloted time: {nominal_duration:?}.\n\
                        Future defined at: {loc}"
                    );
                }
                log::warn!(
                    "Heartbeat exceeded alloted time: {nominal_duration:?}.\n\
                    Future defined at: {loc}"
                );
            })));
        }

        let (output, blocking) = time(|| self_.future.poll(cx));

        if let Some(wallclock_max) = *self_.wallclock_max
            && let now = Instant::now()
            && wallclock_max < now
        {
            let max = wallclock_max - *self_.wallclock_start;
            let actual = now - *self_.wallclock_start;
            if *self_.assert {
                panic!(
                    "Future exceeded alloted time: {max:?} < {actual:?}.\n\
                    Future defined at: {loc}",
                );
            }
            log::warn!(
                "Future exceeded alloted time: {max:?} < {actual:?}.\n\
                Future defined at: {loc}",
            );
        }

        *self_.blocking = self_.blocking.checked_add(blocking).unwrap();

        {
            let blocking = *self_.blocking;
            let blocking_max = *self_.blocking_max;
            if blocking > blocking_max {
                if *self_.assert {
                    panic!(
                        "Future exceeded alloted time: {blocking_max:?} < {blocking:?}.\n\
                        Future defined at: {loc}",
                    );
                }
                log::warn!(
                    "Future exceeded alloted time: {blocking_max:?} < {blocking:?}.\n\
                    Future defined at: {loc}",
                );
            }
        }

        output
    }
}
