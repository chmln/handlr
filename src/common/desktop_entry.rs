use crate::{Error, Result, CONFIG};
use mime::Mime;
use std::{
    convert::TryFrom,
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DesktopEntry {
    pub(crate) name: String,
    pub(crate) exec: String,
    pub(crate) file_name: OsString,
    pub(crate) term: bool,
    pub(crate) mimes: Vec<Mime>,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Mode {
    Launch,
    Open,
}

impl DesktopEntry {
    pub fn exec(&self, mode: Mode, arguments: Vec<String>) -> Result<()> {
        let supports_multiple =
            self.exec.contains("%F") || self.exec.contains("%U");
        if arguments.is_empty() {
            self.exec_inner(vec![])?
        } else if supports_multiple || mode == Mode::Launch {
            self.exec_inner(arguments)?;
        } else {
            for arg in arguments {
                self.exec_inner(vec![arg])?;
            }
        };

        Ok(())
    }
    fn exec_inner(&self, arg: Vec<String>) -> Result<()> {
        let mut cmd = {
            let (cmd, args) = self.get_cmd(arg)?;
            let mut cmd = Command::new(cmd);
            cmd.args(args);
            cmd
        };

        if self.term {
            cmd.spawn()?;
        } else {
            cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
        };

        Ok(())
    }
    pub fn get_cmd(&self, args: Vec<String>) -> Result<(String, Vec<String>)> {
        let special = regex::Regex::new("%(f|F|u|U)").unwrap();

        let mut split = shlex::split(&self.exec)
            .unwrap()
            .into_iter()
            .flat_map(|s| match s.as_str() {
                "%f" | "%F" | "%u" | "%U" => args.clone(),
                s if special.is_match(s) => vec![special
                    .replace_all(s, args.clone().join(" ").as_str())
                    .into()],
                _ => vec![s],
            })
            .collect::<Vec<_>>();

        // If the entry expects a terminal (emulator), but this process is not running in one, we
        // launch a new one.
        if self.term && !atty::is(atty::Stream::Stdout) {
            split = shlex::split(&CONFIG.terminal_emulator)
                .unwrap()
                .into_iter()
                .chain(split.into_iter())
                .collect();
        }

        Ok((split.remove(0), split))
    }
}

fn parse_file(path: &Path) -> Option<DesktopEntry> {
    let raw_entry = freedesktop_entry_parser::parse_entry(&path).ok()?;
    let section = raw_entry.section("Desktop Entry");

    let mut entry = DesktopEntry::default();
    entry.file_name = path.file_name()?.to_owned();

    for attr in section.attrs().into_iter().filter(|a| a.has_value()) {
        match attr.name {
            "Name" if entry.name == "" => {
                entry.name = attr.value.unwrap().into();
            }
            "Exec" => entry.exec = attr.value.unwrap().into(),
            "MimeType" => {
                let mut mimes = attr
                    .value
                    .unwrap()
                    .split(";")
                    .filter_map(|m| Mime::from_str(m).ok())
                    .collect::<Vec<_>>();
                mimes.pop();
                entry.mimes = mimes;
            }
            "Terminal" => entry.term = attr.value.unwrap() == "true",
            _ => {}
        }
    }

    if !entry.name.is_empty() && !entry.exec.is_empty() {
        Some(entry)
    } else {
        None
    }
}

impl TryFrom<PathBuf> for DesktopEntry {
    type Error = Error;
    fn try_from(path: PathBuf) -> Result<DesktopEntry> {
        parse_file(&path).ok_or(Error::BadEntry(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_exec() {
        let path = PathBuf::from("tests/cmus.desktop");
        parse_file(&*path).unwrap();
    }
}
