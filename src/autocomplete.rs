use crate::Result;
pub fn extensions() -> Result<()> {
    use std::io::Write;

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    mime_db::EXTENSIONS.iter().for_each(|(ext, _)| {
        stdout.write_all(ext.as_bytes()).unwrap();
        stdout.write_all(b"\n").unwrap();
    });

    Ok(())
}

pub fn mimes() -> Result<()> {
    use std::io::Write;

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    mime_db::TYPES.iter().for_each(|(mime, _, _)| {
        stdout.write_all(mime.as_bytes()).unwrap();
        stdout.write_all(b"\n").unwrap();
    });

    Ok(())
}
