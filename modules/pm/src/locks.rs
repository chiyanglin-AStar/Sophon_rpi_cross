use alloc::vec::Vec;
use atomic::Ordering;
use core::sync::atomic::AtomicBool;
use kernel_module::SERVICE;
use proc::TaskId;

#[derive(Default)]
pub struct RawMutex {
    is_locked: AtomicBool,
    waiters: spin::Mutex<Vec<TaskId>>,
}

impl RawMutex {
    pub const fn new() -> Self {
        Self {
            is_locked: AtomicBool::new(false),
            waiters: spin::Mutex::new(Vec::new()),
        }
    }

    pub fn lock(&self) {
        let _guard = interrupt::uninterruptible();
        let task = SERVICE.scheduler().get_current_task_id().unwrap();
        while self.is_locked.fetch_or(true, Ordering::SeqCst) {
            self.waiters.lock().push(task);
            syscall::wait();
        }
    }

    pub fn unlock(&self) {
        let _guard = interrupt::uninterruptible();
        self.is_locked.store(false, Ordering::SeqCst);
        let mut waiters = self.waiters.lock();
        for t in &*waiters {
            SERVICE.scheduler().wake_up(*t)
        }
        waiters.clear()
    }
}

#[derive(Default)]
pub struct RawCondvar {
    waiters: spin::Mutex<Vec<TaskId>>,
}

impl RawCondvar {
    pub const fn new() -> Self {
        Self {
            waiters: spin::Mutex::new(Vec::new()),
        }
    }

    pub fn wait(&self, lock: &RawMutex) {
        let _guard = interrupt::uninterruptible();
        {
            let mut waiters = self.waiters.lock();
            let task = SERVICE.scheduler().get_current_task_id().unwrap();
            lock.unlock();
            waiters.push(task);
        }
        syscall::wait();
        lock.lock();
    }

    pub fn notify_all(&self) {
        let _guard = interrupt::uninterruptible();
        let mut waiters = self.waiters.lock();
        for t in &*waiters {
            SERVICE.scheduler().wake_up(*t)
        }
        waiters.clear()
    }
}
