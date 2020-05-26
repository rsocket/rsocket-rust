use std::future::Future;

pub trait Spawner {
    fn spawn<F>(&self, task: F)
    where
        F: Send + Future<Output = ()> + 'static;
}

#[derive(Clone, Copy, Debug)]
pub struct DefaultSpawner;

impl Spawner for DefaultSpawner {
    fn spawn<F>(&self, task: F)
    where
        F: Send + Future<Output = ()> + 'static,
    {
        tokio::spawn(task);
    }
}
