//! The service's state machine: no sockets, threads or clocks.

mod macrospace;
mod queue;
mod registry;

use crate::message::{Item, Reply, Request};

use macrospace::MacroSpace;
use queue::Queues;
use registry::Registries;

/// Who a request came from and where its reply goes.
///
/// `connection` is the identity: one for a client's whole connection, and never reused. `call`
/// tells apart the concurrent calls of one connection, so each parked pull has its own reply.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClientId {
    pub connection: u64,
    pub call: u64,
}

impl ClientId {
    /// The first call of `connection`.
    pub fn connection(connection: u64) -> ClientId {
        ClientId {
            connection,
            call: 0,
        }
    }

    /// Another call on the same connection.
    pub fn call(self, call: u64) -> ClientId {
        ClientId { call, ..self }
    }
}

/// What `Core::handle` did with a request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Reply(Reply),
    /// The reply will come out of `Core::take_ready` after a later call.
    Parked,
}

/// All state the service holds.
#[derive(Default)]
pub struct Core {
    queues: Queues,
    registries: Registries,
    macros: MacroSpace,
}

impl Core {
    pub fn new() -> Core {
        Core::default()
    }

    /// Runs one request to its reply, or parks it.
    pub fn handle(&mut self, client: ClientId, request: Request) -> Outcome {
        let reply = match request {
            Request::CreateSessionQueue { session } => self.queues.attach(client, session),
            Request::DeleteSessionQueue { .. } => Reply::Status(crate::message::code::OK),
            Request::CreateQueue {
                session,
                requested,
                size,
            } => self.queues.create(session, requested, size),
            Request::OpenQueue { name } => self.queues.open(name),
            Request::QueueExists { name } => self.queues.exists(name),
            Request::DeleteQueue { name } => self.queues.delete(name),
            Request::ClearQueue { session, name } => self.queues.clear(client, session, name),
            Request::QueryQueue { session, name } => self.queues.query(client, session, name),
            Request::AddQueue {
                session,
                name,
                data,
                flag,
                time,
            } => self.queues.add(client, session, name, data, flag, time),
            Request::PullQueue {
                session,
                name,
                wait,
            } => return self.queues.pull(client, session, name, wait),
            Request::CancelPull { call } => self.queues.cancel(client.call(call)),
            Request::Register {
                registry,
                name,
                library,
                procedure,
                user_area,
                owner_only,
            } => self
                .registries
                .table(registry)
                .register(client, name, library, procedure, user_area, owner_only),
            Request::QueryRegistration {
                registry,
                name,
                library,
            } => self.registries.table(registry).query(name, library),
            Request::Deregister {
                registry,
                name,
                library,
            } => self
                .registries
                .table(registry)
                .deregister(client, name, library),
            Request::AddMacro {
                name,
                image,
                position,
            } => self.macros.add(name, image, position),
            Request::DropMacro { name } => self.macros.drop_macro(name),
            Request::ClearMacroSpace => self.macros.clear(),
            Request::QueryMacro { name } => self.macros.query(name),
            Request::ReorderMacro { name, position } => self.macros.reorder(name, position),
            Request::ResolveMacro { name } => self.macros.resolve(name),
            Request::SaveMacros { names } => self.macros.save(names),
            Request::LoadMacros { names, macros } => self.macros.load(names, macros),
        };
        Outcome::Reply(reply)
    }

    /// Replies to parked requests that have become ready since the last call.
    pub fn take_ready(&mut self) -> Vec<(ClientId, Reply)> {
        self.queues.take_ready()
    }

    /// A pull of `name` took `item`, and its client was gone. The item goes to the next waiter, or
    /// back to the head of the queue.
    pub fn undelivered(&mut self, session: u64, name: Option<Vec<u8>>, item: Item) {
        self.queues.restore(session, name, item);
    }

    /// `client`'s connection closed: its parked pulls end and its session queues lose a user.
    pub fn client_closed(&mut self, client: ClientId) {
        self.queues.connection_closed(client.connection);
    }

    /// No named queue, registration or macro.
    pub fn is_empty(&self) -> bool {
        self.queues.no_named_queues() && self.registries.is_empty() && self.macros.is_empty()
    }
}

/// The key a caseless name compares under.
fn fold(name: &[u8]) -> Vec<u8> {
    name.to_ascii_uppercase()
}

/// `name` cut to the length the service stores.
fn truncated(mut name: Vec<u8>) -> Vec<u8> {
    name.truncate(crate::message::MAX_NAME);
    name
}
