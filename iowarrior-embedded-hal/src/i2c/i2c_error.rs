use embedded_hal::i2c::NoAcknowledgeSource;
use hidapi::HidError;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum I2CError {
    #[error("USB HID error.")]
    ErrorUSB(HidError),
    #[error("Invalid 7 bit I2C address.")]
    InvalidAddress,
    #[error("I2C slave does not acknowledge command byte.")]
    NoAcknowledge,
    #[error("I2C arbitration lost.")]
    ArbitrationLoss,
    #[error("I2C requested wrong amount of bytes.")]
    WrongAmountOfBytesRequested,
    #[error("I2C transaction without start requested.")]
    TransactionWithoutStartRequested,
    #[error("I2C received NACK.")]
    NackReceived,
    #[error("I2C bus error.")]
    BusError,
}

impl embedded_hal::i2c::Error for I2CError {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        match self {
            I2CError::ErrorUSB(_) => embedded_hal::i2c::ErrorKind::Other,
            I2CError::NoAcknowledge => {
                embedded_hal::i2c::ErrorKind::NoAcknowledge(NoAcknowledgeSource::Unknown)
            }
            I2CError::ArbitrationLoss => embedded_hal::i2c::ErrorKind::ArbitrationLoss,
            I2CError::InvalidAddress => embedded_hal::i2c::ErrorKind::Other,
            I2CError::BusError => embedded_hal::i2c::ErrorKind::Bus,
            I2CError::WrongAmountOfBytesRequested => embedded_hal::i2c::ErrorKind::Other,
            I2CError::TransactionWithoutStartRequested => embedded_hal::i2c::ErrorKind::Bus,
            I2CError::NackReceived => embedded_hal::i2c::ErrorKind::Bus,
        }
    }
}
