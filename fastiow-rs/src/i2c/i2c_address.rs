use crate::I2CAddressError;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct I2CAddress {
    address: u8,
}

impl fmt::Display for I2CAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl I2CAddress {
    pub const fn new(address: u8) -> Result<I2CAddress, I2CAddressError> {
        if address > 127 {
            return Err(I2CAddressError::NotA7BitAddress);
        }

        match address > 0 && !(address >= 0x78 && address <= 0x7F) {
            true => Ok(I2CAddress { address }),
            false => Err(I2CAddressError::ReservedAddress),
        }
    }

    pub const fn to_inner(&self) -> u8 {
        self.address
    }
}
