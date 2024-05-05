use embedded_hal::digital::ErrorKind;
use hidapi::HidError;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum PinError {
    #[error("USB HID error.")]
    ErrorUSB(HidError),
}

impl embedded_hal::digital::Error for PinError {
    fn kind(&self) -> ErrorKind {
        match self {
            PinError::ErrorUSB(_) => ErrorKind::Other,
        }
    }
}
