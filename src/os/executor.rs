extern crate alloc;

use super::*;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use cortex_m::asm;
use crossbeam_queue::ArrayQueue;

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }
    fn wake_task(&self) {
        self.task_queue
            .push(self.task_id)
            .expect("Task queue is full!");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

/// Task executor that runs tasks to completion.
pub struct Executor {
    /// The tasks to run are stored in a BTreeMap. This allows us to store tasks
    /// in a way that they can be accessed by their ID. BtreeMap is used because
    /// it is a sorted map and we can use the task ID as the key.
    tasks: BTreeMap<TaskId, Task>,
    /// The task queue is a queue of task IDs. This queue is used to keep track of
    /// the order in which tasks are spawned. The executor will run tasks in the
    /// order they were spawned.
    task_queue: Arc<ArrayQueue<TaskId>>,
    /// The waker cache is a map of task IDs to wakers. This cache is used to store
    /// wakers for tasks that are currently running. This allows us to wake up tasks
    /// when they are ready to be polled again.
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    /// Create a new executor with an empty task queue.
    pub fn new<const N: usize>() -> Executor {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(N)),
            waker_cache: BTreeMap::new(),
        }
    }

    /// Spawn a new task. This function pushes the task to the back of the task queue.
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task_id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.task_queue.push(task_id).expect("Task queue is full.");
    }

    /// Run the executor. This function runs tasks to completion.
    fn run_ready_tasks(&mut self) {
        // Use a while let loop to pop task_ids from the task queue.
        while let Some(task_id) = self.task_queue.pop() {
            // Get the task from the tasks map.
            let task = match self.tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task was removed from the tasks map so we skip it.
            };

            // Get the waker for the task from the waker cache.
            let waker = self
                .waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, self.task_queue.clone())); // create a new waker if it doesn't exist.

            // Create a new context from the waker.
            let mut context = Context::from_waker(waker);

            // Poll the task.
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task is done, so we remove it and its cached waker.
                    self.tasks.remove(&task_id);
                    self.waker_cache.remove(&task_id);
                }
                Poll::Pending => {} // task is not done, so we do nothing.
            }
        }
    }

    /// Put the processor to sleep if there are no tasks to run.
    fn sleep_if_idle(&self) {
        // If there are no tasks to run, we put the processor to sleep until an interrupt occurs.
        cortex_m::interrupt::free(|_| {
            if self.task_queue.is_empty() {
                asm::wfi(); // Wait for interrupt.
            }
        });
    }

    /// Run the executor. This function will run tasks until the task queue is empty.
    pub fn run(&mut self) {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }
}

pub fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);

    RawWaker::new(0 as *const (), vtable)
}

pub fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}
