//! The per-user table of compiled routines.

use std::collections::{BTreeMap, HashSet};

use super::{fold, truncated};
use crate::message::code::*;
use crate::message::{Macro, Reply, macro_position};

/// The image bytes a save may reach by listing a name repeatedly. A short request can name one
/// large image many times, and the service would hold a copy for each.
const MAX_SAVE_REPEAT: usize = 256 << 20;

/// Macros by folded name. `add` and `reorder` take only a position before or
/// after (Deviation 185).
#[derive(Default)]
pub(super) struct MacroSpace {
    macros: BTreeMap<Vec<u8>, Macro>,
}

fn key(name: &[u8]) -> Vec<u8> {
    fold(&name[..name.len().min(crate::message::MAX_NAME)])
}

impl MacroSpace {
    /// Adds or replaces a macro, or answers `RXMACRO_INVALID_POSITION` and
    /// changes nothing for a position that is neither before nor after.
    pub(super) fn add(&mut self, name: Vec<u8>, image: Vec<u8>, position: u64) -> Reply {
        if !macro_position::is_valid(position) {
            return Reply::Status(MACRO_INVALID_POSITION);
        }
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

    /// Moves a macro, or answers `RXMACRO_INVALID_POSITION` and changes
    /// nothing for a position that is neither before nor after, whether or not
    /// the macro is there.
    pub(super) fn reorder(&mut self, name: Vec<u8>, position: u64) -> Reply {
        if !macro_position::is_valid(position) {
            return Reply::Status(MACRO_INVALID_POSITION);
        }
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
                let found: Option<Vec<&Macro>> = names
                    .iter()
                    .map(|name| self.macros.get(&key(name)))
                    .collect();
                let Some(found) = found else {
                    return refused(MACRO_NOT_FOUND);
                };
                let wanted = found.iter().map(|entry| entry.image.len()).sum::<usize>();
                let held = self.macros.values().map(|entry| entry.image.len()).sum();
                if wanted > usize::max(held, MAX_SAVE_REPEAT) {
                    return refused(MACRO_NO_STORAGE);
                }
                found.into_iter().cloned().collect()
            }
        };
        Reply::Macros { status: OK, macros }
    }

    pub(super) fn load(&mut self, names: Option<Vec<Vec<u8>>>, macros: Vec<Macro>) -> Reply {
        let wanted: Option<HashSet<Vec<u8>>> =
            names.map(|names| names.iter().map(|name| key(name)).collect());
        for loaded in macros {
            let selected = wanted
                .as_ref()
                .is_none_or(|wanted| wanted.contains(&key(&loaded.name)));
            // A macro whose position is refused ends the load as a cut file
            // does: the macros before it stay loaded.
            if selected && self.add(loaded.name, loaded.image, loaded.position) != Reply::Status(OK)
            {
                return Reply::Status(MACRO_FILE_ERROR);
            }
        }
        Reply::Status(OK)
    }

    pub(super) fn is_empty(&self) -> bool {
        self.macros.is_empty()
    }
}
