use anyhow::Result;
use pest::Parser;
use std::convert::TryFrom;
use std::path::PathBuf;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mime(pub String);

impl std::str::FromStr for Mime {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}
#[derive(Debug, derive_more::Display, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handler(String);

impl std::str::FromStr for Handler {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::resolve(s.to_owned())
    }
}

impl Handler {
    pub fn assume_valid(name: String) -> Self {
        Self(name)
    }
    pub fn get_path(name: &str) -> Option<PathBuf> {
        let locally = {
            let mut local_dir = dirs::data_dir().unwrap();
            local_dir.push("applications");
            local_dir.push(name);
            Some(local_dir).filter(|p| p.exists())
        };
        let system = {
            let mut sys = std::path::PathBuf::from("/usr/share/applications");
            sys.push(name);
            Some(sys).filter(|p| p.exists())
        };
        locally.or(system)
    }
    pub fn resolve(name: String) -> Result<Self> {
        let path =
            Self::get_path(&name).ok_or_else(|| anyhow::Error::msg("Handler does not exist"))?;
        DesktopEntry::try_from(path)?;
        Ok(Self(name))
    }
    pub fn get_entry(&self) -> Result<DesktopEntry> {
        DesktopEntry::try_from(Self::get_path(&self.0).unwrap())
    }
    pub fn run(&self, arg: &str) -> Result<()> {
        std::process::Command::new("gtk-launch")
            .args(&[self.0.as_str(), arg])
            .stdout(std::process::Stdio::null())
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

impl TryFrom<PathBuf> for DesktopEntry {
    type Error = anyhow::Error;
    fn try_from(p: PathBuf) -> Result<DesktopEntry> {
        let raw = std::fs::read_to_string(&p)?;
        let file = Self::parse(Rule::file, &raw)?.next().unwrap();

        let mut entry = Self::default();
        entry.file_name = p.file_name().unwrap().to_str().unwrap().to_owned();

        for line in file.into_inner() {
            match line.as_rule() {
                Rule::property => {
                    let mut inner_rules = line.into_inner(); // { name ~ "=" ~ value }

                    let name = inner_rules.next().unwrap().as_str();
                    match name {
                        "Name" => {
                            entry.name = inner_rules.next().unwrap().as_str().into();
                        }
                        "Exec" => {
                            entry.exec = inner_rules.next().unwrap().as_str().into();
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
            Err(anyhow::Error::msg("Invalid desktop entry"))
        }
    }
}
