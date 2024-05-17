use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PWMConfig {
    pub channel_mode: PWMChannel,
    pub requested_frequency_hz: u32,
}

impl Default for PWMConfig {
    fn default() -> Self {
        PWMConfig {
            channel_mode: PWMChannel::First,
            requested_frequency_hz: 1_000,
        }
    }
}

impl fmt::Display for PWMConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PWMChannel {
    First = 1,
    Second = 2,
    Third = 3,
    Fourth = 4,
}

impl fmt::Display for PWMChannel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl PWMChannel {
    #[inline]
    pub(crate) const fn get_value(&self) -> u8 {
        *self as u8
    }
}
