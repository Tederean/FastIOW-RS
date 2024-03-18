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

impl Bit {
    const fn get_value(&self) -> u8 {
        *self as u8
    }
}

impl fmt::Display for Bit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait Bitmasking {
    fn set_bit(&mut self, bit: Bit, value: bool);

    fn get_bit(self, bit: Bit) -> bool;
}

impl Bitmasking for std::primitive::u8 {
    fn set_bit(&mut self, bit: Bit, value: bool) {
        if value {
            *self |= 0x01 << bit.get_value();
        } else {
            *self &= !(0x01 << bit.get_value());
        }
    }

    fn get_bit(self, bit: Bit) -> bool {
        ((self >> bit.get_value()) & 0b0000_0001) > 0
    }
}
