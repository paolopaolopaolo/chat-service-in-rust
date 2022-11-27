use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex, MutexGuard};

type ThreadVector = Vec<Option<JoinHandle<()>>>;
type Threads = Arc<Mutex<ThreadVector>>;

pub struct Threadpool {
    threads: Threads,
}

impl Threadpool {
    pub fn new(pool_count: usize) -> Threadpool {
        let mut thread_vector: ThreadVector = Vec::with_capacity(pool_count);
        for _ in 0..pool_count {
            thread_vector.push(None);
        }
        let threads: Threads = Arc::new(Mutex::new(thread_vector));
        Threadpool {
            threads: threads,
        }
    }

    pub fn execute<F>(&mut self, closure: F)
    where 
        F: FnOnce() -> () + Send + 'static
    {
        let thread_arc: Threads = self.threads.clone();
        let mut threads: MutexGuard<ThreadVector>  = thread_arc.lock().unwrap();
        let thread_count: usize = threads.capacity();
        let mut keep_loop_alive: bool = true;
        let mut idx_plop: usize = 0;
        while keep_loop_alive {
            for idx in 0..thread_count {
                match &threads[idx] {
                    Some(thr) => if thr.is_finished() {
                        idx_plop = idx;
                        keep_loop_alive = false;
                        break;
                    },
                    _ => { 
                        idx_plop = idx;
                        keep_loop_alive = false;
                        break; 
                    }
                }
            }
        }
        threads[idx_plop] = Some(thread::spawn(closure)); 
    }

}
