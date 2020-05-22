use crate::{DesktopEntry, Error, Handler, Mime, Result};
use pest::Parser;
use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
};

#[derive(Debug, pest_derive::Parser)]
#[grammar = "ini.pest"]
pub struct MimeApps {
    added_associations: HashMap<Mime, VecDeque<Handler>>,
    default_apps: HashMap<Mime, VecDeque<Handler>>,
    system_apps: SystemApps,
}

impl MimeApps {
    pub fn set_handler(&mut self, mime: Mime, handler: Handler) -> Result<()> {
        let handlers = self.default_apps.entry(mime).or_default();
        handlers.push_front(handler);
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
        self.default_apps
            .get(mime)
            .or_else(|| self.added_associations.get(mime))
            .map(|hs| hs.get(0).unwrap().clone())
            .or_else(|| self.system_apps.get_handler(mime))
            .ok_or(Error::NotFound)
    }
    pub fn show_handler(&self, mime: &Mime, output_json: bool) -> Result<()> {
        let handler = self.get_handler(mime)?;
        let output = if output_json {
            let entry = handler.get_entry()?;
            (json::object! {
                handler: handler.to_string(),
                name: entry.name.as_str(),
                cmd: entry.get_cmd(None)?.0
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
                        use std::str::FromStr;

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
                        match current_section_name.as_str() {
                            "Added Associations" => conf
                                .added_associations
                                .insert(Mime(name.to_owned()), handlers),
                            "Default Applications" => conf
                                .default_apps
                                .insert(Mime(name.to_owned()), handlers),
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
            writer.write_all(k.0.as_ref())?;
            writer.write_all(b"=")?;
            writer.write_all(v.iter().join(";").as_ref())?;
            writer.write_all(b";\n")?;
        }

        writer.write_all(b"\n[Default Applications]\n")?;
        for (k, v) in self.default_apps.iter().sorted() {
            writer.write_all(k.0.as_ref())?;
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
            .map(|(k, v)| vec![k.0.clone(), v.iter().join(", ")])
            .collect::<Vec<_>>();

        ascii_table::AsciiTable::default().print(rows);

        Ok(())
    }
    pub fn list_handlers(&self) -> Result<()> {
        use std::{convert::TryFrom, io::Write};

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

#[derive(Debug)]
pub struct SystemApps(pub HashMap<Mime, Vec<Handler>>);

impl SystemApps {
    pub fn get_handlers(&self, mime: &Mime) -> Option<Vec<Handler>> {
        Some(self.0.get(mime)?.clone())
    }
    pub fn get_handler(&self, mime: &Mime) -> Option<Handler> {
        Some(self.get_handlers(mime)?.get(0).unwrap().clone())
    }
    pub fn populate() -> Result<Self> {
        use std::convert::TryFrom;

        let mut map = HashMap::<Mime, Vec<Handler>>::with_capacity(50);

        xdg::BaseDirectories::new()?
            .get_data_dirs()
            .into_iter()
            .map(|mut data_dir| {
                data_dir.push("applications");
                data_dir
            })
            .filter_map(|data_dir| std::fs::read_dir(data_dir).ok())
            .for_each(|dir| {
                dir.filter_map(Result::ok)
                    .filter(|p| {
                        p.path()
                            .extension()
                            .map(std::ffi::OsStr::to_str)
                            .flatten()
                            == Some("desktop")
                    })
                    .filter_map(|p| DesktopEntry::try_from(p.path()).ok())
                    .for_each(|entry| {
                        let (file_name, mimes) = (entry.file_name, entry.mimes);
                        mimes.into_iter().for_each(|mime| {
                            map.entry(mime)
                                .or_default()
                                .push(Handler::assume_valid(file_name.clone()));
                        });
                    });
            });

        Ok(Self(map))
    }
}
