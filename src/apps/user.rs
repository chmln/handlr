use crate::{
    apps::SystemApps,
    common::{DesktopEntry, Handler},
    Error, Result,
};
use mime::Mime;
use pest::Parser;
use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    str::FromStr,
};

#[derive(Debug, pest_derive::Parser)]
#[grammar = "common/ini.pest"]
pub struct MimeApps {
    added_associations: HashMap<Mime, VecDeque<Handler>>,
    default_apps: HashMap<Mime, VecDeque<Handler>>,
    system_apps: SystemApps,
}

impl MimeApps {
    pub fn add_handler(&mut self, mime: Mime, handler: Handler) -> Result<()> {
        let handlers = self.default_apps.entry(mime).or_default();
        handlers.push_back(handler);
        self.save()?;
        Ok(())
    }
    pub fn set_handler(&mut self, mime: Mime, handler: Handler) -> Result<()> {
        self.default_apps.insert(mime, {
            let mut handlers = VecDeque::with_capacity(1);
            handlers.push_back(handler);
            handlers
        });
        self.save()?;
        Ok(())
    }
    pub fn remove_handler(&mut self, mime: &Mime) -> Result<()> {
        if let Some(_removed) = self.default_apps.remove(mime) {
            self.save()?;
        }

        Ok(())
    }
    pub fn get_handler(&self, mime: &Mime) -> Result<Handler> {
        let config = crate::config::Config::load()?;

        match self.default_apps.get(mime) {
            Some(handlers) if config.enable_selector && handlers.len() > 1 => {
                let handlers = handlers
                    .into_iter()
                    .map(|h| (h, h.get_entry().unwrap().name))
                    .collect::<Vec<_>>();
                let selected =
                    config.select(handlers.iter().map(|h| h.1.clone()))?;
                let selected =
                    handlers.into_iter().find(|h| h.1 == selected).unwrap().0;
                Ok(selected.clone())
            }
            Some(handlers) => Ok(handlers.get(0).unwrap().clone()),
            None => self
                .added_associations
                .get(mime)
                .map(|h| h.get(0).unwrap().clone())
                .or_else(|| self.system_apps.get_handler(mime))
                .ok_or(Error::NotFound(mime.to_string())),
        }
    }
    pub fn show_handler(&self, mime: &Mime, output_json: bool) -> Result<()> {
        let handler = self.get_handler(mime)?;
        let output = if output_json {
            let entry = handler.get_entry()?;
            (json::object! {
                handler: handler.to_string(),
                name: entry.name.as_str(),
                cmd: entry.get_cmd(vec![])?.0
            })
            .to_string()
        } else {
            handler.to_string()
        };
        println!("{}", output);
        Ok(())
    }
    pub fn path() -> Result<PathBuf> {
        let mut config = xdg::BaseDirectories::new()?.get_config_home();
        config.push("mimeapps.list");
        Ok(config)
    }
    pub fn read() -> Result<Self> {
        let raw_conf = std::fs::read_to_string(Self::path()?)?;
        let file = Self::parse(Rule::file, &raw_conf)
            .expect("unsuccessful parse") // unwrap the parse result
            .next()
            .unwrap();

        let mut current_section_name = "".to_string();
        let mut conf = Self {
            added_associations: HashMap::default(),
            default_apps: HashMap::default(),
            system_apps: SystemApps::populate()?,
        };

        file.into_inner().for_each(|line| {
            match line.as_rule() {
                Rule::section => {
                    current_section_name = line.into_inner().concat();
                }
                Rule::property => {
                    let mut inner_rules = line.into_inner(); // { name ~ "=" ~ value }

                    let name = inner_rules.next().unwrap().as_str();
                    let handlers = {
                        use itertools::Itertools;

                        inner_rules
                            .next()
                            .unwrap()
                            .as_str()
                            .split(";")
                            .filter(|s| !s.is_empty())
                            .unique()
                            .filter_map(|s| Handler::from_str(s).ok())
                            .collect::<VecDeque<_>>()
                    };

                    if !handlers.is_empty() {
                        match (
                            Mime::from_str(name),
                            current_section_name.as_str(),
                        ) {
                            (Ok(mime), "Added Associations") => {
                                conf.added_associations.insert(mime, handlers)
                            }

                            (Ok(mime), "Default Applications") => {
                                conf.default_apps.insert(mime, handlers)
                            }
                            _ => None,
                        };
                    }
                }
                _ => {}
            }
        });

        Ok(conf)
    }
    pub fn save(&self) -> Result<()> {
        use itertools::Itertools;
        use std::io::{prelude::*, BufWriter};

        let f = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .open(Self::path()?)?;
        let mut writer = BufWriter::new(f);

        writer.write_all(b"[Added Associations]\n")?;
        for (k, v) in self.added_associations.iter().sorted() {
            writer.write_all(k.essence_str().as_ref())?;
            writer.write_all(b"=")?;
            writer.write_all(v.iter().join(";").as_ref())?;
            writer.write_all(b";\n")?;
        }

        writer.write_all(b"\n[Default Applications]\n")?;
        for (k, v) in self.default_apps.iter().sorted() {
            writer.write_all(k.essence_str().as_ref())?;
            writer.write_all(b"=")?;
            writer.write_all(v.iter().join(";").as_ref())?;
            writer.write_all(b";\n")?;
        }

        writer.flush()?;
        Ok(())
    }
    pub fn print(&self) -> Result<()> {
        use itertools::Itertools;

        let rows = self
            .default_apps
            .iter()
            .sorted()
            .map(|(k, v)| vec![k.to_string(), v.iter().join(", ")])
            .collect::<Vec<_>>();

        ascii_table::AsciiTable::default().print(rows);

        Ok(())
    }
    pub fn list_handlers(&self) -> Result<()> {
        use std::{convert::TryFrom, io::Write, os::unix::ffi::OsStrExt};

        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();

        xdg::BaseDirectories::new()?
            .list_data_files_once("applications")
            .into_iter()
            .filter(|p| {
                p.extension().map(|x| x.to_str()).flatten() == Some("desktop")
            })
            .filter_map(|p| DesktopEntry::try_from(p).ok())
            .for_each(|e| {
                stdout.write_all(e.file_name.as_bytes()).unwrap();
                stdout.write_all(b"\t").unwrap();
                stdout.write_all(e.name.as_bytes()).unwrap();
                stdout.write_all(b"\n").unwrap();
            });

        Ok(())
    }
}
