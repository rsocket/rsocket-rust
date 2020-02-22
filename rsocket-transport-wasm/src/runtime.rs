use rsocket_rust::runtime::Spawner;
use std::future::Future;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone)]
pub struct WASMSpawner;

impl Spawner for WASMSpawner {
    fn spawn<F>(&self, task: F)
    where
        F: Send + Future<Output = ()> + 'static,
    {
        spawn_local(task);
    }
}
