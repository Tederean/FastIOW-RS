use crate::spi::SPIConfig;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IOWarriorSPIType {
    IOWarrior24,
    IOWarrior56,
}

impl fmt::Display for IOWarriorSPIType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SPIData {
    pub spi_type: IOWarriorSPIType,
    pub spi_config: SPIConfig,
    pub calculated_frequency_hz: u32,
    pub iow24_mode: u8,
    pub iow56_clock_divider: u8,
}

impl fmt::Display for SPIData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
