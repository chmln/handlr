use crate::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub enable_selector: bool,
    pub selector: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            enable_selector: false,
            selector: "rofi -dmenu".to_owned(),
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

        process
            .stdin
            .unwrap()
            .write_all(opts.join("\n").as_bytes())?;

        let mut output = String::with_capacity(24);
        process.stdout.unwrap().read_to_string(&mut output)?;
        let output = output.trim_end().to_owned();

        Ok(output)
    }
}
