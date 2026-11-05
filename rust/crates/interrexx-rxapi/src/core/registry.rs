//! The function, subcommand and exit registries of library registrations.

use std::collections::HashMap;

use super::{ClientId, fold, truncated};
use crate::message::code::*;
use crate::message::{Registered, Registry, Reply};

struct Entry {
    library_key: Vec<u8>,
    registered: Registered,
    /// The connection that first registered the pair.
    owner: u64,
    /// The authority as first registered.
    owner_only: bool,
    /// References anyone may drop.
    pool: u64,
    /// References belonging to one connection.
    owned: HashMap<u64, u64>,
    touched: u64,
}

impl Entry {
    fn add_reference(&mut self, connection: u64, owner_only: bool) {
        if owner_only {
            *self.owned.entry(connection).or_default() += 1;
        } else {
            self.pool += 1;
        }
    }

    /// Removes the reference a drop by `connection` may take, answering whether there was one.
    fn take_reference(&mut self, connection: u64) -> bool {
        let references = if self.owner_only {
            self.owned.get_mut(&connection)
        } else {
            Some(&mut self.pool)
        };
        match references {
            Some(references) if *references > 0 => {
                *references -= 1;
                true
            }
            _ => false,
        }
    }

    fn has_references(&self) -> bool {
        self.pool > 0 || self.owned.values().any(|references| *references > 0)
    }
}

/// One registry, by folded registration name.
#[derive(Default)]
pub(super) struct Table {
    always_owner_only: bool,
    entries: HashMap<Vec<u8>, Vec<Entry>>,
    clock: u64,
}

impl Table {
    /// The index of the entry an operation acts on, touching it if `library` names it.
    fn find(&mut self, key: &[u8], library: &[u8]) -> Option<usize> {
        let tick = self.tick();
        let entries = self.entries.get_mut(key)?;
        if library.is_empty() {
            return (0..entries.len()).max_by_key(|at| entries[*at].touched);
        }
        let library_key = fold(library);
        let at = entries
            .iter()
            .position(|entry| entry.library_key == library_key)?;
        entries[at].touched = tick;
        Some(at)
    }

    fn tick(&mut self) -> u64 {
        self.clock += 1;
        self.clock
    }

    pub(super) fn register(
        &mut self,
        client: ClientId,
        name: Vec<u8>,
        library: Vec<u8>,
        procedure: Vec<u8>,
        user_area: Option<[u8; 16]>,
        owner_only: bool,
    ) -> Reply {
        let owner_only = owner_only || self.always_owner_only;
        let (name, library) = (truncated(name), truncated(library));
        let key = fold(&name);
        let library_key = fold(&library);
        let tick = self.tick();
        let entries = self.entries.entry(key).or_default();
        let existing = entries
            .iter_mut()
            .find(|entry| entry.library_key == library_key);
        Reply::Status(match existing {
            Some(entry) => {
                entry.add_reference(client.connection, owner_only);
                entry.touched = tick;
                DUPLICATE
            }
            None => {
                let mut entry = Entry {
                    library_key,
                    registered: Registered {
                        library,
                        procedure: truncated(procedure),
                        user_area: user_area.unwrap_or_default(),
                    },
                    owner: client.connection,
                    owner_only,
                    pool: 0,
                    owned: HashMap::new(),
                    touched: tick,
                };
                entry.add_reference(client.connection, owner_only);
                entries.push(entry);
                OK
            }
        })
    }

    pub(super) fn query(&mut self, name: Vec<u8>, library: Vec<u8>) -> Reply {
        let key = fold(&truncated(name));
        let library = truncated(library);
        let registered = self
            .find(&key, &library)
            .map(|at| self.entries[&key][at].registered.clone());
        Reply::Registration {
            status: if registered.is_some() {
                OK
            } else {
                NOT_REGISTERED
            },
            registered,
        }
    }

    pub(super) fn deregister(
        &mut self,
        client: ClientId,
        name: Vec<u8>,
        library: Vec<u8>,
    ) -> Reply {
        let key = fold(&truncated(name));
        let library = truncated(library);
        let Some(at) = self.find(&key, &library) else {
            return Reply::Status(NOT_REGISTERED);
        };
        let entries = self.entries.get_mut(&key).expect("find saw this name");
        let entry = &mut entries[at];
        if entry.owner_only && entry.owner != client.connection {
            return Reply::Status(NO_CAN_DROP);
        }
        entry.take_reference(client.connection);
        if entry.has_references() {
            return Reply::Status(if library.is_empty() { OK } else { DUPLICATE });
        }
        entries.swap_remove(at);
        if entries.is_empty() {
            self.entries.remove(&key);
        }
        Reply::Status(OK)
    }
}

/// The independent registries.
pub(super) struct Registries {
    function: Table,
    subcom: Table,
    exit: Table,
}

impl Default for Registries {
    fn default() -> Registries {
        Registries {
            function: Table {
                always_owner_only: true,
                ..Table::default()
            },
            subcom: Table::default(),
            exit: Table::default(),
        }
    }
}

impl Registries {
    pub(super) fn table(&mut self, registry: Registry) -> &mut Table {
        match registry {
            Registry::Function => &mut self.function,
            Registry::Subcom => &mut self.subcom,
            Registry::Exit => &mut self.exit,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        [&self.function, &self.subcom, &self.exit]
            .iter()
            .all(|table| table.entries.is_empty())
    }
}
