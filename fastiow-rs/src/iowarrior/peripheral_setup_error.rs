use crate::Peripheral;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PeripheralSetupError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
    #[error("Hardware is already set up.")]
    AlreadySetup,
    #[error("Hardware is blocked by other peripheral {0}.")]
    BlockedByOtherPeripheral(Peripheral),
    #[error("Hardware is blocked by pin {0}.")]
    BlockedByGpio(u8),
    #[error("Peripheral is not supported by hardware.")]
    NotSupported,
}
