use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool
    ///
    /// # Panics
    ///
    /// The 'new' function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Pass a function to be run by a thread in the ThreadPool
    ///
    /// The f is the function to be run
    ///
    /// # Panics
    ///
    /// The 'execute' function will panic if the receiver has been closed
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    /// Applies steps for deallocating and closing ThreadPool Safely
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Create a new Worker.
    ///
    /// The id is the thread number being created. It is not representative of the value held by the os.
    ///
    /// The receiver is the communication channel that receives jobs to be executed
    ///
    /// # Panics
    ///
    /// The 'new' function will panic if the thread already has a lock on the receiver
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Make a threadpool that works for a dual-core cpu
    fn create_thread_pool1() {
        let pool = ThreadPool::new(4);
        assert_eq!(4, pool.workers.len());
    }

    #[test]
    fn create_thread_pool2() {
        let pool = ThreadPool::new(384);
        assert_eq!(384, pool.workers.len());
    }

    #[test]
    #[should_panic]
    fn create_thread_pool3() {
        let _pool = ThreadPool::new(0);
    }
}
