# A minimal async runtime for Cortex-M microcontrollers

This crate provides a minimal async runtime for Cortex-M microcontrollers. It's based on the tutorial by Phil Opperman,
[Writing an OS in Rust](https://os.phil-opp.com/async-await/). The runtime is designed to be as simple as possible, while
still providing a good foundation for building async applications on Cortex-M microcontrollers. Note that this is a project
for learning purposes and is not intended for production use. For a more complete async runtime, consider using the
[Embassy Project](https://embassy.dev).


## Minimal Example

```rust
#![no_std]
#![no_main]

use cortex_m_asyncrt::os::{self, executor, init_heap, Task};
use cortex_m_rt::entry;
use cortex_m_semihosting::{dbg, hprintln};
// use panic_probe as _;
use panic_semihosting as _;

#[entry]
fn main() -> ! {
    init_heap();
    hprintln!("Hello, worlds!");
    // New executor that can run up to 64 tasks
    let mut executor = executor::Executor::new::<64>();

    // Spawn a task
    executor.spawn(Task::new(example_task()));

    // Run the executor
    executor.run();

    // This code is unreachable because the executor.run() function runs tasks to completion.
    loop {}
}

async fn example_task() {
    // your code goes here
    let r = example_fn().await;

    hprintln!("r = {}", r);
}

async fn example_fn() -> u32 {
    42
}
```

Compile and run the example with qemu.

```sh
qemu-system-arm -cpu cortex-m4 \
                -machine lm3s6965evb \
                -nographic \
                -semihosting-config enable=on,target=native \
                -kernel target/thumbv7em-none-eabihf/release/qemu-test
```

You should see the following output:

```
Hello, worlds!
r = 42
```
