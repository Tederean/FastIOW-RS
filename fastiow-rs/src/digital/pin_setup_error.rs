use crate::Peripheral;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PinSetupError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
    #[error("GPIO not existing.")]
    PinNotExisting,
    #[error("GPIO already configured.")]
    AlreadySetup,
    #[error("GPIO is blocked by peripheral {0}.")]
    BlockedByPeripheral(Peripheral),
    #[error("GPIOs are not supported by hardware.")]
    NotSupported,
}
