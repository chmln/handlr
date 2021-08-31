use config::CONFIG;
use error::{Error, Result};
use once_cell::sync::Lazy;
use std::{
    ffi::OsStr,
    ffi::OsString,
    path::Path,
};

mod apps;
mod cli;
mod common;
mod config;
mod error;
mod utils;

fn main() -> Result<()> {
    use clap::Clap;
    use cli::Cmd;
    use common::Handler;
    use std::collections::HashMap;

    // create config if it doesn't exist
    Lazy::force(&CONFIG);

    let mut apps = (*apps::APPS).clone();

    let mut args = std::env::args_os().collect::<Vec<OsString>>();
    let cmd_name = Path::new(&args[0]).file_stem();

    // If the program is invoked with name `xdg-open` (via symlink xdg-open -> handlr),
    // rewrite arguments to `handlr open ...`.
    if cmd_name == Some(OsStr::new("xdg-open")) {
        args[0] = clap::crate_name!().into();  // fix program name in help message
        args.insert(1, "open".into());
    }

    let res = || -> Result<()> {
        match Cmd::parse_from(args) {
            Cmd::Set { mime, handler } => {
                apps.set_handler(mime.0, handler);
                apps.save()?;
            }
            Cmd::Add { mime, handler } => {
                apps.add_handler(mime.0, handler);
                apps.save()?;
            }
            Cmd::Launch { mime, args } => {
                apps.get_handler(&mime.0)?.launch(args.into_iter().map(|a| a.to_string()).collect())?;
            }
            Cmd::Get { mime, json } => {
                apps.show_handler(&mime.0, json)?;
            }
            Cmd::Open { paths } => {
                let mut handlers: HashMap<Handler, Vec<String>> =
                    HashMap::new();

                for path in paths.into_iter() {
                    handlers
                        .entry(apps.get_handler(&path.get_mime()?.0)?)
                        .or_default()
                        .push(path.to_string());
                }

                for (handler, paths) in handlers.into_iter() {
                    handler.open(paths)?;
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
