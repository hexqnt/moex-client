#![cfg(any(feature = "async", feature = "blocking"))]

mod clients;
mod policies;
#[cfg(feature = "blocking")]
mod requests;

#[cfg(feature = "async")]
pub(in crate::moex) fn immediate<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};

    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut future = pin!(future);
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("test future unexpectedly returned Poll::Pending"),
    }
}
