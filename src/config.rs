use crate::{apps::SystemApps, common::Handler, Error, Result};
use mime::Mime;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub static CONFIG: Lazy<Config> = Lazy::new(Config::load);

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub enable_selector: bool,
    pub selector: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            enable_selector: false,
            selector: "rofi -dmenu -p 'Open With: '".into(),
        }
    }
}

impl Config {
    pub fn terminal() -> Result<String> {
        let terminal_entry = crate::apps::APPS
            .get_handler(&Mime::from_str("x-scheme-handler/terminal").unwrap())
            .ok()
            .map(|h| h.get_entry().ok())
            .flatten();

        terminal_entry
            .or_else(|| {
                let entry = SystemApps::get_entries()
                    .ok()?
                    .find(|(_handler, entry)| {
                        entry.categories.contains_key("TerminalEmulator")
                    })
                    .map(|e| e.clone())?;

                crate::utils::notify(
                    "handlr",
                    &format!(
                        "Guessed terminal emulator: {}.\n\nIf this is wrong, use `handlr set x-scheme-handler/terminal` to update it.",
                        entry.0.to_string_lossy()
                    )
                ).ok()?;

                let mut apps = (*crate::apps::APPS).clone();
                apps.set_handler(
                    Mime::from_str("x-scheme-handler/terminal").unwrap(),
                    Handler::assume_valid(entry.0),
                );
                apps.save().ok()?;

                Some(entry.1)
            })
            .map(|e| e.exec)
            .ok_or(Error::NoTerminal)
    }
    pub fn load() -> Self {
        confy::load("handlr").unwrap()
    }

    pub fn select<O: Iterator<Item = String>>(
        &self,
        mut opts: O,
    ) -> Result<String> {
        use itertools::Itertools;
        use std::{
            io::prelude::*,
            process::{Command, Stdio},
        };

        let process = {
            let mut split = shlex::split(&self.selector).unwrap();
            let (cmd, args) = (split.remove(0), split);
            Command::new(cmd)
                .args(args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?
        };

        let output = {
            process
                .stdin
                .ok_or(Error::Selector(self.selector.clone()))?
                .write_all(opts.join("\n").as_bytes())?;

            let mut output = String::with_capacity(24);

            process
                .stdout
                .ok_or(Error::Selector(self.selector.clone()))?
                .read_to_string(&mut output)?;

            output.trim_end().to_owned()
        };

        if output.is_empty() {
            Err(Error::Cancelled)
        } else {
            Ok(output)
        }
    }
}
