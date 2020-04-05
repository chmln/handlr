use crate::{DesktopEntry, Mime};
use anyhow::Result;
use pest::Parser;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, pest_derive::Parser, Default)]
#[grammar = "ini.pest"]
pub struct MimeApps {
    added_associations: HashMap<Mime, Vec<PathBuf>>,
    default_apps: HashMap<Mime, Vec<PathBuf>>,
}

fn handler_exists(name: &str) -> Option<std::path::PathBuf> {
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

impl MimeApps {
    pub fn get_handler(&self, mime: &Mime) -> Option<DesktopEntry> {
        use std::convert::TryFrom;

        Some(
            self.default_apps
                .get(mime)
                .or_else(|| self.added_associations.get(mime))
                .map(|hs| hs.get(0).unwrap().clone())
                .map(DesktopEntry::try_from)
                .map(Result::ok)
                .flatten()?
                .clone(),
        )
    }
    pub fn read() -> Result<Self> {
        let path = dirs::config_dir()
            .map(|mut data_dir| {
                data_dir.push("mimeapps.list");
                data_dir
            })
            .ok_or_else(|| anyhow::Error::msg("Could not determine xdg data dir"))?;

        let raw_conf = std::fs::read_to_string(path)?;
        let file = Self::parse(Rule::file, &raw_conf)
            .expect("unsuccessful parse") // unwrap the parse result
            .next()
            .unwrap();

        let mut current_section_name = "".to_string();
        let mut conf = Self::default();

        file.into_inner().for_each(|line| {
            match line.as_rule() {
                Rule::section => {
                    current_section_name = line.into_inner().concat();
                }
                Rule::property => {
                    let mut inner_rules = line.into_inner(); // { name ~ "=" ~ value }

                    let name = inner_rules.next().unwrap().as_str();
                    let mut handlers = inner_rules
                        .next()
                        .unwrap()
                        .as_str()
                        .split(";")
                        .filter_map(handler_exists)
                        .collect::<Vec<_>>();
                    handlers.pop();

                    if !handlers.is_empty() {
                        match current_section_name.as_str() {
                            "Added Associations" => conf
                                .added_associations
                                .insert(Mime(name.to_owned()), handlers.to_owned()),
                            "Default Applications" => conf
                                .default_apps
                                .insert(Mime(name.to_owned()), handlers.to_owned()),
                            _ => None,
                        };
                    }
                }
                _ => {}
            }
        });

        Ok(conf)
    }
}
