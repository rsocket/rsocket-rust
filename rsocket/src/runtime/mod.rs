use std::future::Future;

pub trait Spawner {
    fn spawn<T, F>(&self, task: F)
    where
        F: Send + Future<Output = T> + 'static,
        T: Send + 'static;
}

#[derive(Clone)]
pub struct DefaultSpawner;

impl Spawner for DefaultSpawner {
    fn spawn<T, F>(&self, task: F)
    where
        F: Send + Future<Output = T> + 'static,
        T: Send + 'static,
    {
        tokio::spawn(task);
    }
}
