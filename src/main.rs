use error::{Error, Result};
use structopt::StructOpt;

mod autocomplete;
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
        #[structopt(long, short)]
        mime: Option<Mime>,
        #[structopt(long, short)]
        ext: Option<String>,
        handler: Handler,
    },
    Unset {
        mime: Mime,
    },
    #[structopt(setting = structopt::clap::AppSettings::Hidden)]
    Autocomplete {
        #[structopt(short)]
        desktop_files: bool,
        #[structopt(short)]
        mimes: bool,
        #[structopt(short)]
        extensions: bool,
    },
}

fn main() -> Result<()> {
    let mut apps = mimeapps::MimeApps::read()?;

    match Cmd::from_args() {
        Cmd::Set { mime, ext, handler } => {
            let mime = match ext {
                Some(extension) => mime_guess::from_ext(&extension)
                    .first_raw()
                    .map(ToOwned::to_owned)
                    .map(Mime)
                    .ok_or(Error::Ambiguous)?,
                None => mime.ok_or(Error::MissingMimeOrExt)?,
            };

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
                let mime = match mime_guess::from_path(&path).first_raw() {
                    Some(mime) => mime,
                    None if std::fs::metadata(&path)?.is_dir() => {
                        "inode/directory"
                    }
                    _ => {
                        return Err(Error::Ambiguous);
                    }
                };
                apps.get_handler(&Mime(mime.to_owned()))?.open(path)?;
            }
        },
        Cmd::List => {
            apps.print()?;
        }
        Cmd::Unset { mime } => {
            apps.remove_handler(&mime)?;
        }
        Cmd::Autocomplete {
            desktop_files,
            mimes,
            extensions,
        } => {
            if desktop_files {
                apps.list_handlers()?;
            } else if mimes {
                autocomplete::mimes()?;
            } else if extensions {
                autocomplete::extensions()?;
            }
        }
    };

    Ok(())
}
