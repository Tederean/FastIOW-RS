use hidapi::HidError;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum SPIError {
    #[error("USB HID error.")]
    ErrorUSB(HidError),
    #[error("SPI input output error.")]
    IOErrorSPI,
}

impl embedded_hal::spi::Error for SPIError {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        match self {
            SPIError::ErrorUSB(_) | SPIError::IOErrorSPI => embedded_hal::spi::ErrorKind::Other,
        }
    }
}
