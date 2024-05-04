use embedded_hal::pwm::ErrorKind;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PWMError {
    #[error("USB input output error.")]
    IOErrorUSB,
}

impl embedded_hal::pwm::Error for PWMError {
    fn kind(&self) -> ErrorKind {
        match self {
            PWMError::IOErrorUSB => ErrorKind::Other,
        }
    }
}
