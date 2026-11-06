use std::future::Future;
use std::sync::OnceLock;

use tokio::runtime::Handle;
use tokio::task::JoinHandle;

static RUNTIME: OnceLock<Handle> = OnceLock::new();

pub fn adopt(handle: Handle) {
    if RUNTIME.set(handle).is_err() {
        log::debug!("the session's runtime was already set");
    }
}

fn handle() -> &'static Handle {
    RUNTIME.get().expect("the session starts inside a runtime")
}

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    handle().spawn(future)
}

pub fn spawn_blocking<F, R>(work: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    handle().spawn_blocking(work)
}
