use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SerialNumberError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
    #[error("Serialnumber for IOW40 only those with firmware V1.0.1.0 and later.")]
    NotExisting,
}
