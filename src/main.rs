use clap::Clap;
use error::{Error, Result};
use notify_rust::Notification;
use std::convert::TryFrom;

mod apps;
mod common;
mod error;

use common::{DesktopEntry, FlexibleMime, Handler, MimeOrExtension};

#[derive(Clap)]
#[clap(global_setting = clap::AppSettings::DeriveDisplayOrder)]
#[clap(global_setting = clap::AppSettings::DisableHelpSubcommand)]
#[clap(version = clap::crate_version!())]
enum Cmd {
    /// List default apps and the associated handlers
    List,

    /// Open a path/URL with its default handler
    Open {
        #[clap(required = true)]
        path: Vec<String>,
    },

    /// Set the default handler for mime/extension
    Set {
        mime: MimeOrExtension,
        handler: Handler,
    },

    /// Unset the default handler for mime/extension
    Unset { mime: MimeOrExtension },

    /// Launch the handler for specified extension/mime with optional arguments
    Launch {
        mime: MimeOrExtension,
        args: Vec<String>,
    },

    /// Get handler for this mime/extension
    Get {
        #[clap(long)]
        json: bool,
        mime: MimeOrExtension,
    },

    /// Add a handler for given mime/extension
    /// Note that the first handler is the default
    Add {
        mime: MimeOrExtension,
        handler: Handler,
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
    let mut apps = apps::MimeApps::read()?;

    let res = || -> Result<()> {
        match Cmd::parse() {
            Cmd::Set { mime, handler } => {
                apps.set_handler(mime.0, handler)?;
            }
            Cmd::Add { mime, handler } => {
                apps.add_handler(mime.0, handler)?;
            }
            Cmd::Launch { mime, args } => {
                apps.get_handler(&mime.0)?.launch(args)?;
            }
            Cmd::Get { mime, json } => {
                apps.show_handler(&mime.0, json)?;
            }
            Cmd::Open { path } => {
                std::process::Command::new("notify-send")
                    .arg(&format!("{:?}", path))
                    .spawn()?;
                apps.get_handler(
                    &FlexibleMime::try_from(path.get(0).unwrap().as_str())?.0,
                )?
                .launch(path)?;
            }
            Cmd::List => {
                apps.print()?;
            }
            Cmd::Unset { mime } => {
                apps.remove_handler(&mime.0)?;
            }
            Cmd::Autocomplete {
                desktop_files,
                mimes,
            } => {
                if desktop_files {
                    apps.list_handlers()?;
                } else if mimes {
                    common::db_autocomplete()?;
                }
            }
        }
        Ok(())
    }();

    match (res, atty::is(atty::Stream::Stdout)) {
        (Err(e), true) => eprintln!("{}", e),
        (Err(e), false) => {
            Notification::new()
                .summary("handlr error")
                .body(&e.to_string())
                .show()?;
        }
        _ => {}
    };
    Ok(())
}
