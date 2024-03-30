use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum I2CCommunicationError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
    #[error("I2C input output error.")]
    IOErrorI2C,
    #[error("I2C input output error, arbitration lost.")]
    IOErrorI2CArbitrationLost,
}
