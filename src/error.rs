#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Parse(#[from] pest::error::Error<crate::common::Rule>),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no handler defined for this mime/extension")]
    NotFound,
    #[error("badly-formatted desktop entry")]
    BadCmd,
    #[error("could not locate config dir")]
    NoConfigDir,
    #[error("could not figure out the mime type")]
    Ambiguous,
    #[error("could not figure out the mime type from extension .{0}")]
    UnknownExtension(String),
    #[error("invalid mime {0}")]
    BadMime(String),
    #[error("either mime (via -m) or extension (via -e) must be provided")]
    MissingMimeOrExt,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
