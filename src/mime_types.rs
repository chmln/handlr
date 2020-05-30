use crate::{Error, Mime, Result};
use once_cell::sync::Lazy;
use std::path::Path;
use xdg_mime::SharedMimeInfo;

static SHARED_MIME_DB: Lazy<SharedMimeInfo> = Lazy::new(SharedMimeInfo::new);

static CUSTOM_MIMES: &[&'static str] = &[
    "inode/directory",
    "x-scheme-handler/http",
    "x-scheme-handler/https",
];

pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Mime> {
    let guess = SHARED_MIME_DB.guess_mime_type().path(path).guess();

    match guess.mime_type().essence_str() {
        "application/octet-stream" => Err(Error::Ambiguous),
        mime => Ok(Mime(mime.to_owned())),
    }
}

pub fn verify(mime: &str) -> Result<&str> {
    if mime.starts_with("x-scheme-handler/") || CUSTOM_MIMES.contains(&mime) {
        return Ok(mime);
    }

    mime_db::TYPES
        .iter()
        .find(|(m, _, _)| m == &mime)
        .ok_or(Error::BadMime(mime.to_owned()))?;

    Ok(mime)
}

pub fn list() -> Result<()> {
    use std::io::Write;

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    mime_db::EXTENSIONS.iter().for_each(|(ext, _)| {
        stdout.write_all(b".").unwrap();
        stdout.write_all(ext.as_bytes()).unwrap();
        stdout.write_all(b"\n").unwrap();
    });

    CUSTOM_MIMES.iter().for_each(|mime| {
        stdout.write_all(mime.as_bytes()).unwrap();
        stdout.write_all(b"\n").unwrap();
    });

    mime_db::TYPES.iter().for_each(|(mime, _, _)| {
        stdout.write_all(mime.as_bytes()).unwrap();
        stdout.write_all(b"\n").unwrap();
    });

    Ok(())
}
