use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Shared {
    queue: VecDeque<Job>,
    shutdown: bool,
}

struct SharedState {
    inner: Mutex<Shared>,
    cv: Condvar,
}

/// A generic thread pool implemented with Arc + Mutex + Condvar.
/// - No third-party crates
/// - Dynamic number of workers
/// - Graceful shutdown + join
pub struct ThreadPool {
    state: Arc<SharedState>,
    workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn new(num_workers: usize) -> Self {
        assert!(num_workers > 0, "ThreadPool needs at least 1 worker");

        let state = Arc::new(SharedState {
            inner: Mutex::new(Shared {
                queue: VecDeque::new(),
                shutdown: false,
            }),
            cv: Condvar::new(),
        });

        let mut workers = Vec::with_capacity(num_workers);
        for id in 0..num_workers {
            workers.push(Worker::spawn(id, Arc::clone(&state)));
        }

        Self { state, workers }
    }

    pub fn size(&self) -> usize {
        self.workers.len()
    }

    pub fn execute<F>(&self, f: F) -> Result<(), &'static str>
    where
        F: FnOnce() + Send + 'static,
    {
        let mut shared = self.state.inner.lock().map_err(|_| "mutex poisoned")?;
        if shared.shutdown {
            return Err("thread pool is shut down");
        }
        shared.queue.push_back(Box::new(f));
        self.state.cv.notify_one();
        Ok(())
    }

    pub fn shutdown(&mut self) {
        {
            let mut shared = match self.state.inner.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            shared.shutdown = true;
            // Wake all workers so they can observe shutdown.
            self.state.cv.notify_all();
        }

        for w in &mut self.workers {
            if let Some(handle) = w.thread.take() {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

struct Worker {
    #[allow(dead_code)]
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn spawn(id: usize, state: Arc<SharedState>) -> Self {
        let thread = thread::spawn(move || loop {
            let job_opt: Option<Job> = {
                let mut shared = match state.inner.lock() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };

                while shared.queue.is_empty() && !shared.shutdown {
                    shared = match state.cv.wait(shared) {
                        Ok(g) => g,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                }

                if shared.shutdown && shared.queue.is_empty() {
                    None
                } else {
                    shared.queue.pop_front()
                }
            };

            match job_opt {
                Some(job) => job(),
                None => break,
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn runs_jobs() {
        let pool = ThreadPool::new(4);
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..500 {
            let c = Arc::clone(&counter);
            pool.execute(move || {
                c.fetch_add(1, Ordering::SeqCst);
            }).unwrap();
        }

        drop(pool);
        assert_eq!(counter.load(Ordering::SeqCst), 500);
    }

    #[test]
    fn shutdown_is_graceful() {
        let mut pool = ThreadPool::new(2);
        for _ in 0..10 {
            pool.execute(|| std::thread::sleep(Duration::from_millis(5))).unwrap();
        }
        pool.shutdown();
        // should not panic on double shutdown
        pool.shutdown();
    }
}
