use crate::adc::{ADCChannel, ADCConfig};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct ADCData {
    pub adc_type: IOWarriorADCType,
    pub adc_config: ADCConfig,
    pub resolution_bits: u8,
    pub report_channel_count: u8,
    pub report_samples_count: u8,
    pub highest_enabled_channel: ADCChannel,
    pub sampling_frequency_hz: f32,
}

impl fmt::Display for ADCData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IOWarriorADCType {
    IOWarrior28,
    IOWarrior56,
    IOWarrior100,
}

impl fmt::Display for IOWarriorADCType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
