use std::sync::{Arc, Mutex};
use std::{sync::mpsc, thread};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.

    pub fn new(count: usize) -> ThreadPool {
        assert!(count > 0);
        let mut workers = Vec::with_capacity(count);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..count {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
        where
        F: FnOnce() + Send + 'static,
        {
            let job = Box::new(f);
            self.sender.as_ref().unwrap().send(job).unwrap();
        }
}

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("Worker {} got a job; executing.", { id });
                    job();
                }
                Err(_) => {
                    println!("Worker id {} disconnected. Shutting down", id);
                    break;
                }
            };
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                println!("Shutting down worker {}", worker.id);
                thread.join().unwrap();
            }
        }
    }
}


mod tests{
    #![allow(dead_code,unused)]
    use super::*;

    #[test]
    fn create_threadpool(){
        let pool = ThreadPool::new(4);
        assert_eq!(4,pool.workers.len());
        assert_eq!(pool.sender.is_some(),true);
    }

    #[test]
    #[should_panic(expected = "assertion failed: count > 0")]
    fn test_new_thread_pool_invalid_count() {
        ThreadPool::new(0);
    }

}
