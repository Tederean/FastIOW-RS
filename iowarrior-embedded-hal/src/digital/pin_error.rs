use embedded_hal::digital::ErrorKind;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PinError {
    #[error("USB input output error.")]
    IOErrorUSB,
}

impl embedded_hal::digital::Error for PinError {
    fn kind(&self) -> ErrorKind {
        match self {
            PinError::IOErrorUSB => ErrorKind::Other,
        }
    }
}
