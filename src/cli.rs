use crate::common::{Handler, MimeOrExtension, UserPath};

#[derive(clap::Clap)]
#[clap(global_setting = clap::AppSettings::DeriveDisplayOrder)]
#[clap(global_setting = clap::AppSettings::DisableHelpSubcommand)]
#[clap(version = clap::crate_version!())]
pub enum Cmd {
    /// List default apps and the associated handlers
    List {
        #[clap(long, short)]
        all: bool,
    },

    /// Open a path/URL with its default handler
    Open {
        #[clap(required = true)]
        paths: Vec<UserPath>,
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
