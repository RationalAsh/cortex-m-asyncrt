use conquer_once::spin::OnceCell;
use cortex_m::peripheral::syst;
use cortex_m_rt::exception;
use fugit::{Duration, Instant};

/// Create a global static instance of the `Timer` struct.
// static TIMER: OnceCell<Timer> = OnceCell::uninit();

// pub struct Timer {
//     pub ticks: u32,
// }

// impl Timer {
//     pub fn advance_one_tick(&mut self) {
//         self.ticks += 1;
//     }
// }

#[exception]
fn SysTick() {
    cortex_m::asm::nop();
    cortex_m::asm::nop();
    cortex_m::asm::nop();
}
