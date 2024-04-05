use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SPIError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
    #[error("SPI input output error.")]
    IOErrorSPI,
}

impl embedded_hal::spi::Error for SPIError {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        match self {
            SPIError::IOErrorIOWarrior | SPIError::IOErrorSPI => {
                embedded_hal::spi::ErrorKind::Other
            }
        }
    }
}
