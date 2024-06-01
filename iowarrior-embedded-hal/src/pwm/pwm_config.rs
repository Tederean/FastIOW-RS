use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PWMConfig {
    pub iow56_config: IOW56PWMConfig,
    pub iow100_config: IOW100PWMConfig,
    pub requested_frequency_hz: u32,
}

impl Default for PWMConfig {
    fn default() -> Self {
        PWMConfig {
            iow56_config: IOW56PWMConfig::One,
            iow100_config: IOW100PWMConfig::One,
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
pub enum IOW100PWMConfig {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

impl fmt::Display for IOW100PWMConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl IOW100PWMConfig {
    #[inline]
    pub(crate) const fn get_value(&self) -> u8 {
        *self as u8
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IOW56PWMConfig {
    One = 1,
    Two = 2,
}

impl fmt::Display for IOW56PWMConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl IOW56PWMConfig {
    #[inline]
    pub(crate) const fn get_value(&self) -> u8 {
        *self as u8
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

    pub fn from_u8(channel: u8) -> PWMChannel {
        match channel {
            1 => PWMChannel::First,
            2 => PWMChannel::Second,
            3 => PWMChannel::Third,
            4 => PWMChannel::Fourth,
            _ => panic!("channel {} is not existing", channel),
        }
    }
}
