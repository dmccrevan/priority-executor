use std::future::Future;
use std::panic::catch_unwind;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::thread;
use std::time;
use threadpool::ThreadPool;

use crossbeam::channel;
use once_cell::sync::Lazy;

struct JoinHandle<R>(async_task::JoinHandle<R, ()>);

impl<R> Future for JoinHandle<R> {
    type Output = R;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.0).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(output) => Poll::Ready(output.expect("Task failed")),
        }
    }
}

type Task = async_task::Task<()>;
static QUEUE: Lazy<channel::Sender<Task>> = Lazy::new(|| {
    let v: Arc<Mutex<Vec<Task>>> = Arc::new(Mutex::new(Vec::new()));
    let (sender, receiver) = channel::unbounded::<Task>();
    thread::spawn({
        let v = Arc::clone(&v);
        move || {
            receiver.iter().for_each(|task| {
                println!("Task received!");
                let mut vector = v.lock().unwrap();
                vector.push(task);
            })
        }
    });
    thread::spawn({
        let v = Arc::clone(&v);
        move || {
            let pool = ThreadPool::with_name("worker".into(), 2);
            loop {
                let mut vector = v.lock().unwrap();
                if !vector.is_empty() {
                    let task = vector.pop().unwrap();
                    pool.execute(|| {
                        task.run();
                    })
                }
            }
        }
    });
    sender
});

fn spawn<F, R>(future: F) -> JoinHandle<R>
where
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    let (task, handle) = async_task::spawn(future, |t| QUEUE.send(t).unwrap(), ());
    task.schedule();
    JoinHandle(handle)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parses_nothing() {
        futures::executor::block_on(async {
            // Spawn a future.
            let handle = spawn(async {
                println!("Running task...");
                thread::sleep(time::Duration::from_secs(10));
                1
            });
            let handle2 = spawn(async {
                println!("Running task...");
                thread::sleep(time::Duration::from_secs(10));
                2
            });
            let handle3 = spawn(async {
                println!("Running task...");
                thread::sleep(time::Duration::from_secs(10));
                3
            });
            let handle4 = spawn(async {
                println!("Running task...");
                thread::sleep(time::Duration::from_secs(10));
                4
            });
            let handle5 = spawn(async {
                println!("Running task...");
                thread::sleep(time::Duration::from_secs(10));
                5
            });
            let handle6 = spawn(async {
                println!("Running task...");
                thread::sleep(time::Duration::from_secs(10));
                6
            });

            // Await its output.
            //assert_eq!(handle.await, 3);
            let (a, b, c, d, e, f) =
                futures::join!(handle, handle2, handle3, handle4, handle5, handle6);
            println!("{} {} {} {} {} {}", a, b, c, d, e, f);
        });
    }
}
