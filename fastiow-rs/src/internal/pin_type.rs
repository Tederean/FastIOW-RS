use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PinType {
    Input = 0,
    Output = 1,
}

impl fmt::Display for PinType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl PinType {
    pub fn get_value(&self) -> u8 {
        *self as u8
    }
}
