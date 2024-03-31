use embedded_hal::spi::ErrorKind;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SPIError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
}

impl embedded_hal::spi::Error for SPIError {
    fn kind(&self) -> ErrorKind {
        match self {
            SPIError::IOErrorIOWarrior => ErrorKind::Other,
        }
    }
}
