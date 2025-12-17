use core::future::Future;

pub trait Delay {
    fn delay_ms(&mut self, ms: u32) -> impl Future<Output = ()>;
}
