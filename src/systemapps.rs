use crate::{DesktopEntry, Mime};
use anyhow::Result;
use dashmap::DashMap;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct SystemApps(pub DashMap<Mime, Vec<DesktopEntry>>);

impl SystemApps {
    pub fn get_handlers(&self, mime: &Mime) -> Option<Vec<DesktopEntry>> {
        Some(self.0.get(mime)?.value().clone())
    }
    pub fn populate() -> Result<Self> {
        use rayon::iter::ParallelBridge;
        use rayon::prelude::ParallelIterator;

        let map = DashMap::<Mime, Vec<DesktopEntry>>::with_capacity(50);

        std::fs::read_dir("/usr/share/applications")?
            .par_bridge()
            .filter_map(|path| {
                path.ok()
                    .map(|p| DesktopEntry::try_from(p.path()).ok())
                    .flatten()
            })
            .for_each(|entry| {
                let (name, exec, mimes) = (entry.name, entry.exec, entry.mimes);
                mimes.into_iter().for_each(|mime| {
                    map.entry(mime).or_default().push(DesktopEntry {
                        name: name.clone(),
                        exec: exec.clone(),
                        mimes: Vec::new(),
                    });
                });
            });

        Ok(Self(map))
    }
}
