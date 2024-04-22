use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SPIConfig {
    pub mode: SPIMode,
    pub use_data_ready_pin: bool,
    pub requested_frequency_hz: u32,
    pub dummy_value: u8,
}

impl Default for SPIConfig {
    fn default() -> Self {
        SPIConfig {
            mode: SPIMode::Mode0,
            use_data_ready_pin: false,
            requested_frequency_hz: 1_000_000,
            dummy_value: 0x00,
        }
    }
}

impl fmt::Display for SPIConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SPIMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

impl fmt::Display for SPIMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}