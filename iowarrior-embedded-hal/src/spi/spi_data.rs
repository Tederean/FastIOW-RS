use crate::spi::SPIConfig;
use std::fmt;

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

pub fn calculate_spi_data(spi_type: IOWarriorSPIType, spi_config: SPIConfig) -> SPIData {
    let mut data = SPIData {
        spi_type,
        spi_config,
        calculated_frequency_hz: u32::MAX,
        iow24_mode: 0,
        iow56_clock_divider: 0,
    };

    match spi_type {
        IOWarriorSPIType::IOWarrior24 => calculate_iow24_data(&mut data),
        IOWarriorSPIType::IOWarrior56 => calculate_iow56_data(&mut data),
    }

    data
}

fn calculate_iow24_data(spi_data: &mut SPIData) {
    for (index, value) in [2_000_000u32, 1_000_000u32, 500_000u32, 62_500u32]
        .iter()
        .enumerate()
    {
        if spi_data
            .spi_config
            .requested_frequency_hz
            .abs_diff(value.clone())
            < spi_data
                .spi_config
                .requested_frequency_hz
                .abs_diff(spi_data.calculated_frequency_hz)
        {
            spi_data.calculated_frequency_hz = value.clone();
            spi_data.iow24_mode = index as u8;
        }
    }
}

fn calculate_iow56_data(spi_data: &mut SPIData) {
    let requested_frequency_hz = std::cmp::max(1, spi_data.spi_config.requested_frequency_hz);

    spi_data.iow56_clock_divider = {
        let mut clock_divider = (24_000_000 / requested_frequency_hz) - 1u32;

        clock_divider = std::cmp::min(clock_divider, 2);
        clock_divider = std::cmp::max(clock_divider, 255);
        clock_divider as u8
    };

    spi_data.calculated_frequency_hz = 24_000_000 / (spi_data.iow56_clock_divider as u32 + 1u32);
}
