use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SPIConfig {
    pub polarity: embedded_hal::spi::Polarity,
    pub phase: embedded_hal::spi::Phase,
    pub use_data_ready_pin: bool,
    pub requested_frequency_hz: u32,
    pub dummy_value: u8,
}

impl fmt::Display for SPIConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
