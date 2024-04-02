use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct I2CConfig {
    pub iow56_clock: IOW56Clock,
    pub iow100_speed: IOW100Speed,
}

impl fmt::Display for I2CConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IOW56Clock {
    Slow46kHz = 2,
    Standard93kHz = 0,
    Fast375kHz = 1,
}

impl fmt::Display for IOW56Clock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for IOW56Clock {
    fn default() -> Self {
        IOW56Clock::Standard93kHz
    }
}

impl IOW56Clock {
    #[inline]
    pub(crate) const fn get_value(&self) -> u8 {
        *self as u8
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IOW100Speed {
    ExtraSlow10kb = 3,
    Slow50kb = 2,
    Standard100kb = 0,
    Fast400kb = 1,
    FastPlus1000kb = 4,
}

impl fmt::Display for IOW100Speed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for IOW100Speed {
    fn default() -> Self {
        IOW100Speed::Standard100kb
    }
}

impl IOW100Speed {
    #[inline]
    pub(crate) const fn get_value(&self) -> u8 {
        *self as u8
    }
}
