use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CommunicationError {
    #[error("USB input output error.")]
    IOErrorUSB,
}
