use error::{Error, Result};
use structopt::StructOpt;

mod common;
mod error;
mod mimeapps;

pub use common::{DesktopEntry, Handler, Mime};

#[derive(StructOpt)]
enum Cmd {
    List,
    Open {
        path: String,
    },
    Get {
        #[structopt(long)]
        json: bool,
        mime: Mime,
    },
    Launch {
        mime: Mime,
        args: Vec<String>,
    },
    Set {
        mime: Mime,
        handler: Handler,
    },
    Unset {
        mime: Mime
    }
}

fn main() -> Result<()> {
    let mut apps = mimeapps::MimeApps::read()?;

    match Cmd::from_args() {
        Cmd::Set { mime, handler } => {
            apps.set_handler(mime, handler)?;
        }
        Cmd::Launch { mime, args } => {
            apps.get_handler(&mime)?.launch(args)?;
        }
        Cmd::Get { mime, json } => {
            apps.show_handler(&mime, json)?;
        }
        Cmd::Open { path } => match url::Url::parse(&path) {
            Ok(url) => {
                let mime = Mime(format!("x-scheme-handler/{}", url.scheme()));
                apps.get_handler(&mime)?.open(path)?;
            }
            Err(_) => {
                let guess = mime_guess::from_path(&path)
                    .first_raw()
                    .ok_or(Error::Ambiguous)?;
                apps.get_handler(&Mime(guess.to_owned()))?.open(path)?;
            }
        },
        Cmd::List => {
            apps.print()?;
        },
        Cmd::Unset { mime } => {
            apps.remove_handler(&mime)?;
        }
    };

    Ok(())
}
