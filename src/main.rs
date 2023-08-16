mod logger;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use logger::SimpleLogger;

#[derive(Debug)]
pub enum ThreadWorkerStatus {
    Idle,
    Running,
    Stop,
}

#[allow(dead_code)]
pub struct Worker {
    status: ThreadWorkerStatus,
    value: String,
    id: u32,
}

impl Worker {
    pub fn new(id: u32) -> Worker {
        Worker {
            status: ThreadWorkerStatus::Idle,
            value: "".to_string(),
            id,
        }
    }
}

struct ThreadWorker {
    lock: Arc<(Mutex<Worker>, Condvar)>,
    job: Option<std::thread::JoinHandle<()>>,
}

impl ThreadWorker {
    pub fn new() -> ThreadWorker {
        ThreadWorker {
            lock: Arc::new((Mutex::new(Worker::new(1)), Condvar::new())),
            job: None,
        }
    }
    pub fn init(&mut self) -> Result<(), String> {
        if let Some(_) = self.job {
            return Err("ThreadWorker::init: already initialized".to_string());
        }

        // Copy the lock to be used in the thread
        let lock = Arc::clone(&self.lock);
        self.job = Some(thread::spawn(move || {
            log::debug!("ThreadWorker: job start");
            loop {
                log::debug!("ThreadWorker: job loop");
                {
                    // Get the lock
                    let (lock, cvar) = &*lock;
                    let mut worker = lock.lock().unwrap();
                    log::debug!("ThreadWorker: job loop get lock");

                    // Wait for the status to change
                    while let ThreadWorkerStatus::Idle = worker.status {
                        log::debug!("ThreadWorker: job loop wait start");
                        worker = cvar.wait(worker).unwrap();
                        log::debug!("ThreadWorker: job loop wait end");
                    }

                    // Check if we need to stop
                    if let ThreadWorkerStatus::Stop = worker.status {
                        break;
                    }

                    // Do the work
                    if let ThreadWorkerStatus::Running = worker.status {
                        log::debug!("ThreadWorker: job loop running");
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
            log::debug!("ThreadWorker: job end");
        }));

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        log::debug!("ThreadWorker::stop start");
        let (lock, cvar) = &*self.lock;
        let mut worker = lock.lock().map_err(|e| e.to_string())?;
        log::debug!("ThreadWorker::stop get lock");
        if let Some(job) = self.job.take() {
            log::debug!("ThreadWorker::stop setup stop flag");
            worker.status = ThreadWorkerStatus::Stop;
            log::debug!("ThreadWorker::stop notify");
            cvar.notify_one();
            drop(worker);
            if !job.is_finished() {
                log::debug!("ThreadWorker::stop join job");
                let _ = job.join();
            }
        }
        self.job = None;
        Ok(())
    }

    pub fn update(&mut self, status: ThreadWorkerStatus) -> Result<(), String> {
        log::debug!("ThreadWorker::update start");
        let (lock, cvar) = &*self.lock;
        log::debug!("ThreadWorker::update get lock");
        let mut worker = lock.lock().map_err(|e| e.to_string())?;
        worker.status = status;
        log::debug!("ThreadWorker::update notify");
        cvar.notify_one();
        Ok(())
    }
}

fn main() {
    let log_level = log::Level::Debug;
    SimpleLogger::new_with_level(log_level).init().unwrap();

    let mut worker = ThreadWorker::new();
    worker.init().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(3000));
    worker.update(ThreadWorkerStatus::Running).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(3000));
    worker.stop().unwrap();
}
