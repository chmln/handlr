use crate::{common::SHARED_MIME_DB, Error, Result};
use mime::Mime;
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

fn read_mb(path: &Path) -> Result<(std::fs::Metadata, Vec<u8>)> {
    let metadata = std::fs::metadata(path)?;
    let data = if metadata.len() <= 1024 {
        std::fs::read(path)?
    } else {
        use std::io::prelude::*;
        let mut buffer = Vec::with_capacity(1024);
        let reader = std::io::BufReader::new(std::fs::File::open(path)?);
        reader.take(1024).read_exact(&mut buffer)?;
        buffer
    };
    Ok((metadata, data))
}

impl TryFrom<&Path> for MimeType {
    type Error = Error;
    fn try_from(path: &Path) -> Result<Self> {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        match &*SHARED_MIME_DB.get_mime_types_from_file_name(&file_name) {
            [t, ..] if t == &mime::APPLICATION_OCTET_STREAM => Ok(Self({
                let (meta, data) = read_mb(path)?;
                SHARED_MIME_DB
                    .guess_mime_type()
                    .metadata(meta)
                    .data(&data)
                    .guess()
                    .mime_type()
                    .clone()
            })),
            [other, ..] => Ok(Self(other.clone())),
            _ => Err(Error::Ambiguous(path.to_string_lossy().into())),
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
