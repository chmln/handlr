use crate::{
    common::{DesktopEntry, ExecMode, UserPath},
    error::Result,
    CONFIG,
};
use once_cell::sync::Lazy;
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsString,
    hash::{Hash, Hasher},
};

pub static REGEX_APPS: Lazy<RegexApps> = Lazy::new(RegexApps::populate);

// used for deserializing from config file
#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigHandler {
    exec: String,
    #[serde(default)]
    terminal: bool,
    regexes: Vec<String>,
}

impl ConfigHandler {
    // convert to RegexHandler
    fn compile_regex(&self) -> Result<RegexHandler> {
        Ok(RegexHandler {
            exec: self.exec.clone(),
            terminal: self.terminal,
            regexes: HandlerRegexSet::new(self.regexes.clone())?,
        })
    }
}

// wrapping RegexSet in a struct and implementing Eq and Hash for it
// saves us from having to implement them for RegexHandler as a whole
// although it complicates method calls a bit
#[derive(Debug, Clone)]
struct HandlerRegexSet(RegexSet);

impl HandlerRegexSet {
    fn new<I, S>(exprs: I) -> Result<HandlerRegexSet>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        Ok(HandlerRegexSet(RegexSet::new(exprs)?))
    }

    fn is_match(&self, text: &str) -> bool {
        self.0.is_match(text)
    }
}

impl PartialEq for HandlerRegexSet {
    fn eq(&self, other: &Self) -> bool {
        self.0.patterns() == other.0.patterns()
    }
}

impl Eq for HandlerRegexSet {}

impl Hash for HandlerRegexSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.patterns().hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegexHandler {
    exec: String,
    terminal: bool,
    regexes: HandlerRegexSet,
}

impl RegexHandler {
    // kludge together a fake DesktopEntry
    // there's probably a better way to avoid reinventing the wheel with the program execution code
    fn get_entry(&self) -> DesktopEntry {
        //
        DesktopEntry {
            name: String::from(""),
            exec: self.exec.clone(),
            file_name: OsString::from(""),
            terminal: self.terminal,
            mimes: Vec::new(),
            categories: HashMap::new(),
        }
    }

    // open the given paths with handler
    pub fn open(&self, args: Vec<String>) -> Result<()> {
        self.get_entry().exec(ExecMode::Open, args)
    }

    fn is_match(&self, path: &str) -> bool {
        self.regexes.is_match(path)
    }
}

#[derive(Debug)]
pub struct RegexApps(Vec<RegexHandler>);

impl RegexApps {
    // convert Config's ConfigHandlers
    pub fn populate() -> Self {
        RegexApps(
            CONFIG
                .handlers
                .iter()
                .filter_map(|handler| handler.compile_regex().ok())
                .collect(),
        )
    }
    // get matching handler
    pub fn get_handler(&self, path: &UserPath) -> Option<RegexHandler> {
        Some(
            self.0
                .iter()
                .find(|app| app.is_match(&path.to_string()))?
                .clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn regex_handlers() -> Result<()> {
        let exec: &str = "freetube %u";
        let regexes: &[String] =
            &[String::from(r"(https://)?(www\.)?youtu(be\.com|\.be)/*")];

        let config_handler = ConfigHandler {
            exec: String::from(exec),
            terminal: false,
            regexes: regexes.to_owned(),
        };

        let regex_handler = config_handler
            .compile_regex()
            .expect("ConfigHandler::compile_regex() returned Err");

        let expected_regex_handler = RegexHandler {
            exec: String::from(exec),
            terminal: false,
            regexes: HandlerRegexSet::new(regexes)
                .expect("Test regex is invalid"),
        };

        assert_eq!(regex_handler, expected_regex_handler);

        let regex_apps = RegexApps(vec![regex_handler]);

        assert_eq!(
            regex_apps
                .get_handler(&UserPath::Url(
                    Url::parse("https://youtu.be/dQw4w9WgXcQ").unwrap()
                ))
                .expect("RegexApps::get_handler() returned None"),
            expected_regex_handler
        );

        assert_eq!(
            regex_apps.get_handler(&UserPath::Url(
                Url::parse("https://en.wikipedia.org").unwrap()
            )),
            None
        );

        Ok(())
    }
}
