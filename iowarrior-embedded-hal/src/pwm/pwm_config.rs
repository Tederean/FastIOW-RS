use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PWMConfig {
    pub channel_mode: ChannelMode,
    pub requested_frequency_hz: u32,
}

impl fmt::Display for PWMConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ChannelMode {
    Single = 1,
    Dual = 2,
    Triple = 3,
    Quad = 4,
}

impl fmt::Display for ChannelMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for ChannelMode {
    fn default() -> Self {
        ChannelMode::Single
    }
}

impl ChannelMode {
    #[inline]
    pub(crate) const fn get_value(&self) -> u8 {
        *self as u8
    }
}
