use embedded_hal::i2c::ErrorKind;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum I2CError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
    #[error("Invalid 7 bit I2C address.")]
    InvalidI2CAddress,
    #[error("I2C input output error.")]
    IOErrorI2C,
    #[error("I2C input output error, arbitration lost.")]
    IOErrorI2CArbitrationLoss,
}

impl embedded_hal::i2c::Error for I2CError {
    fn kind(&self) -> ErrorKind {
        match self {
            I2CError::IOErrorIOWarrior => ErrorKind::Other,
            I2CError::IOErrorI2C => ErrorKind::Bus,
            I2CError::IOErrorI2CArbitrationLoss => ErrorKind::ArbitrationLoss,
            I2CError::InvalidI2CAddress => ErrorKind::Other,
        }
    }
}
