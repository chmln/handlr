use crate::{Error, Result};

static CUSTOM_MIMES: &[&'static str] = &[
    "inode/directory",
    "x-scheme-handler/http",
    "x-scheme-handler/https",
];

pub fn lookup_extension(ext: &str) -> Result<&str> {
    mime_db::lookup(ext).ok_or(Error::UnknownExtension(ext.to_owned()))
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
