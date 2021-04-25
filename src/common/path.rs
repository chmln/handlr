use url::Url;

use crate::{common::MimeType, Error, Result};
use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
    path::PathBuf,
    str::FromStr,
};

pub enum UserPath {
    Url(Url),
    File(PathBuf),
}

impl UserPath {
    pub fn get_mime(&self) -> Result<MimeType> {
        match self {
            Self::Url(url) => Ok(url.into()),
            Self::File(f) => MimeType::try_from(f.as_path()),
        }
    }
}

impl FromStr for UserPath {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = match url::Url::parse(&s) {
            Ok(url) if url.scheme() == "file" => {
                let path = url
                    .to_file_path()
                    .map_err(|_| Error::BadPath(url.path().to_owned()))?;

                Self::File(path)
            }
            Ok(url) => Self::Url(url),
            _ => Self::File(PathBuf::from(s)),
        };

        Ok(normalized)
    }
}

impl Display for UserPath {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::File(f) => fmt.write_str(&f.to_string_lossy().to_string()),
            Self::Url(u) => fmt.write_str(&u.to_string()),
        }
    }
}
