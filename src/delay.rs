use core::future;

pub trait Delay {
    type Future: future::Future<Output = ()>;
    fn delay_ms(&mut self, ms: u32) -> Self::Future;
}
