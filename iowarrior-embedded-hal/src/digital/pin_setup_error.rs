use hidapi::HidError;
use crate::iowarrior::Peripheral;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum PinSetupError {
    #[error("USB HID error.")]
    ErrorUSB(HidError),
    #[error("Pin not existing.")]
    PinNotExisting,
    #[error("Pin already configured.")]
    AlreadySetup,
    #[error("Pin is blocked by peripheral {0}.")]
    BlockedByPeripheral(Peripheral),
    #[error("Pins are not supported by hardware.")]
    NotSupported,
}
