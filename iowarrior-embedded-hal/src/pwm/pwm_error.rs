use embedded_hal::pwm::ErrorKind;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PWMError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
}

#[cfg(feature = "embedded-hal")]
impl embedded_hal::pwm::Error for PWMError {
    fn kind(&self) -> ErrorKind {
        match self {
            PWMError::IOErrorIOWarrior => ErrorKind::Other,
        }
    }
}
