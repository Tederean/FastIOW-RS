use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SPIError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
}

#[cfg(feature = "embedded-hal")]
impl embedded_hal::spi::Error for SPIError {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        match self {
            SPIError::IOErrorIOWarrior => embedded_hal::spi::ErrorKind::Other,
        }
    }
}
