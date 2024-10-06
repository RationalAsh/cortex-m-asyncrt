extern crate alloc;

use alloc::boxed::Box;
use core::ptr::addr_of_mut;
use core::sync::atomic::{AtomicU32, Ordering};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

/// Initialize the heap.
pub fn init_heap() {
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 1024;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
}

/// Task ID type. We use a 32 bit unsigned integer to represent a task ID.
/// since the cortex-m architecture is 32 bit and does not have support for
/// atomic 64 bit integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u32);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Base struct to represent a task.
pub struct Task {
    /// A unique identifier for the task.
    id: TaskId,
    /// The future representing the task.
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    /// Create a new task from a future.
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    /// Poll the task.
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

pub mod executor;
pub mod time;
