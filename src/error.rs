#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Parse(#[from] pest::error::Error<crate::common::PestRule>),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Notify(#[from] notify_rust::error::Error),
    #[error(transparent)]
    Xdg(#[from] xdg::BaseDirectoriesError),
    #[error("no handler defined for this mime/extension")]
    NotFound(String),
    #[error("could not figure out the mime type .{0}")]
    Ambiguous(String),
    #[error(transparent)]
    BadMimeType(#[from] mime::FromStrError),
    #[error("Malformed desktop entry")]
    BadEntry(std::path::PathBuf),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
