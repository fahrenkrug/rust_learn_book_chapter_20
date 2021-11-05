use std::thread;
use std::thread::JoinHandle;
use std::fmt::{Display, Formatter};
use std::sync::{mpsc, Mutex, Arc};

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Clone, Debug)]
pub struct PoolCreationError;

enum Message {
    NewJob(Job),
    Terminate
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job. Executing.", id);
                    job();
                },
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
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

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers now.");
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}.", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Display for PoolCreationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Size of ThreadPool must be 1 or higher")
    }
}

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// The size is the number of possible threads in this thread pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is 0.
    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size <= 0 {
            return Err(PoolCreationError)
        }

        let (sender, receiver ) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        Ok(ThreadPool {
            workers,
            sender
        })
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static, {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap()
    }
}