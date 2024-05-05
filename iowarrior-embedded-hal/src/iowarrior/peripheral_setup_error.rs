use hidapi::HidError;
use crate::iowarrior::Peripheral;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum PeripheralSetupError {
    #[error("USB HID error.")]
    ErrorUSB(HidError),
    #[error("Hardware is already set up.")]
    AlreadySetup,
    #[error("Required hardware is blocked by other peripheral {0}.")]
    HardwareBlocked(Peripheral),
    #[error("Required pins are blocked by other peripherals.")]
    PinsBlocked(Vec<u8>),
    #[error("Peripheral is not supported by hardware.")]
    NotSupported,
}
