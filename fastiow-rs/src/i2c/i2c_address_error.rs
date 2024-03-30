use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum I2CAddressError {
    #[error("Address is to large for a valid 7 bit I2C address.")]
    NotA7BitAddress,
    #[error("Reserved I2C addresses are not allowed.")]
    ReservedAddress,
}
