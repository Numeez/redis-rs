use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

use crate::{hash::Value, zset::LARGE_ZSET};

type Job = Box<dyn FnOnce() + Send + 'static>;
pub struct ThreadPool {
    threads: Vec<JoinHandle<()>>,
    sender: Sender<Job>,
}

impl ThreadPool {
    pub fn new(n: usize) -> Self {
        let mut threads = vec![];
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver: Arc<Mutex<Receiver<Job>>> = Arc::new(Mutex::new(receiver));
        for _ in 0..n {
            let rec = Arc::clone(&receiver);
            let handle = thread::spawn(move || {
                loop {
                    let message = rec.lock().unwrap().recv();
                    match message {
                        Ok(job) => job(),
                        Err(_) => break,
                    }
                }
            });
            threads.push(handle);
        }

        Self { threads, sender }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(f)).unwrap()
    }
}

pub fn drop_value(value: Value, threadpool: &ThreadPool) {
    match value {
        Value::ZSet(zset) => {
            if zset.len() >= LARGE_ZSET {
                threadpool.execute(move || {
                    eprintln!("freeing zset of {} members on a worker", zset.len());
                    drop(zset)
                });
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threadpool() {
        let tp = ThreadPool::new(4);
        for i in 1..11 {
            tp.execute(move || println!("Executing: {i}"));
        }
    }
}
