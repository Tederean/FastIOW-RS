use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Bit {
    Bit0 = 0,
    Bit1 = 1,
    Bit2 = 2,
    Bit3 = 3,
    Bit4 = 4,
    Bit5 = 5,
    Bit6 = 6,
    Bit7 = 7,
}

impl fmt::Display for Bit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Bit {
    #[inline]
    pub fn from(bit_index: u8) -> Bit {
        match bit_index {
            0 => Bit::Bit0,
            1 => Bit::Bit1,
            2 => Bit::Bit2,
            3 => Bit::Bit3,
            4 => Bit::Bit4,
            5 => Bit::Bit5,
            6 => Bit::Bit6,
            7 => Bit::Bit7,
            _ => panic!("bit index {} is out of bounds for u8", bit_index),
        }
    }

    #[inline]
    pub const fn get_value(&self) -> u8 {
        *self as u8
    }
}
