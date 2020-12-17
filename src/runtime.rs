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

async fn pingpong(executor: Arc<Executor<'_>>) -> io::Result<()> {
    // spawn hello loop in parallel
    let ex = executor.clone();
    let ex2 = ex.clone();
    let task1 = ex.spawn(async {
        foo().await;
    });
    let task2 = ex2.spawn(async {
        bar().await;
    });

    ex.spawn(async {
        println!("Debug 1");
        // Using this sleep will block everything since we are using a single thread
        // for running the executor
        //thread::sleep(Duration::from_secs(5));
        sleep(Duration::from_secs(5)).await;
        println!("Debug 2");
    })
    .await;

    // This cancels the running tasks
    task1.cancel().await;
    task2.cancel().await;

    // This will wait for the tasks to finish
    //task1.await;
    //task2.await;

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
                pingpong(ex).await;
                drop(signal);
            })
        });
}

fn main() -> io::Result<()> {
    let ex = Arc::new(Executor::new());
    runtime(ex.clone());
    Ok(())
}
