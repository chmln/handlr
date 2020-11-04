use crate::{
    common::{DesktopEntry, Handler},
    Result,
};
use mime::Mime;
use std::{collections::HashMap, convert::TryFrom, ffi::OsStr};

#[derive(Debug, Default)]
pub struct SystemApps(pub HashMap<Mime, Vec<Handler>>);

impl SystemApps {
    pub fn get_handlers(&self, mime: &Mime) -> Option<Vec<Handler>> {
        Some(self.0.get(mime)?.clone())
    }
    pub fn get_handler(&self, mime: &Mime) -> Option<Handler> {
        Some(self.get_handlers(mime)?.get(0).unwrap().clone())
    }
    pub fn populate() -> Result<Self> {
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
                        p.path().extension() == Some(OsStr::new("desktop"))
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
