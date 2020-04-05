use anyhow::Result;

mod common;
mod mimeapps;
mod systemapps;

pub use common::{DesktopEntry, Mime};

fn main() -> Result<()> {
    let mut args: Vec<_> = std::env::args().collect();
    let user = mimeapps::MimeApps::read()?;
    let sys = systemapps::SystemApps::populate()?;

    let mime = Mime(args.remove(1));

    let user_handler = user.get_handler(&mime);
    match user_handler {
        Some(h) => {
            dbg!(&h);
        }
        None => {
            let s = sys.get_handlers(&mime);
            dbg!(&s);
        }
    };
    Ok(())
}
