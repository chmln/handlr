use anyhow::Result;
use structopt::StructOpt;

mod common;
mod mimeapps;

pub use common::{DesktopEntry, Handler, Mime};

#[derive(StructOpt)]
enum Options {
    List,
    Open { path: String },
    Get { mime: Mime },
    Set { mime: Mime, handler: Handler },
}

fn main() -> Result<()> {
    let cmd = Options::from_args();

    let mut user = mimeapps::MimeApps::read()?;

    match cmd {
        Options::Set { mime, handler } => {
            user.set_handler(mime, handler)?;
        }
        Options::Get { mime } => {
            println!("{}", user.get_handler(&mime)?);
        }
        Options::Open { path } => match url::Url::parse(&path) {
            Ok(url) => {
                let mime = Mime(format!("x-scheme-handler/{}", url.scheme()));
                user.get_handler(&mime)?.run(&path)?;
            }
            Err(_) => {
                let guess = mime_guess::from_path(&path)
                    .first_raw()
                    .ok_or_else(|| anyhow::Error::msg("Could not determine mime type"))?;
                user.get_handler(&Mime(guess.to_owned()))?.run(&path)?;
            }
        },
        _ => {}
    };

    Ok(())
}
