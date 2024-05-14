use hidapi::HidError;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("USB HID error.")]
    ErrorUSB(HidError),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("IOWarrior not found with serial number: {0}")]
    NotFound(String),
}
