use super::{current_task, TaskControlBlock};
use crate::sync::UPSafeCell;
use alloc::collections::binary_heap::BinaryHeap;
// use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;

pub struct TaskManager {
    ready_queue: BinaryHeap<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self {
            // ready_queue: VecDeque::new(),
            ready_queue: BinaryHeap::new(),
        }
    }
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push(task);
    }
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop().inspect(|t| {
            t.inner_exclusive_access().add_stride();
        })
    }
    pub fn get_task(&mut self, pid: usize) -> Option<Arc<TaskControlBlock>> {
        let task = current_task().unwrap();
        if task.pid.0 == pid {
            return task.into();
        }
        self.ready_queue.iter().find(|v| v.pid.0 == pid).cloned()
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}

pub fn get_task(pid: usize) -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().get_task(pid)
}
