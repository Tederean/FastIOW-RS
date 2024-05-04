use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SPIError {
    #[error("USB input output error.")]
    IOErrorUSB,
    #[error("SPI input output error.")]
    IOErrorSPI,
}

impl embedded_hal::spi::Error for SPIError {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        match self {
            SPIError::IOErrorUSB | SPIError::IOErrorSPI => embedded_hal::spi::ErrorKind::Other,
        }
    }
}
