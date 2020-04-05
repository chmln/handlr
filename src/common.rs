use anyhow::Result;
use pest::Parser;
use std::convert::TryFrom;
use std::path::PathBuf;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Mime(pub String);

impl std::fmt::Debug for Mime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, pest_derive::Parser, Default, PartialEq, Eq)]
#[grammar = "ini.pest"]
pub struct DesktopEntry {
    pub(crate) name: String,
    pub(crate) exec: String,
    pub(crate) mimes: Vec<Mime>,
}

impl TryFrom<PathBuf> for DesktopEntry {
    type Error = anyhow::Error;
    fn try_from(p: PathBuf) -> Result<DesktopEntry> {
        let raw = std::fs::read_to_string(p)?;
        let file = Self::parse(Rule::file, &raw)?.next().unwrap();

        let mut entry = Self::default();

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
