use crate::{
    common::{DesktopEntry, Handler},
    Result,
};
use mime::Mime;
use std::{
    collections::{HashMap, VecDeque},
    convert::TryFrom,
    ffi::OsString,
};

#[derive(Debug, Default, Clone)]
pub struct SystemApps(pub HashMap<Mime, VecDeque<Handler>>);

impl SystemApps {
    pub fn get_handlers(&self, mime: &Mime) -> Option<VecDeque<Handler>> {
        Some(self.0.get(mime)?.clone())
    }
    pub fn get_handler(&self, mime: &Mime) -> Option<Handler> {
        Some(self.get_handlers(mime)?.get(0).unwrap().clone())
    }

    pub fn get_entries(
    ) -> Result<impl Iterator<Item = (OsString, DesktopEntry)>> {
        Ok(xdg::BaseDirectories::new()?
            .list_data_files_once("applications")
            .into_iter()
            .filter(|p| {
                p.extension().map(|x| x.to_str()).flatten() == Some("desktop")
            })
            .filter_map(|p| {
                Some((
                    p.file_name().unwrap().to_owned(),
                    DesktopEntry::try_from(p.clone()).ok()?,
                ))
            }))
    }

    pub fn populate() -> Result<Self> {
        let mut map = HashMap::<Mime, VecDeque<Handler>>::with_capacity(50);

        Self::get_entries()?.for_each(|(_, entry)| {
            let (file_name, mimes) = (entry.file_name, entry.mimes);
            mimes.into_iter().for_each(|mime| {
                map.entry(mime)
                    .or_default()
                    .push_back(Handler::assume_valid(file_name.clone()));
            });
        });

        Ok(Self(map))
    }
}
