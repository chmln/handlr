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
                Err(Error::Ambiguous(path.to_owned()))
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
        let mime = if s.starts_with(".") {
            mime_db::lookup(&s[1..])
                .ok_or(Error::Ambiguous(s.into()))?
                .parse::<Mime>()
                .unwrap()
        } else {
            Mime::from_str(s)?
        };

        Ok(Self(mime))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_input() -> Result<()> {
        assert_eq!(MimeOrExtension::from_str(".pdf")?.0, mime::APPLICATION_PDF);
        assert_eq!(
            MimeOrExtension::from_str("image/jpeg")?.0,
            mime::IMAGE_JPEG
        );

        "image//jpg".parse::<MimeOrExtension>().unwrap_err();
        "image".parse::<MimeOrExtension>().unwrap_err();

        Ok(())
    }

    #[test]
    fn from_path() -> Result<()> {
        assert_eq!(MimeType::try_from(".")?.0.essence_str(), "inode/directory");
        assert_eq!(MimeType::try_from("./tests/cat")?.0.type_(), "text");
        assert_eq!(MimeType::try_from("./tests/rust.vim")?.0.type_(), "text");

        Ok(())
    }
}
