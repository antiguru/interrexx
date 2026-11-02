//! The per-user table of compiled routines.

use std::collections::BTreeMap;

use super::{fold, truncated};
use crate::message::code::*;
use crate::message::{Macro, Reply};

/// Macros by folded name. Positions are stored unchecked, as the API does.
#[derive(Default)]
pub(super) struct MacroSpace {
    macros: BTreeMap<Vec<u8>, Macro>,
}

fn key(name: &[u8]) -> Vec<u8> {
    fold(&name[..name.len().min(crate::message::MAX_NAME)])
}

impl MacroSpace {
    pub(super) fn add(&mut self, name: Vec<u8>, image: Vec<u8>, position: u64) -> Reply {
        let name = truncated(name);
        self.macros
            .entry(fold(&name))
            .and_modify(|existing| {
                existing.image = image.clone();
                existing.position = position;
            })
            .or_insert(Macro {
                name,
                image,
                position,
            });
        Reply::Status(OK)
    }

    pub(super) fn drop_macro(&mut self, name: Vec<u8>) -> Reply {
        Reply::Status(match self.macros.remove(&key(&name)) {
            Some(_) => OK,
            None => MACRO_NOT_FOUND,
        })
    }

    pub(super) fn clear(&mut self) -> Reply {
        self.macros.clear();
        Reply::Status(OK)
    }

    pub(super) fn query(&self, name: Vec<u8>) -> Reply {
        match self.macros.get(&key(&name)) {
            Some(found) => Reply::MacroPosition {
                status: OK,
                position: found.position,
            },
            None => Reply::MacroPosition {
                status: MACRO_NOT_FOUND,
                position: 0,
            },
        }
    }

    pub(super) fn reorder(&mut self, name: Vec<u8>, position: u64) -> Reply {
        Reply::Status(match self.macros.get_mut(&key(&name)) {
            Some(found) => {
                found.position = position;
                OK
            }
            None => MACRO_NOT_FOUND,
        })
    }

    pub(super) fn resolve(&self, name: Vec<u8>) -> Reply {
        match self.macros.get(&key(&name)) {
            Some(found) => Reply::MacroImage {
                status: OK,
                image: Some(found.image.clone()),
            },
            None => Reply::MacroImage {
                status: MACRO_NOT_FOUND,
                image: None,
            },
        }
    }

    pub(super) fn save(&self, names: Option<Vec<Vec<u8>>>) -> Reply {
        let refused = |status| Reply::Macros {
            status,
            macros: Vec::new(),
        };
        let macros = match names {
            None if self.macros.is_empty() => return refused(MACRO_FILE_ERROR),
            None => self.macros.values().cloned().collect(),
            Some(names) => {
                let listed: Option<Vec<Macro>> = names
                    .iter()
                    .map(|name| self.macros.get(&key(name)).cloned())
                    .collect();
                match listed {
                    Some(listed) => listed,
                    None => return refused(MACRO_NOT_FOUND),
                }
            }
        };
        Reply::Macros { status: OK, macros }
    }

    pub(super) fn load(&mut self, names: Option<Vec<Vec<u8>>>, macros: Vec<Macro>) -> Reply {
        let wanted: Option<Vec<Vec<u8>>> =
            names.map(|names| names.iter().map(|name| key(name)).collect());
        for loaded in macros {
            let selected = wanted
                .as_ref()
                .is_none_or(|wanted| wanted.contains(&key(&loaded.name)));
            if selected {
                self.add(loaded.name, loaded.image, loaded.position);
            }
        }
        Reply::Status(OK)
    }

    pub(super) fn is_empty(&self) -> bool {
        self.macros.is_empty()
    }
}
