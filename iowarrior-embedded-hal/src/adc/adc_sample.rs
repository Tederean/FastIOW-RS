use crate::adc::ADCChannel;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ADCSample {
    pub channel: ADCChannel,
    pub value: u16,
}

impl ADCSample {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
