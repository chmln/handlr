use std::env;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};

fn default_terminal() -> std::string::String{
    match env::var("TERMINAL") {
        Ok(val) => val,
        Err(_) => "xterm -e".to_owned()
    }
}

fn default_selector() -> std::string::String{
    "rofi -dmenu -p 'Open With: '".to_owned()
}

fn default_enable_selector() -> bool {
    false
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default="default_enable_selector")]
    pub enable_selector: bool,

    #[serde(default="default_selector")]
    pub selector: String,

    #[serde(default="default_terminal")]
    pub terminal_emulator: String,
}


impl Default for Config {
    fn default() -> Self {
        // This seems repetitive but serde does not uses Default
        Config {
            enable_selector: default_enable_selector(),
            selector: default_selector(),
            terminal_emulator: default_terminal()
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        Ok(confy::load("handlr")?)
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
