use config::CONFIG;
use error::{Error, Result};
use once_cell::sync::Lazy;

mod apps;
mod cli;
mod common;
mod config;
mod error;
mod utils;

fn main() -> Result<()> {
    use clap::Clap;
    use cli::Cmd;
    use common::{Handler, MimeType};
    use std::{collections::HashMap, convert::TryFrom};

    // create config if it doesn't exist
    Lazy::force(&CONFIG);

    let mut apps = (*apps::APPS).clone();

    let res = || -> Result<()> {
        match Cmd::parse() {
            Cmd::Set { mime, handler } => {
                apps.set_handler(mime.0, handler);
                apps.save()?;
            }
            Cmd::Add { mime, handler } => {
                apps.add_handler(mime.0, handler);
                apps.save()?;
            }
            Cmd::Launch { mime, args } => {
                apps.get_handler(&mime.0)?.launch(args)?;
            }
            Cmd::Get { mime, json } => {
                apps.show_handler(&mime.0, json)?;
            }
            Cmd::Open { paths } => {
                let mut handlers: HashMap<Handler, Vec<String>> =
                    HashMap::new();
                for path in paths.into_iter() {
                    let mime = MimeType::try_from(path.as_str())?.0;
                    let handler = apps.get_handler(&mime)?;
                    handlers.entry(handler).or_insert(Vec::new()).push(path);
                }
                for (handler, handled_paths) in handlers.into_iter() {
                    handler.open(handled_paths)?;
                }
            }
            Cmd::List { all } => {
                apps.print(all)?;
            }
            Cmd::Unset { mime } => {
                apps.remove_handler(&mime.0)?;
            }
            Cmd::Autocomplete {
                desktop_files,
                mimes,
            } => {
                if desktop_files {
                    apps::MimeApps::list_handlers()?;
                } else if mimes {
                    common::db_autocomplete()?;
                }
            }
        }
        Ok(())
    }();

    match (res, atty::is(atty::Stream::Stdout)) {
        (Err(e), _) if matches!(e, Error::Cancelled) => {
            std::process::exit(1);
        }
        (Err(e), true) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
        (Err(e), false) => {
            utils::notify("handlr error", &e.to_string())?;
            std::process::exit(1);
        }
        _ => Ok(()),
    }
}
