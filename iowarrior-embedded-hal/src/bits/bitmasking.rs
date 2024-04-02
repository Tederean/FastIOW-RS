use crate::bits::bit::Bit;

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
