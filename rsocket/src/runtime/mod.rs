use std::future::Future;

pub fn spawn<F>(task: F)
where
    F: Send + Future<Output = ()> + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::spawn(task);
    }

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen_futures::spawn_local;
        spawn_local(task);
    }
}
