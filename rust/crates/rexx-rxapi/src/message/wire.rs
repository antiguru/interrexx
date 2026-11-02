//! The byte encoding of requests and replies, little-endian and length-prefixed.
//!
//! A connection opens with `hello` and `welcome`. After that each direction carries frames: an
//! 8-byte length of what follows, an 8-byte call number, and a body. Only this crate's own clients
//! and daemon of one `PROTOCOL_VERSION` speak it.

use std::io::{self, Read};

use super::{Item, Macro, Registered, Registry, Reply, Request, Timestamp};

/// Bumped on any change here; it names the socket.
pub const PROTOCOL_VERSION: u32 = 1;

const HELLO_MAGIC: [u8; 4] = *b"RXAP";
/// Length of a `hello`.
pub const HELLO_LENGTH: usize = 8;
/// Length of a `welcome`.
pub const WELCOME_LENGTH: usize = 5;

/// What a client sends first.
pub fn hello(version: u32) -> [u8; HELLO_LENGTH] {
    let mut bytes = [0; HELLO_LENGTH];
    bytes[..4].copy_from_slice(&HELLO_MAGIC);
    bytes[4..].copy_from_slice(&version.to_le_bytes());
    bytes
}

/// The version a `hello` declares, or `None` for bytes that are not one.
pub fn hello_version(bytes: &[u8; HELLO_LENGTH]) -> Option<u32> {
    (bytes[..4] == HELLO_MAGIC)
        .then(|| u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]))
}

/// The service's answer to a `hello`: whether it accepts, and its own version.
pub fn welcome(accepted: bool, version: u32) -> [u8; WELCOME_LENGTH] {
    let mut bytes = [0; WELCOME_LENGTH];
    bytes[0] = u8::from(accepted);
    bytes[1..].copy_from_slice(&version.to_le_bytes());
    bytes
}

/// Reads a `welcome` as (accepted, version).
pub fn read_welcome(bytes: &[u8; WELCOME_LENGTH]) -> (bool, u32) {
    (
        bytes[0] == 1,
        u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]),
    )
}

/// A client's message: an operation, or the stop that `rxapi --stop` sends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Request(Request),
    Stop,
}

/// Bytes that do not decode.
#[derive(Debug, PartialEq, Eq)]
pub struct Malformed;

/// A frame around `body`.
pub fn frame(call: u64, body: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(body.len() + 16);
    bytes.extend_from_slice(&(body.len() as u64 + 8).to_le_bytes());
    bytes.extend_from_slice(&call.to_le_bytes());
    bytes.extend_from_slice(body);
    bytes
}

/// Reads one frame as (call, body). `None` is a clean end of stream.
///
/// The length is trusted only as far as bytes arrive, so a bad one cannot allocate.
pub fn read_frame(reader: &mut impl Read) -> io::Result<Option<(u64, Vec<u8>)>> {
    let mut length = [0; 8];
    let mut filled = 0;
    while filled < length.len() {
        match reader.read(&mut length[filled..]) {
            Ok(0) if filled == 0 => return Ok(None),
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(count) => filled += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    let length = u64::from_le_bytes(length);
    let body_length = length
        .checked_sub(8)
        .ok_or(io::Error::from(io::ErrorKind::InvalidData))?;
    let mut call = [0; 8];
    reader.read_exact(&mut call)?;
    let mut body = Vec::new();
    let read = reader.by_ref().take(body_length).read_to_end(&mut body)?;
    if read as u64 != body_length {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    Ok(Some((u64::from_le_bytes(call), body)))
}

#[derive(Default)]
struct Encoder(Vec<u8>);

impl Encoder {
    fn tag(&mut self, tag: u8) {
        self.0.push(tag);
    }

    fn number(&mut self, value: u64) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    fn flag(&mut self, value: bool) {
        self.0.push(u8::from(value));
    }

    fn bytes(&mut self, bytes: &[u8]) {
        self.number(bytes.len() as u64);
        self.0.extend_from_slice(bytes);
    }

    fn optional<T>(&mut self, value: &Option<T>, mut encode: impl FnMut(&mut Encoder, &T)) {
        self.flag(value.is_some());
        if let Some(value) = value {
            encode(self, value);
        }
    }

    fn optional_bytes(&mut self, value: &Option<Vec<u8>>) {
        self.optional(value, |encoder, bytes| encoder.bytes(bytes));
    }

    fn names(&mut self, names: &[Vec<u8>]) {
        self.number(names.len() as u64);
        for name in names {
            self.bytes(name);
        }
    }

    fn timestamp(&mut self, time: &Timestamp) {
        for field in [
            time.year,
            time.month,
            time.day,
            time.weekday,
            time.yearday,
            time.hours,
            time.minutes,
            time.seconds,
        ] {
            self.0.extend_from_slice(&field.to_le_bytes());
        }
    }

    fn registry(&mut self, registry: Registry) {
        self.tag(match registry {
            Registry::Function => 0,
            Registry::Subcom => 1,
            Registry::Exit => 2,
        });
    }

    fn macros(&mut self, macros: &[Macro]) {
        self.number(macros.len() as u64);
        for entry in macros {
            self.bytes(&entry.name);
            self.bytes(&entry.image);
            self.number(entry.position);
        }
    }
}

struct Decoder<'a>(&'a [u8]);

impl Decoder<'_> {
    fn take(&mut self, count: usize) -> Result<&[u8], Malformed> {
        if count > self.0.len() {
            return Err(Malformed);
        }
        let (taken, rest) = self.0.split_at(count);
        self.0 = rest;
        Ok(taken)
    }

    fn tag(&mut self) -> Result<u8, Malformed> {
        Ok(self.take(1)?[0])
    }

    fn number(&mut self) -> Result<u64, Malformed> {
        let bytes: [u8; 8] = self.take(8)?.try_into().map_err(|_| Malformed)?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn flag(&mut self) -> Result<bool, Malformed> {
        match self.tag()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Malformed),
        }
    }

    fn bytes(&mut self) -> Result<Vec<u8>, Malformed> {
        let length = usize::try_from(self.number()?).map_err(|_| Malformed)?;
        Ok(self.take(length)?.to_vec())
    }

    fn optional<T>(
        &mut self,
        mut decode: impl FnMut(&mut Self) -> Result<T, Malformed>,
    ) -> Result<Option<T>, Malformed> {
        if self.flag()? {
            decode(self).map(Some)
        } else {
            Ok(None)
        }
    }

    fn optional_bytes(&mut self) -> Result<Option<Vec<u8>>, Malformed> {
        self.optional(Self::bytes)
    }

    fn names(&mut self) -> Result<Vec<Vec<u8>>, Malformed> {
        let count = self.number()?;
        let mut names = Vec::new();
        for _ in 0..count {
            names.push(self.bytes()?);
        }
        Ok(names)
    }

    fn field(&mut self) -> Result<u16, Malformed> {
        let bytes: [u8; 2] = self.take(2)?.try_into().map_err(|_| Malformed)?;
        Ok(u16::from_le_bytes(bytes))
    }

    fn timestamp(&mut self) -> Result<Timestamp, Malformed> {
        Ok(Timestamp {
            year: self.field()?,
            month: self.field()?,
            day: self.field()?,
            weekday: self.field()?,
            yearday: self.field()?,
            hours: self.field()?,
            minutes: self.field()?,
            seconds: self.field()?,
        })
    }

    fn registry(&mut self) -> Result<Registry, Malformed> {
        match self.tag()? {
            0 => Ok(Registry::Function),
            1 => Ok(Registry::Subcom),
            2 => Ok(Registry::Exit),
            _ => Err(Malformed),
        }
    }

    fn user_area(&mut self) -> Result<[u8; 16], Malformed> {
        self.take(16)?.try_into().map_err(|_| Malformed)
    }

    fn macros(&mut self) -> Result<Vec<Macro>, Malformed> {
        let count = self.number()?;
        let mut macros = Vec::new();
        for _ in 0..count {
            macros.push(Macro {
                name: self.bytes()?,
                image: self.bytes()?,
                position: self.number()?,
            });
        }
        Ok(macros)
    }

    fn finished<T>(self, value: T) -> Result<T, Malformed> {
        if self.0.is_empty() {
            Ok(value)
        } else {
            Err(Malformed)
        }
    }
}

const STOP: u8 = 255;

/// The body of a stop.
pub fn encode_stop() -> Vec<u8> {
    vec![STOP]
}

/// The body of `request`.
pub fn encode_request(request: &Request) -> Vec<u8> {
    let mut out = Encoder::default();
    match request {
        Request::CreateSessionQueue { session } => {
            out.tag(0);
            out.number(*session);
        }
        Request::DeleteSessionQueue { session } => {
            out.tag(1);
            out.number(*session);
        }
        Request::CreateQueue {
            session,
            requested,
            size,
        } => {
            out.tag(2);
            out.number(*session);
            out.optional_bytes(requested);
            out.number(*size);
        }
        Request::OpenQueue { name } => {
            out.tag(3);
            out.optional_bytes(name);
        }
        Request::QueueExists { name } => {
            out.tag(4);
            out.optional_bytes(name);
        }
        Request::DeleteQueue { name } => {
            out.tag(5);
            out.optional_bytes(name);
        }
        Request::ClearQueue { session, name } => {
            out.tag(6);
            out.number(*session);
            out.optional_bytes(name);
        }
        Request::QueryQueue { session, name } => {
            out.tag(7);
            out.number(*session);
            out.optional_bytes(name);
        }
        Request::AddQueue {
            session,
            name,
            data,
            flag,
            time,
        } => {
            out.tag(8);
            out.number(*session);
            out.optional_bytes(name);
            out.bytes(data);
            out.number(*flag);
            out.timestamp(time);
        }
        Request::PullQueue {
            session,
            name,
            wait,
        } => {
            out.tag(9);
            out.number(*session);
            out.optional_bytes(name);
            out.number(*wait);
        }
        Request::Register {
            registry,
            name,
            library,
            procedure,
            user_area,
            owner_only,
        } => {
            out.tag(10);
            out.registry(*registry);
            out.bytes(name);
            out.bytes(library);
            out.bytes(procedure);
            out.optional(user_area, |encoder, area| encoder.0.extend_from_slice(area));
            out.flag(*owner_only);
        }
        Request::QueryRegistration {
            registry,
            name,
            library,
        } => {
            out.tag(11);
            out.registry(*registry);
            out.bytes(name);
            out.bytes(library);
        }
        Request::Deregister {
            registry,
            name,
            library,
        } => {
            out.tag(12);
            out.registry(*registry);
            out.bytes(name);
            out.bytes(library);
        }
        Request::AddMacro {
            name,
            image,
            position,
        } => {
            out.tag(13);
            out.bytes(name);
            out.bytes(image);
            out.number(*position);
        }
        Request::DropMacro { name } => {
            out.tag(14);
            out.bytes(name);
        }
        Request::ClearMacroSpace => out.tag(15),
        Request::QueryMacro { name } => {
            out.tag(16);
            out.bytes(name);
        }
        Request::ReorderMacro { name, position } => {
            out.tag(17);
            out.bytes(name);
            out.number(*position);
        }
        Request::ResolveMacro { name } => {
            out.tag(18);
            out.bytes(name);
        }
        Request::SaveMacros { names } => {
            out.tag(19);
            out.optional(names, |encoder, names| encoder.names(names));
        }
        Request::LoadMacros { names, macros } => {
            out.tag(20);
            out.optional(names, |encoder, names| encoder.names(names));
            out.macros(macros);
        }
    }
    out.0
}

/// Reads the body of a `Command`.
pub fn decode_command(bytes: &[u8]) -> Result<Command, Malformed> {
    let mut input = Decoder(bytes);
    let request = match input.tag()? {
        0 => Request::CreateSessionQueue {
            session: input.number()?,
        },
        1 => Request::DeleteSessionQueue {
            session: input.number()?,
        },
        2 => Request::CreateQueue {
            session: input.number()?,
            requested: input.optional_bytes()?,
            size: input.number()?,
        },
        3 => Request::OpenQueue {
            name: input.optional_bytes()?,
        },
        4 => Request::QueueExists {
            name: input.optional_bytes()?,
        },
        5 => Request::DeleteQueue {
            name: input.optional_bytes()?,
        },
        6 => Request::ClearQueue {
            session: input.number()?,
            name: input.optional_bytes()?,
        },
        7 => Request::QueryQueue {
            session: input.number()?,
            name: input.optional_bytes()?,
        },
        8 => Request::AddQueue {
            session: input.number()?,
            name: input.optional_bytes()?,
            data: input.bytes()?,
            flag: input.number()?,
            time: input.timestamp()?,
        },
        9 => Request::PullQueue {
            session: input.number()?,
            name: input.optional_bytes()?,
            wait: input.number()?,
        },
        10 => Request::Register {
            registry: input.registry()?,
            name: input.bytes()?,
            library: input.bytes()?,
            procedure: input.bytes()?,
            user_area: input.optional(Decoder::user_area)?,
            owner_only: input.flag()?,
        },
        11 => Request::QueryRegistration {
            registry: input.registry()?,
            name: input.bytes()?,
            library: input.bytes()?,
        },
        12 => Request::Deregister {
            registry: input.registry()?,
            name: input.bytes()?,
            library: input.bytes()?,
        },
        13 => Request::AddMacro {
            name: input.bytes()?,
            image: input.bytes()?,
            position: input.number()?,
        },
        14 => Request::DropMacro {
            name: input.bytes()?,
        },
        15 => Request::ClearMacroSpace,
        16 => Request::QueryMacro {
            name: input.bytes()?,
        },
        17 => Request::ReorderMacro {
            name: input.bytes()?,
            position: input.number()?,
        },
        18 => Request::ResolveMacro {
            name: input.bytes()?,
        },
        19 => Request::SaveMacros {
            names: input.optional(Decoder::names)?,
        },
        20 => Request::LoadMacros {
            names: input.optional(Decoder::names)?,
            macros: input.macros()?,
        },
        STOP => return input.finished(Command::Stop),
        _ => return Err(Malformed),
    };
    input.finished(Command::Request(request))
}

/// The body of `reply`.
pub fn encode_reply(reply: &Reply) -> Vec<u8> {
    let mut out = Encoder::default();
    match reply {
        Reply::Status(status) => {
            out.tag(0);
            out.number(*status);
        }
        Reply::QueueCreated {
            status,
            name,
            duplicate,
        } => {
            out.tag(1);
            out.number(*status);
            out.optional_bytes(name);
            out.optional(duplicate, |encoder, value| encoder.flag(*value));
        }
        Reply::QueueOpened { status, duplicate } => {
            out.tag(2);
            out.number(*status);
            out.optional(duplicate, |encoder, value| encoder.flag(*value));
        }
        Reply::QueueCount { status, count } => {
            out.tag(3);
            out.number(*status);
            out.optional(count, |encoder, value| encoder.number(*value));
        }
        Reply::Pulled { status, item } => {
            out.tag(4);
            out.number(*status);
            out.optional(item, |encoder, item| {
                encoder.bytes(&item.data);
                encoder.timestamp(&item.time);
            });
        }
        Reply::Registration { status, registered } => {
            out.tag(5);
            out.number(*status);
            out.optional(registered, |encoder, registered| {
                encoder.bytes(&registered.library);
                encoder.bytes(&registered.procedure);
                encoder.0.extend_from_slice(&registered.user_area);
            });
        }
        Reply::MacroPosition { status, position } => {
            out.tag(6);
            out.number(*status);
            out.number(*position);
        }
        Reply::MacroImage { status, image } => {
            out.tag(7);
            out.number(*status);
            out.optional_bytes(image);
        }
        Reply::Macros { status, macros } => {
            out.tag(8);
            out.number(*status);
            out.macros(macros);
        }
    }
    out.0
}

/// Reads the body of a `Reply`.
pub fn decode_reply(bytes: &[u8]) -> Result<Reply, Malformed> {
    let mut input = Decoder(bytes);
    let reply = match input.tag()? {
        0 => Reply::Status(input.number()?),
        1 => Reply::QueueCreated {
            status: input.number()?,
            name: input.optional_bytes()?,
            duplicate: input.optional(Decoder::flag)?,
        },
        2 => Reply::QueueOpened {
            status: input.number()?,
            duplicate: input.optional(Decoder::flag)?,
        },
        3 => Reply::QueueCount {
            status: input.number()?,
            count: input.optional(Decoder::number)?,
        },
        4 => Reply::Pulled {
            status: input.number()?,
            item: input.optional(|input| {
                Ok(Item {
                    data: input.bytes()?,
                    time: input.timestamp()?,
                })
            })?,
        },
        5 => Reply::Registration {
            status: input.number()?,
            registered: input.optional(|input| {
                Ok(Registered {
                    library: input.bytes()?,
                    procedure: input.bytes()?,
                    user_area: input.user_area()?,
                })
            })?,
        },
        6 => Reply::MacroPosition {
            status: input.number()?,
            position: input.number()?,
        },
        7 => Reply::MacroImage {
            status: input.number()?,
            image: input.optional_bytes()?,
        },
        8 => Reply::Macros {
            status: input.number()?,
            macros: input.macros()?,
        },
        _ => return Err(Malformed),
    };
    input.finished(reply)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn requests() -> Vec<Request> {
        let bytes = || b"bytes\0\xff".to_vec();
        let time = Timestamp {
            year: 126,
            month: 10,
            day: 2,
            weekday: 1,
            yearday: 305,
            hours: 23,
            minutes: 59,
            seconds: 60,
        };
        let entry = Macro {
            name: bytes(),
            image: Vec::new(),
            position: u64::MAX,
        };
        vec![
            Request::CreateSessionQueue { session: u64::MAX },
            Request::DeleteSessionQueue { session: 1 },
            Request::CreateQueue {
                session: 2,
                requested: Some(bytes()),
                size: 3,
            },
            Request::CreateQueue {
                session: 2,
                requested: None,
                size: 0,
            },
            Request::OpenQueue { name: None },
            Request::OpenQueue {
                name: Some(Vec::new()),
            },
            Request::QueueExists {
                name: Some(bytes()),
            },
            Request::DeleteQueue {
                name: Some(bytes()),
            },
            Request::ClearQueue {
                session: 4,
                name: None,
            },
            Request::QueryQueue {
                session: 5,
                name: Some(bytes()),
            },
            Request::AddQueue {
                session: 6,
                name: None,
                data: bytes(),
                flag: 7,
                time,
            },
            Request::PullQueue {
                session: 8,
                name: Some(bytes()),
                wait: 9,
            },
            Request::Register {
                registry: Registry::Exit,
                name: bytes(),
                library: Vec::new(),
                procedure: bytes(),
                user_area: Some([7; 16]),
                owner_only: true,
            },
            Request::Register {
                registry: Registry::Function,
                name: bytes(),
                library: bytes(),
                procedure: Vec::new(),
                user_area: None,
                owner_only: false,
            },
            Request::QueryRegistration {
                registry: Registry::Subcom,
                name: bytes(),
                library: bytes(),
            },
            Request::Deregister {
                registry: Registry::Subcom,
                name: bytes(),
                library: Vec::new(),
            },
            Request::AddMacro {
                name: bytes(),
                image: bytes(),
                position: 99,
            },
            Request::DropMacro { name: bytes() },
            Request::ClearMacroSpace,
            Request::QueryMacro { name: bytes() },
            Request::ReorderMacro {
                name: bytes(),
                position: 1,
            },
            Request::ResolveMacro { name: bytes() },
            Request::SaveMacros { names: None },
            Request::SaveMacros {
                names: Some(vec![bytes(), Vec::new()]),
            },
            Request::LoadMacros {
                names: Some(Vec::new()),
                macros: vec![entry.clone(), entry],
            },
        ]
    }

    fn replies() -> Vec<Reply> {
        vec![
            Reply::Status(1004),
            Reply::QueueCreated {
                status: 0,
                name: Some(b"Q".to_vec()),
                duplicate: Some(true),
            },
            Reply::QueueCreated {
                status: 5,
                name: None,
                duplicate: None,
            },
            Reply::QueueOpened {
                status: 0,
                duplicate: Some(false),
            },
            Reply::QueueCount {
                status: 0,
                count: Some(3),
            },
            Reply::QueueCount {
                status: 5,
                count: None,
            },
            Reply::Pulled {
                status: 0,
                item: Some(Item {
                    data: vec![0, 255],
                    time: Timestamp::default(),
                }),
            },
            Reply::Pulled {
                status: 8,
                item: None,
            },
            Reply::Registration {
                status: 0,
                registered: Some(Registered {
                    library: b"lib".to_vec(),
                    procedure: Vec::new(),
                    user_area: [9; 16],
                }),
            },
            Reply::Registration {
                status: 30,
                registered: None,
            },
            Reply::MacroPosition {
                status: 0,
                position: 2,
            },
            Reply::MacroImage {
                status: 0,
                image: Some(Vec::new()),
            },
            Reply::Macros {
                status: 0,
                macros: vec![Macro {
                    name: Vec::new(),
                    image: vec![1],
                    position: 0,
                }],
            },
        ]
    }

    #[test]
    fn requests_round_trip() {
        for request in requests() {
            let body = encode_request(&request);
            assert_eq!(
                decode_command(&body),
                Ok(Command::Request(request.clone())),
                "{request:?}"
            );
        }
        assert_eq!(decode_command(&encode_stop()), Ok(Command::Stop));
    }

    #[test]
    fn replies_round_trip() {
        for reply in replies() {
            assert_eq!(decode_reply(&encode_reply(&reply)), Ok(reply.clone()));
        }
    }

    #[test]
    fn truncated_and_trailing_bytes_are_malformed_at_every_length() {
        for request in requests() {
            let body = encode_request(&request);
            for length in 0..body.len() {
                assert_eq!(decode_command(&body[..length]), Err(Malformed));
            }
            let mut long = body;
            long.push(0);
            assert_eq!(decode_command(&long), Err(Malformed));
        }
        for reply in replies() {
            let body = encode_reply(&reply);
            for length in 0..body.len() {
                assert_eq!(decode_reply(&body[..length]), Err(Malformed));
            }
        }
    }

    #[test]
    fn unknown_tags_are_malformed() {
        assert_eq!(decode_command(&[21]), Err(Malformed));
        assert_eq!(decode_reply(&[9]), Err(Malformed));
    }

    #[test]
    fn a_huge_declared_length_is_malformed_not_an_allocation() {
        let mut body = vec![3, 1];
        body.extend_from_slice(&u64::MAX.to_le_bytes());
        assert_eq!(decode_command(&body), Err(Malformed));
    }

    #[test]
    fn frames_round_trip_and_a_cut_frame_is_an_error() {
        let bytes = frame(7, b"body");
        let mut reader = &bytes[..];
        assert_eq!(
            read_frame(&mut reader).expect("a whole frame"),
            Some((7, b"body".to_vec()))
        );
        assert_eq!(read_frame(&mut reader).expect("the end"), None);

        for cut in 1..bytes.len() {
            let mut reader = &bytes[..cut];
            assert!(read_frame(&mut reader).is_err(), "cut at {cut}");
        }
    }

    #[test]
    fn a_frame_shorter_than_its_call_number_is_an_error() {
        let bytes = 7u64.to_le_bytes();
        assert!(read_frame(&mut &bytes[..]).is_err());
    }

    #[test]
    fn hello_declares_a_version_and_refuses_other_bytes() {
        assert_eq!(hello_version(&hello(3)), Some(3));
        assert_eq!(hello_version(b"GET /\r\n\r"), None);
        assert_eq!(read_welcome(&welcome(true, 3)), (true, 3));
        assert_eq!(read_welcome(&welcome(false, 4)), (false, 4));
    }
}
