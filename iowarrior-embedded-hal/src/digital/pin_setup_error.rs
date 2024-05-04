use crate::iowarrior::Peripheral;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PinSetupError {
    #[error("USB input output error.")]
    IOErrorUSB,
    #[error("Pin not existing.")]
    PinNotExisting,
    #[error("Pin already configured.")]
    AlreadySetup,
    #[error("Pin is blocked by peripheral {0}.")]
    BlockedByPeripheral(Peripheral),
    #[error("Pins are not supported by hardware.")]
    NotSupported,
}
