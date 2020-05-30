use clap::Clap;
use error::{Error, Result};

mod common;
mod error;
mod mime_types;
mod mimeapps;

pub use common::{DesktopEntry, Handler, Mime};

#[derive(Clap)]
enum Cmd {
    List,
    Open {
        path: String,
    },
    Get {
        #[clap(long)]
        json: bool,
        mime: Mime,
    },
    Launch {
        mime: Mime,
        args: Vec<String>,
    },
    Add {
        mime: Mime,
        handler: Handler,
    },
    Set {
        mime: Mime,
        handler: Handler,
    },
    Unset {
        mime: Mime,
    },
    #[clap(setting = clap::AppSettings::Hidden)]
    Autocomplete {
        #[clap(short)]
        desktop_files: bool,
        #[clap(short)]
        mimes: bool,
    },
}

fn main() -> Result<()> {
    let mut apps = mimeapps::MimeApps::read()?;

    match Cmd::parse() {
        Cmd::Set { mime, handler } => {
            apps.set_handler(mime, handler)?;
        }
        Cmd::Add { mime, handler } => {
            apps.add_handler(mime, handler)?;
        }
        Cmd::Launch { mime, args } => {
            apps.get_handler(&mime)?.launch(args)?;
        }
        Cmd::Get { mime, json } => {
            apps.show_handler(&mime, json)?;
        }
        Cmd::Open { path } => {
            apps.get_handler(&Mime::try_from_path(&path)?)?.open(path)?;
        }
        Cmd::List => {
            apps.print()?;
        }
        Cmd::Unset { mime } => {
            apps.remove_handler(&mime)?;
        }
        Cmd::Autocomplete {
            desktop_files,
            mimes,
        } => {
            if desktop_files {
                apps.list_handlers()?;
            } else if mimes {
                mime_types::list()?;
            }
        }
    };

    Ok(())
}
