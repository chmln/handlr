use crate::{Error, Result};
use mime::Mime;
use std::{
    convert::TryFrom,
    path::{Path, PathBuf},
    str::FromStr,
};

// A mime derived from a path or URL
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MimeType(pub Mime);

impl MimeType {
    fn from_ext(ext: &str) -> Result<Mime> {
        match &*xdg_mime::SharedMimeInfo::new()
            .get_mime_types_from_file_name(ext)
        {
            [m] if m == &mime::APPLICATION_OCTET_STREAM => {
                Err(Error::Ambiguous(ext.into()))
            }
            [] => unreachable!(),
            [guess, ..] => Ok(guess.clone()),
        }
    }
}

impl TryFrom<&str> for MimeType {
    type Error = Error;

    fn try_from(arg: &str) -> Result<Self> {
        if let Ok(url) = url::Url::parse(arg) {
            if url.scheme() == "file" {
                return Self::try_from(url.path())
            }
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
        match xdg_mime::SharedMimeInfo::new()
            .guess_mime_type()
            .path(&path)
            .guess()
            .mime_type()
        {
            guess if guess == &mime::APPLICATION_OCTET_STREAM => {
                Err(Error::Ambiguous(path.to_owned()))
            }
            guess => Ok(Self(guess.clone())),
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
            MimeType::from_ext(s)?
        } else {
            match Mime::from_str(s)? {
                m if m.subtype() == "" => return Err(Error::InvalidMime(m)),
                proper_mime => proper_mime,
            }
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
        assert_eq!(MimeType::try_from("./tests/rust.vim")?.0, "text/plain");
        assert_eq!(
            MimeType::try_from("./tests/cat")?.0,
            "application/x-shellscript"
        );

        Ok(())
    }

    #[test]
    fn from_ext() -> Result<()> {
        assert_eq!(".mp3".parse::<MimeOrExtension>()?.0, "audio/mpeg");
        assert_eq!("audio/mpeg".parse::<MimeOrExtension>()?.0, "audio/mpeg");
        ".".parse::<MimeOrExtension>().unwrap_err();
        "audio/".parse::<MimeOrExtension>().unwrap_err();

        Ok(())
    }
}
