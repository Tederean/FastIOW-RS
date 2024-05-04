use std::time::Duration;
use std::{fmt, thread};

#[derive(Debug, Default)]
pub struct Delay;

impl fmt::Display for Delay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl embedded_hal::delay::DelayNs for Delay {
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

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::delay::DelayMs<u8> for Delay {
    #[inline]
    fn delay_ms(&mut self, ms: u8) {
        thread::sleep(Duration::from_millis(ms as u64));
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::delay::DelayMs<u16> for Delay {
    #[inline]
    fn delay_ms(&mut self, ms: u16) {
        thread::sleep(Duration::from_millis(ms as u64));
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::delay::DelayMs<u32> for Delay {
    #[inline]
    fn delay_ms(&mut self, ms: u32) {
        thread::sleep(Duration::from_millis(ms as u64));
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::delay::DelayMs<u64> for Delay {
    #[inline]
    fn delay_ms(&mut self, ms: u64) {
        thread::sleep(Duration::from_millis(ms));
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::delay::DelayUs<u8> for Delay {
    #[inline]
    fn delay_us(&mut self, us: u8) {
        thread::sleep(Duration::from_micros(us as u64));
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::delay::DelayUs<u16> for Delay {
    #[inline]
    fn delay_us(&mut self, us: u16) {
        thread::sleep(Duration::from_micros(us as u64));
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::delay::DelayUs<u32> for Delay {
    #[inline]
    fn delay_us(&mut self, us: u32) {
        thread::sleep(Duration::from_micros(us as u64));
    }
}

#[cfg(feature = "embedded-hal-0")]
impl embedded_hal_0::blocking::delay::DelayUs<u64> for Delay {
    #[inline]
    fn delay_us(&mut self, us: u64) {
        thread::sleep(Duration::from_micros(us));
    }
}
