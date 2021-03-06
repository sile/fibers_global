//! This crate provides the global [`ThreadPoolExecutor`] that enables to spawn/execute fibers anywhere in a program.
//!
//! This is useful for briefly writing test or example code that use [`fibers`].
//!
//! [`ThreadPoolExecutor`]: https://docs.rs/fibers/0.1/fibers/struct.ThreadPoolExecutor.html
//! [`fibers`]: https://github.com/dwango/fibers-rs
//!
//! # Examples
//!
//! ```
//! # extern crate fibers;
//! # extern crate fibers_global;
//! # extern crate futures;
//! use fibers::sync::oneshot;
//! use futures::{lazy, Future};
//!
//! # fn main() {
//! // Spawns two auxiliary fibers.
//! let (tx0, rx0) = oneshot::channel();
//! let (tx1, rx1) = oneshot::channel();
//! fibers_global::spawn(lazy(move || {
//!     let _ = tx0.send(1);
//!     Ok(())
//! }));
//! fibers_global::spawn(lazy(move || {
//!     let _ = tx1.send(2);
//!     Ok(())
//! }));
//!
//! // Executes a calculation that depends on the above fibers.
//! let result = fibers_global::execute(rx0.join(rx1).map(|(v0, v1)| v0 + v1));
//! assert_eq!(result.ok(), Some(3));
//! # }
//! ```
#![warn(missing_docs)]
#[macro_use]
extern crate lazy_static;

use fibers::executor::ThreadPoolExecutorHandle;
use fibers::sync::oneshot::{Monitor, MonitorError};
use fibers::Spawn;
use futures::{Async, Future};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

static THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);

fn get_thread_count() -> usize {
    match THREAD_COUNT.swap(std::usize::MAX, Ordering::SeqCst) {
        0 => num_cpus::get(),
        n => n,
    }
}

/// Sets the number of scheduler threads used by the global executor.
///
/// If the global executor already has started,
/// the invocation of this function has no effect and `false` is returned.
///
/// # Panics
///
/// If the specified count is `0` or `usize::MAX`, the current thread will panic.
pub fn set_thread_count(n: usize) -> bool {
    assert_ne!(n, 0);
    assert_ne!(n, std::usize::MAX);

    loop {
        let current = THREAD_COUNT.load(Ordering::SeqCst);
        if current == std::usize::MAX {
            return false;
        }
        if THREAD_COUNT.compare_and_swap(current, n, Ordering::SeqCst) == current {
            return true;
        }
    }
}

lazy_static! {
    static ref GLOBAL_EXECUTOR: ThreadPoolExecutorHandle = {
        use fibers::Executor;

        let executor = fibers::ThreadPoolExecutor::with_thread_count(get_thread_count())
            .expect("Cannot create the global `ThreadPoolExecutor`");
        let handle = executor.handle();
        std::thread::spawn(move || {
            executor
                .run()
                .expect("The global `ThreadPoolExecutor` aborted")
        });
        handle
    };
}

/// Spawns a fiber to execute the given future by using the global `ThreadPoolExecutor`.
pub fn spawn<F>(future: F)
where
    F: Future<Item = (), Error = ()> + Send + 'static,
{
    handle().spawn(future);
}

/// Spawns a fiber by using the global `ThreadPoolExecutor` and returns a future to monitor it's execution result.
pub fn spawn_monitor<F>(future: F) -> Monitor<F::Item, F::Error>
where
    F: Future + Send + 'static,
    F::Item: Send + 'static,
    F::Error: Send + 'static,
{
    handle().spawn_monitor(future)
}

/// Returns the handle of the global `ThreadPoolExecutor`.
pub fn handle() -> ThreadPoolExecutorHandle {
    GLOBAL_EXECUTOR.clone()
}

/// Executes the given future by using the global `ThreadPoolExecutor` and waits the result.
pub fn execute<F>(future: F) -> Result<F::Item, F::Error>
where
    F: Future + Send + 'static,
    F::Item: Send + 'static,
    F::Error: Send + 'static,
{
    let mut monitor = handle().spawn_monitor(future);
    loop {
        match monitor.poll() {
            Err(MonitorError::Aborted) => panic!("The global `ThreadPoolExecutor` aborted"),
            Err(MonitorError::Failed(e)) => return Err(e),
            Ok(Async::Ready(v)) => return Ok(v),
            Ok(Async::NotReady) => {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use fibers::sync::oneshot;
    use futures::{lazy, Future};

    use super::*;

    #[test]
    fn it_works() {
        let (tx0, rx0) = oneshot::channel();
        let (tx1, rx1) = oneshot::channel();
        spawn(lazy(move || {
            let _ = tx0.send(1);
            Ok(())
        }));
        spawn(lazy(move || {
            let _ = tx1.send(2);
            Ok(())
        }));

        let result = execute(rx0.join(rx1).map(|(v0, v1)| v0 + v1));
        assert_eq!(result.ok(), Some(3));
    }

}
