use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IowkitError {
    #[error("USB input output error.")]
    IOErrorUSB,
}
