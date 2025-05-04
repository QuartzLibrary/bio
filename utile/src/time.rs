use std::time::Duration;

pub async fn sleep(duration: Duration) {
    #[cfg(not(target_family = "wasm"))]
    tokio::time::sleep(duration).await;

    #[cfg(target_family = "wasm")]
    {
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
