use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Pipe {
    IOPins = 0,
    SpecialMode = 1,
    I2CMode = 2,
    ADCMode = 3,
}

impl fmt::Display for Pipe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Pipe {
    #[inline]
    pub fn get_value(&self) -> u8 {
        *self as u8
    }
}
