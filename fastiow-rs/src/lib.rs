#![allow(dead_code)]
#![allow(unused_variables)]

mod bits;
mod fastiow;
mod gpio;
mod i2c;
mod iowarrior;
mod iowkit;

pub use self::fastiow::get_iowarriors;
pub use self::i2c::{I2CAddress, I2C};
pub use self::iowarrior::*;

extern crate iowkit_sys;
extern crate libloading;
