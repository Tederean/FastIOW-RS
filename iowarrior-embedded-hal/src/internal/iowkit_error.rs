use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IowkitError {
    #[error("IOWarrior input output error.")]
    IOErrorIOWarrior,
}
