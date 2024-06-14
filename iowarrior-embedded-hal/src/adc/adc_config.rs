use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ADCConfig {
    pub iow28_iow100_config: IOW28IOW100ADCConfig,
    pub iow56_config: IOW56ADCConfig,
}

impl fmt::Display for ADCConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for ADCConfig {
    fn default() -> Self {
        ADCConfig {
            iow28_iow100_config: IOW28IOW100ADCConfig::One(SampleRate1ch::TenKhz),
            iow56_config: IOW56ADCConfig::One,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IOW28IOW100ADCConfig {
    One(SampleRate1ch),
    Two(SampleRate2ch),
    Four(SampleRate4ch),
}

impl fmt::Display for IOW28IOW100ADCConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IOW56ADCConfig {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

impl fmt::Display for IOW56ADCConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SampleRate1ch {
    OneKhz = 0,
    TwoKhz = 1,
    ThreeKhz = 2,
    FourKhz = 3,
    SixKhz = 4,
    EightKhz = 5,
    TenKhz = 6,
    TwelfthKhz = 7,
    FifteenKhz = 8,
    SixteenKhz = 9,
    TwentyKhz = 10,
    TwentyfourKhz = 11,
    ThirtyKhz = 13,
}

impl fmt::Display for SampleRate1ch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SampleRate1ch {
    #[inline]
    pub(crate) const fn get_value(&self) -> u8 {
        *self as u8
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SampleRate2ch {
    OneKhz = 0,
    TwoKhz = 1,
    ThreeKhz = 2,
    FourKhz = 3,
    SixKhz = 4,
    EightKhz = 5,
    TenKhz = 6,
    TwelfthKhz = 7,
    FifteenKhz = 8,
}

impl fmt::Display for SampleRate2ch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SampleRate2ch {
    #[inline]
    pub(crate) const fn get_value(&self) -> u8 {
        *self as u8
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SampleRate4ch {
    OneKhz = 0,
    TwoKhz = 1,
    ThreeKhz = 2,
    FourKhz = 3,
    SixKhz = 4,
}

impl fmt::Display for SampleRate4ch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SampleRate4ch {
    #[inline]
    pub(crate) const fn get_value(&self) -> u8 {
        *self as u8
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ADCChannel {
    First = 1,
    Second = 2,
    Third = 3,
    Fourth = 4,
    Fifth = 5,
    Sixth = 6,
    Seventh = 7,
    Eighth = 8,
}

impl fmt::Display for ADCChannel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ADCChannel {
    #[inline]
    pub(crate) const fn get_value(&self) -> u8 {
        *self as u8
    }

    #[inline]
    pub fn from_u8(channel: u8) -> ADCChannel {
        match channel {
            1 => ADCChannel::First,
            2 => ADCChannel::Second,
            3 => ADCChannel::Third,
            4 => ADCChannel::Fourth,
            5 => ADCChannel::Fifth,
            6 => ADCChannel::Sixth,
            7 => ADCChannel::Seventh,
            8 => ADCChannel::Eighth,
            _ => panic!("Channel {} is not existing", channel),
        }
    }
}
