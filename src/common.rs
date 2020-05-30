use crate::{mime_types, Error, Result};
use std::{convert::TryFrom, path::PathBuf};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mime(pub String);

impl Mime {
    pub fn try_from_path(path: &str) -> Result<Self> {
        if let Ok(url) = url::Url::parse(path) {
            return Ok(Mime(format!("x-scheme-handler/{}", url.scheme())));
        }

        mime_types::from_file(path)
    }
}

impl std::str::FromStr for Mime {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with(".") {
            mime_types::from_file(s)
        } else {
            Ok(Mime(mime_types::verify(&s)?.to_owned()))
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handler(String);

impl std::fmt::Display for Handler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for Handler {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::resolve(s.to_owned())
    }
}

impl Handler {
    pub fn assume_valid(name: String) -> Self {
        Self(name)
    }
    pub fn get_path(name: &str) -> Option<PathBuf> {
        xdg::BaseDirectories::new()
            .ok()?
            .find_data_file(&format!("applications/{}", name))
    }
    pub fn resolve(name: String) -> Result<Self> {
        let path = Self::get_path(&name).ok_or(Error::NotFound)?;
        DesktopEntry::try_from(path)?;
        Ok(Self(name))
    }
    pub fn get_entry(&self) -> Result<DesktopEntry> {
        DesktopEntry::try_from(Self::get_path(&self.0).unwrap())
    }
    pub fn open(&self, arg: String) -> Result<()> {
        let (cmd, args) = self.get_entry()?.get_cmd(Some(arg))?;
        std::process::Command::new(cmd)
            .args(args)
            .stdout(std::process::Stdio::null())
            .spawn()?;
        Ok(())
    }
    pub fn launch(&self, args: Vec<String>) -> Result<()> {
        let (cmd, mut base_args) = self.get_entry()?.get_cmd(None)?;
        base_args.extend_from_slice(&args);
        std::process::Command::new(cmd)
            .args(base_args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
        Ok(())
    }
}

#[derive(Debug, Clone, pest_derive::Parser, Default, PartialEq, Eq)]
#[grammar = "ini.pest"]
pub struct DesktopEntry {
    pub(crate) name: String,
    pub(crate) exec: String,
    pub(crate) file_name: String,
    pub(crate) mimes: Vec<Mime>,
}

impl DesktopEntry {
    pub fn get_cmd(
        &self,
        arg: Option<String>,
    ) -> Result<(String, Vec<String>)> {
        let special = regex::Regex::new("%(f|F|u|U)").unwrap();
        let mut split = shlex::split(&self.exec)
            .ok_or(Error::BadCmd)?
            .into_iter()
            .map(|s| match s.as_str() {
                "%f" | "%F" | "%u" | "%U" => arg.clone(),
                s if special.is_match(s) => Some(
                    special
                        .replace_all(s, arg.as_deref().unwrap_or_default())
                        .into(),
                ),
                _ => Some(s),
            })
            .filter_map(std::convert::identity)
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

        let mut section: &str = Default::default();
        let mut entry = Self::default();
        entry.file_name = p.file_name().unwrap().to_str().unwrap().to_owned();

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
                                .map(ToOwned::to_owned)
                                .map(Mime)
                                .collect::<Vec<_>>();
                            mimes.pop();
                            entry.mimes = mimes;
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
            Err(Error::BadCmd)
        }
    }
}
