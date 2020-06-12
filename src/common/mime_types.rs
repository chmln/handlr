use crate::{Error, Result};
use mime::Mime;
use mime_detective::MimeDetective;
use std::{
    convert::TryFrom,
    path::{Path, PathBuf},
    str::FromStr,
};

// A mime derived from a path or URL
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MimeType(pub Mime);

impl TryFrom<&str> for MimeType {
    type Error = Error;

    fn try_from(arg: &str) -> Result<Self> {
        if let Ok(url) = url::Url::parse(arg) {
            Ok(Self(
                format!("x-scheme-handler/{}", url.scheme())
                    .parse::<Mime>()
                    .unwrap(),
            ))
        } else {
            Self::try_from(&*PathBuf::from(arg))
        }
    }
}

impl TryFrom<&Path> for MimeType {
    type Error = Error;
    fn try_from(path: &Path) -> Result<Self> {
        match MimeDetective::new()?.detect_filepath(path)? {
            guess if guess == mime::APPLICATION_OCTET_STREAM => {
                Err(Error::Ambiguous(path.to_string_lossy().into()))
            }
            guess => Ok(Self(guess)),
        }
    }
}

// Mime derived from user input: extension(.pdf) or type like image/jpg
#[derive(Debug)]
pub struct MimeOrExtension(pub Mime);
impl FromStr for MimeOrExtension {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        if s.starts_with(".") {
            Ok(Self(MimeType::try_from(s)?.0))
        } else {
            Ok(Self(Mime::from_str(s)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_input() {
        "image/jpg".parse::<MimeOrExtension>().unwrap();
        ".jpg".parse::<MimeOrExtension>().unwrap();
        "image//jpg".parse::<MimeOrExtension>().unwrap_err();
        "image".parse::<MimeOrExtension>().unwrap_err();
    }

    #[test]
    fn from_path_with_extension() {
        assert_eq!(
            MimeType::try_from(".pdf").unwrap().0,
            mime::APPLICATION_PDF
        );
        assert_eq!(
            MimeType::try_from(".").unwrap().0.essence_str(),
            "inode/directory"
        );
    }
}
