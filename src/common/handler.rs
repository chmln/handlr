use crate::{
    common::{DesktopEntry, ExecMode},
    Error, Result,
};
use std::{fmt::Display, convert::TryFrom, ffi::OsString, path::PathBuf, str::FromStr};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handler(OsString);

impl Display for Handler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string_lossy())
    }
}

impl FromStr for Handler {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::resolve(s.into())
    }
}

impl Handler {
    pub fn assume_valid(name: OsString) -> Self {
        Self(name)
    }
    pub fn get_path(name: &std::ffi::OsStr) -> Option<PathBuf> {
        let mut path = PathBuf::from("applications");
        path.push(name);
        xdg::BaseDirectories::new().ok()?.find_data_file(path)
    }
    pub fn resolve(name: OsString) -> Result<Self> {
        let path = Self::get_path(&name)
            .ok_or(Error::NotFound(name.to_string_lossy().into()))?;
        DesktopEntry::try_from(path)?;
        Ok(Self(name))
    }
    pub fn get_entry(&self) -> Result<DesktopEntry> {
        DesktopEntry::try_from(Self::get_path(&self.0).unwrap())
    }
    pub fn launch(&self, args: Vec<String>) -> Result<()> {
        self.get_entry()?.exec(ExecMode::Launch, args)
    }
    pub fn open(&self, args: Vec<String>) -> Result<()> {
        self.get_entry()?.exec(ExecMode::Open, args)
    }
}
