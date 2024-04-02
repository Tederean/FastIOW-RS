use crate::internal::IowkitError;
use embedded_hal::digital::ErrorKind;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PinError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
}

impl embedded_hal::digital::Error for PinError {
    fn kind(&self) -> ErrorKind {
        match self {
            PinError::IOErrorIOWarrior => ErrorKind::Other,
        }
    }
}

pub(crate) fn map_error<T>(result: Result<T, IowkitError>) -> Result<T, PinError> {
    match result {
        Ok(t) => Ok(t),
        Err(error) => match error {
            IowkitError::IOErrorIOWarrior => Err(PinError::IOErrorIOWarrior),
        },
    }
}
