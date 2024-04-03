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

#[cfg(feature = "embedded-hal")]
impl embedded_hal::i2c::Error for I2CError {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        match self {
            I2CError::IOErrorIOWarrior => embedded_hal::i2c::ErrorKind::Other,
            I2CError::IOErrorI2C => embedded_hal::i2c::ErrorKind::Bus,
            I2CError::IOErrorI2CArbitrationLoss => embedded_hal::i2c::ErrorKind::ArbitrationLoss,
            I2CError::InvalidI2CAddress => embedded_hal::i2c::ErrorKind::Other,
        }
    }
}
