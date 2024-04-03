use embedded_hal::digital::ErrorKind;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PinError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
}

#[cfg(feature = "embedded-hal")]
impl embedded_hal::digital::Error for PinError {
    fn kind(&self) -> ErrorKind {
        match self {
            PinError::IOErrorIOWarrior => ErrorKind::Other,
        }
    }
}
