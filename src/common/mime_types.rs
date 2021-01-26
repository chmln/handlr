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
            [guess, ..] => Ok(guess.clone()),
            [] => unreachable!(),
        }
    }
}

impl TryFrom<&str> for MimeType {
    type Error = Error;

    fn try_from(arg: &str) -> Result<Self> {
        match url::Url::parse(arg) {
            Ok(url) if url.scheme() == "file" => {
                Self::try_from(&*PathBuf::from(url.path()))
            }
            Ok(url) => Ok(Self(
                format!("x-scheme-handler/{}", url.scheme())
                    .parse::<Mime>()
                    .unwrap(),
            )),
            Err(_) => Self::try_from(&*PathBuf::from(arg)),
        }
    }
}

fn mime_to_option(mime: Mime) -> Option<Mime> {
    if mime == mime::APPLICATION_OCTET_STREAM {
        None
    } else {
        Some(mime)
    }
}

impl TryFrom<&Path> for MimeType {
    type Error = Error;
    fn try_from(path: &Path) -> Result<Self> {
        let db = xdg_mime::SharedMimeInfo::new();

        let guess = db.guess_mime_type().path(&path).guess();

        let mime = mime_to_option(guess.mime_type().clone())
            .ok_or_else(|| Error::Ambiguous(path.to_owned()))?;

        Ok(Self(mime))
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
        assert_eq!(
            MimeType::try_from("./tests/SettingsWidgetFdoSecrets.ui")?.0,
            "application/x-designer"
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
