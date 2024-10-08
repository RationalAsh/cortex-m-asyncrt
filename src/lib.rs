#![no_std]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub mod os;
pub use macros::async_main;
