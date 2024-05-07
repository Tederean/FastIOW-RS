#![allow(dead_code)]
#![allow(unused_variables)]

pub mod bits;
pub(crate) mod communication;
pub mod delay;
pub mod digital;
pub mod i2c;
pub mod iowarrior;
pub mod pwm;
pub mod spi;
pub use communication::initialization_service::*;

#[macro_export]
macro_rules! pin {
    ($n:expr, $m:expr) => {
        8 * $n + $m
    };
}
