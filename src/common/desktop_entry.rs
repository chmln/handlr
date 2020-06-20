use crate::{Error, Result};
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

#[derive(PartialEq, Eq)]
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
            cmd.status()?;
        } else {
            cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
        };

        Ok(())
    }
    pub fn get_cmd(&self, arg: Vec<String>) -> Result<(String, Vec<String>)> {
        let special = regex::Regex::new("%(f|F|u|U)").unwrap();

        let mut split = shlex::split(&self.exec)
            .unwrap()
            .into_iter()
            .flat_map(|s| match s.as_str() {
                "%f" | "%F" | "%u" | "%U" => arg.clone(),
                s if special.is_match(s) => vec![special
                    .replace_all(s, arg.clone().join(" ").as_str())
                    .into()],
                _ => vec![s],
            })
            .collect::<Vec<_>>();

        Ok((split.remove(0), split))
    }
}

fn parse_file(path: &Path) -> Option<DesktopEntry> {
    let raw = std::fs::read(&path).ok()?;
    let parsed = freedesktop_entry_parser::parse_entry(&raw)
        .filter_map(Result::ok)
        .find(|s| s.title == b"Desktop Entry")?;

    let mut entry = DesktopEntry::default();
    entry.file_name = path.file_name()?.to_owned();

    for attr in parsed.attrs {
        match attr.name {
            b"Name" if entry.name == "" => {
                entry.name = String::from_utf8(attr.value.into()).ok()?;
            }
            b"Exec" => {
                entry.exec = String::from_utf8(attr.value.into()).ok()?
            }
            b"MimeType" => {
                let mut mimes = String::from_utf8(attr.value.into())
                    .ok()?
                    .split(";")
                    .filter_map(|m| Mime::from_str(m).ok())
                    .collect::<Vec<_>>();
                mimes.pop();
                entry.mimes = mimes;
            }
            b"Terminal" => entry.term = attr.value == b"true",
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
