use crate::Peripheral;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PeripheralSetupError {
    #[error("USB input output error.")]
    IOErrorUSB,
    #[error("Hardware is already set up.")]
    AlreadySetup,
    #[error("Required hardware is blocked by other peripheral {0}.")]
    HardwareBlocked(Peripheral),
    #[error("Required pins are blocked by other peripherals.")]
    PinsBlocked(Vec<u8>),
    #[error("Peripheral is not supported by hardware.")]
    NotSupported,
}
