use crate::Result;
use once_cell::sync::Lazy;
use xdg_mime::SharedMimeInfo;

pub static SHARED_MIME_DB: Lazy<SharedMimeInfo> =
    Lazy::new(SharedMimeInfo::new);

static CUSTOM_MIMES: &[&'static str] = &[
    "inode/directory",
    "x-scheme-handler/http",
    "x-scheme-handler/https",
];

pub fn autocomplete() -> Result<()> {
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
