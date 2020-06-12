#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ParseEntry(#[from] pest::error::Error<crate::common::EntryRule>),
    #[error(transparent)]
    ParseApps(#[from] pest::error::Error<crate::apps::MimeappsRule>),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Xdg(#[from] xdg::BaseDirectoriesError),
    #[error(transparent)]
    Config(#[from] confy::ConfyError),
    #[error("no handlers found for '{0}'")]
    NotFound(String),
    #[error("could not figure out the mime type of '{0}'")]
    Ambiguous(std::path::PathBuf),
    #[error(transparent)]
    BadMimeType(#[from] mime::FromStrError),
    #[error("malformed desktop entry at {0}")]
    BadEntry(std::path::PathBuf),
    #[error(transparent)]
    MimeDetect(#[from] mime_detective::DetectiveError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
