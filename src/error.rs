#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Parse(#[from] pest::error::Error<crate::common::Rule>),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("handler not found")]
    NotFound,
    #[error("Invalid desktop entry")]
    BadCmd,
    #[error("Could not find config dir")]
    NoConfigDir,
    #[error("could not guess mime type")]
    Ambiguous,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
