use {
    crate::{p, error::GtResult},
    std::future::Future,
    tokio::{runtime::{Builder, Runtime}, task::JoinHandle}
};

lazy_static::lazy_static! {
    static ref RT: Runtime = runtime_builder().unwrap();
}

fn runtime_builder() -> GtResult<Runtime> {

    Ok(p!(Builder::new()
    .threaded_scheduler()
    .enable_all()
    .core_threads(2)
    .max_threads(8)
    .build()))

}

// pub fn run<F>(f: F) -> JoinHandle<F::Output>
//     where F: Future + Send + 'static,
//     F::Output: Send + 'static
// {
//     RT.handle().spawn(f)
// }

pub fn run_cb_local<F, C>(f: F, mut c: C)
    where F: Future + Send + 'static,
    F::Output: Send + 'static,
    C: FnMut(F::Output) + 'static
{
    let (tx, rx) = glib::MainContext::channel::<F::Output>(glib::PRIORITY_DEFAULT);
    RT.handle().spawn(async move {
        drop(tx.send(f.await));
    });
    rx.attach(None, move |msg: F::Output| {
        c(msg);
        glib::Continue(false)
    });
}

// pub fn run_blocking<F: Future>(f: F) -> F::Output {
//     RT.handle().block_on(f)
// }
