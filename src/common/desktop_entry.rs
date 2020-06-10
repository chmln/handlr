use crate::{Error, Result};
use mime::Mime;
use pest::Parser;
use std::{
    convert::TryFrom,
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
};

#[derive(Debug, Clone, pest_derive::Parser, Default, PartialEq, Eq)]
#[grammar = "common/ini.pest"]
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
        let (cmd, args) = self.get_cmd(arg)?;
        let mut cmd = Command::new(cmd);
        cmd.args(args);
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
    let raw = std::fs::read_to_string(&path).ok()?;
    let file = DesktopEntry::parse(Rule::file, &raw).ok()?.next()?;

    let mut entry = DesktopEntry::default();
    entry.file_name = path.file_name()?.to_owned();

    let mut section = "";

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::section => section = line.into_inner().as_str(),
            Rule::property if section == "Desktop Entry" => {
                let mut inner = line.into_inner(); // { name ~ "=" ~ value }

                match inner.next()?.as_str() {
                    "Name" if entry.name == "" => {
                        entry.name = inner.next()?.as_str().into()
                    }
                    "Exec" => entry.exec = inner.next()?.as_str().into(),
                    "MimeType" => {
                        let mut mimes = inner
                            .next()?
                            .as_str()
                            .split(";")
                            .filter_map(|m| Mime::from_str(m).ok())
                            .collect::<Vec<_>>();
                        mimes.pop();
                        entry.mimes = mimes;
                    }
                    "Terminal" => entry.term = inner.next()?.as_str() == "true",
                    _ => {}
                }
            }
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
