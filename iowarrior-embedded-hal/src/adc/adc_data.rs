use crate::adc::ADCConfig;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ADCData {
    pub adc_type: IOWarriorADCType,
    pub adc_config: ADCConfig,
    pub resolution_bits: u8,
    pub report_sample_count: u8,
    pub max_channel_value: u8,
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
