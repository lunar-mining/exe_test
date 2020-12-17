use async_channel::unbounded;
use async_dup::Arc;
use async_executor::Executor;
use easy_parallel::Parallel;
use smol::{future, io, Timer};
use std::time::Duration;

async fn sleep(dur: Duration) {
    Timer::after(dur).await;
}

async fn foo() {
    loop {
        println!("Hello fren");
        sleep(Duration::from_secs(2)).await;
    }
}

async fn bar() {
    loop {
        println!("fren");
        sleep(Duration::from_secs(1)).await;
    }
}

async fn nested_arc(executor: Arc<Executor<'_>>) {
    for _ in 0..5 {
        let ex = executor.clone();
        let task = executor.clone().spawn(async move {
            loop {
                pingpong(ex.clone()).await;
            }
        });
    }
}

async fn pingpong(executor: Arc<Executor<'_>>) -> io::Result<()> {
    let ex = executor.clone();
    let ex2 = ex.clone();
    let task1 = ex.spawn(async {
        foo().await;
    });
    let task2 = ex2.spawn(async {
        bar().await;
    });

    task1.await;
    task2.await;

    Ok(())
}

fn runtime(executor: Arc<Executor<'_>>) {
    let (signal, shutdown) = unbounded::<()>();

    let ex = executor.clone();
    Parallel::new()
        // Run four executor threads.
        .each(0..1, |_| future::block_on(executor.run(shutdown.recv())))
        // Run the main future on the current thread.
        .finish(|| {
            future::block_on(async {
                nested_arc(ex).await;
                drop(signal);
            })
        });
}

fn main() -> io::Result<()> {
    let ex = Arc::new(Executor::new());
    runtime(ex.clone());
    Ok(())
}
