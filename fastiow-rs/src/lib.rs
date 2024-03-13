#![allow(dead_code)]
#![allow(unused_variables)]

mod fastiow;
mod iowarrior;
mod iowkit;

pub use self::fastiow::get_iowarriors;
pub use self::iowarrior::*;

extern crate iowkit_sys;
extern crate libloading;
