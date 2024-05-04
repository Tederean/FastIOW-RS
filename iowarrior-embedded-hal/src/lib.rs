#![allow(dead_code)]
#![allow(unused_variables)]

pub mod bits;
pub mod delay;
pub mod digital;
pub mod i2c;
mod internal;
pub mod iowarrior;
pub mod pwm;
pub mod spi;
pub use self::internal::iowarrior_service::get_iowarriors;

#[macro_export]
macro_rules! pin {
    ($n:expr, $m:expr) => {
        8 * $n + $m
    };
}
