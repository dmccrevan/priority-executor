use crossbeam::channel;
use once_cell::sync::Lazy;
use std::cmp::{max, Ordering};
use std::collections::BinaryHeap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::thread;
use std::time;
use threadpool::ThreadPool;

pub struct JoinHandle<R>(async_task::JoinHandle<R, ()>);

pub type Task = async_task::Task<()>;

struct TaskContainer {
    t: Task,
    priority: usize,
}

impl Eq for TaskContainer {}

impl PartialEq for TaskContainer {
    fn eq(&self, other: &TaskContainer) -> bool {
        other.priority.eq(&self.priority)
    }
}

impl Ord for TaskContainer {
    fn cmp(&self, other: &TaskContainer) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for TaskContainer {
    fn partial_cmp(&self, other: &TaskContainer) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<R> Future for JoinHandle<R> {
    type Output = R;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.0).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(output) => Poll::Ready(output.expect("Task failed")),
        }
    }
}

#[allow(dead_code)]
static QUEUE: Lazy<channel::Sender<TaskContainer>> = Lazy::new(|| {
    let h: Arc<Mutex<BinaryHeap<TaskContainer>>> = Arc::new(Mutex::new(BinaryHeap::new()));
    let (sender, receiver) = channel::unbounded::<TaskContainer>();
    thread::spawn({
        let h = Arc::clone(&h);
        move || {
            receiver.iter().for_each(|task| {
                println!("Task received!");
                let mut heap = h.lock().unwrap();
                heap.push(task);
            })
        }
    });
    thread::spawn({
        let h = Arc::clone(&h);
        move || {
            let pool = ThreadPool::with_name("worker".into(), max(1, num_cpus::get()));
            loop {
                let mut heap = h.lock().unwrap();
                if !heap.is_empty() {
                    let task = heap.pop().unwrap();
                    pool.execute(|| task.t.run())
                }
            }
        }
    });
    sender
});

pub fn spawn<F, R>(future: F, priority: usize) -> JoinHandle<R>
where
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    let (task, handle) = async_task::spawn(
        future,
        move |t| QUEUE.send(TaskContainer { t, priority }).unwrap(),
        (),
    );
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
            let handle: JoinHandle<i32> = spawn(
                async {
                    println!("Running task 1...");
                    thread::sleep(time::Duration::from_secs(10));
                    1
                },
                1,
            );
            let handle2 = spawn(
                async {
                    println!("Running task 2...");
                    thread::sleep(time::Duration::from_secs(10));
                    2
                },
                2,
            );
            let handle3 = spawn(
                async {
                    println!("Running task 3...");
                    thread::sleep(time::Duration::from_secs(10));
                    3
                },
                3,
            );
            let handle4 = spawn(
                async {
                    println!("Running task 4...");
                    thread::sleep(time::Duration::from_secs(10));
                    4
                },
                4,
            );
            let handle5 = spawn(
                async {
                    println!("Running task 5...");
                    thread::sleep(time::Duration::from_secs(10));
                    5
                },
                5,
            );
            let handle6 = spawn(
                async {
                    println!("Running task 6...");
                    thread::sleep(time::Duration::from_secs(10));
                    6
                },
                6,
            );

            // Await its output.
            let (a, b, c, d, e, f) =
                futures::join!(handle, handle2, handle3, handle4, handle5, handle6);
        });
    }
}
