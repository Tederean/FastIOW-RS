use std::{fmt, thread};
use std::time::Duration;

#[derive(Debug)]
pub struct DelayNs {
}

impl fmt::Display for DelayNs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl embedded_hal::delay::DelayNs for DelayNs {
    #[inline]
    fn delay_ns(&mut self, ns: u32) {
        thread::sleep(Duration::from_nanos(ns as u64));
    }

    #[inline]
    fn delay_us(&mut self, us: u32) {
        thread::sleep(Duration::from_micros(us as u64));
    }

    #[inline]
    fn delay_ms(&mut self, ms: u32) {
        thread::sleep(Duration::from_millis(ms as u64));
    }
}