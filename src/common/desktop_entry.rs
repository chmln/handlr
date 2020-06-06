use crate::{Error, Result};
use mime::Mime;
use std::{
    convert::TryFrom,
    ffi::OsString,
    path::PathBuf,
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

impl DesktopEntry {
    pub fn exec(&self, arguments: Vec<String>) -> Result<()> {
        let supports_multiple =
            self.exec.contains("%F") || self.exec.contains("%U");
        if arguments.is_empty() {
            self.exec_inner(None)?
        } else if supports_multiple {
            self.exec_inner(Some(arguments.join(" ")))?;
        } else {
            for arg in arguments {
                self.exec_inner(Some(arg))?;
            }
        };

        Ok(())
    }
    fn exec_inner(&self, arg: Option<String>) -> Result<()> {
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
    pub fn get_cmd(
        &self,
        arg: Option<String>,
    ) -> Result<(String, Vec<String>)> {
        let special = regex::Regex::new("%(f|F|u|U)").unwrap();

        let mut split = shlex::split(&self.exec)
            .unwrap()
            .into_iter()
            .filter_map(|s| match s.as_str() {
                "%f" | "%F" | "%u" | "%U" => arg.clone(),
                s if special.is_match(s) => Some(
                    special
                        .replace_all(s, arg.as_deref().unwrap_or_default())
                        .into(),
                ),
                _ => Some(s),
            })
            .collect::<Vec<_>>();

        Ok((split.remove(0), split))
    }
}

impl TryFrom<PathBuf> for DesktopEntry {
    type Error = Error;
    fn try_from(p: PathBuf) -> Result<DesktopEntry> {
        use pest::Parser;
        let raw = std::fs::read_to_string(&p)?;
        let file = Self::parse(Rule::file, &raw)?.next().unwrap();

        let mut entry = Self::default();
        entry.file_name = p.file_name().unwrap().to_owned();
        let mut section = "";

        for line in file.into_inner() {
            match line.as_rule() {
                Rule::section => {
                    section = line.into_inner().as_str();
                }
                Rule::property if section == "Desktop Entry" => {
                    let mut inner_rules = line.into_inner(); // { name ~ "=" ~ value }

                    let name = inner_rules.next().unwrap().as_str();
                    match name {
                        "Name" if entry.name.is_empty() => {
                            entry.name =
                                inner_rules.next().unwrap().as_str().into();
                        }
                        "Exec" => {
                            entry.exec =
                                inner_rules.next().unwrap().as_str().into();
                        }
                        "MimeType" => {
                            let mut mimes = inner_rules
                                .next()
                                .unwrap()
                                .as_str()
                                .split(";")
                                .filter_map(|m| Mime::from_str(m).ok())
                                .collect::<Vec<_>>();
                            mimes.pop();
                            entry.mimes = mimes;
                        }
                        "Terminal" => {
                            entry.term =
                                inner_rules.next().unwrap().as_str() == "true"
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if !entry.name.is_empty() && !entry.exec.is_empty() {
            Ok(entry)
        } else {
            Err(Error::BadEntry(p.clone()))
        }
    }
}
