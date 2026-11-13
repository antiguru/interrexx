//! `rxsubcom REGISTER|QUERY|DROP|LOAD envname [library [procname]]`: the subcommand registry from
//! the command line. The exit status is the API's return code.

use std::io::Write;
use std::process::ExitCode;

use interrexx_rxapi::client::connect;
use interrexx_rxapi::message::code::{NOT_REGISTERED, OK};
use interrexx_rxapi::message::{Registry, Reply, Request};
use interrexx_rxapi::transport::Transport;
use interrexx_rxapi::utility::{
    SUBCOM_DROP, SUBCOM_GENERAL, SUBCOM_LOAD, SUBCOM_QUERY, SUBCOM_REGISTER, message,
};

/// The exit status of a usage error, -1 as a byte.
const USAGE: u8 = 255;

#[derive(Debug, PartialEq, Eq)]
enum Action {
    Register {
        name: Vec<u8>,
        library: Vec<u8>,
        procedure: Vec<u8>,
    },
    Query {
        name: Vec<u8>,
        library: Vec<u8>,
    },
    Drop {
        name: Vec<u8>,
        library: Vec<u8>,
    },
    Load {
        name: Vec<u8>,
    },
}

/// The action the arguments name, or the subcode of the usage message to print.
fn parse(arguments: &[Vec<u8>]) -> Result<Action, u16> {
    let Some((action, operands)) = arguments.split_first() else {
        return Err(SUBCOM_GENERAL);
    };
    let name_and_library = |sub| match operands {
        [name] => Ok((name.clone(), Vec::new())),
        [name, library] => Ok((name.clone(), library.clone())),
        _ => Err(sub),
    };
    match action.to_ascii_uppercase().as_slice() {
        b"REGISTER" => match operands {
            [name, library, procedure] => Ok(Action::Register {
                name: name.clone(),
                library: library.clone(),
                procedure: procedure.clone(),
            }),
            _ => Err(SUBCOM_REGISTER),
        },
        b"QUERY" => {
            name_and_library(SUBCOM_QUERY).map(|(name, library)| Action::Query { name, library })
        }
        b"DROP" => {
            name_and_library(SUBCOM_DROP).map(|(name, library)| Action::Drop { name, library })
        }
        b"LOAD" => name_and_library(SUBCOM_LOAD).map(|(name, _)| Action::Load { name }),
        _ => Err(SUBCOM_GENERAL),
    }
}

/// The request an action sends. `LOAD` asks for the registration by name alone.
fn request(action: Action) -> Request {
    let registry = Registry::Subcom;
    match action {
        Action::Register {
            name,
            library,
            procedure,
        } => Request::Register {
            registry,
            name,
            library,
            procedure,
            user_area: None,
            owner_only: false,
        },
        Action::Query { name, library } => Request::QueryRegistration {
            registry,
            name,
            library,
        },
        Action::Drop { name, library } => Request::Deregister {
            registry,
            name,
            library,
        },
        Action::Load { name } => Request::QueryRegistration {
            registry,
            name,
            library: Vec::new(),
        },
    }
}

/// The return code of `reply`. A `LOAD` succeeds for a registration with no library, or whose
/// library loads and has its procedure, as spelled or uppercased.
fn status(reply: Reply, load: bool) -> u64 {
    match reply {
        Reply::Registration {
            status: OK,
            registered: Some(registered),
        } if load => {
            let loads = registered.library.is_empty()
                || interrexx_api::load::classic_procedure(
                    &registered.library,
                    &registered.procedure,
                    &[],
                    std::path::Path::new("."),
                )
                .is_some();
            if loads { OK } else { NOT_REGISTERED }
        }
        Reply::Status(status) | Reply::Registration { status, .. } => status,
        _ => NOT_REGISTERED,
    }
}

fn main() -> ExitCode {
    use std::os::unix::ffi::OsStringExt;
    let arguments: Vec<Vec<u8>> = std::env::args_os()
        .skip(1)
        .map(OsStringExt::into_vec)
        .collect();
    let action = match parse(&arguments) {
        Ok(action) => action,
        Err(sub) => {
            let mut out = std::io::stdout();
            let _ = writeln!(out, "{}", message(sub, &[]));
            let _ = out.flush();
            return ExitCode::from(USAGE);
        }
    };
    let load = matches!(action, Action::Load { .. });
    let code = match connect() {
        Ok(client) => match client.call(request(action)) {
            Ok(reply) => status(reply, load),
            Err(error) => error.status(),
        },
        Err(status) => status,
    };
    ExitCode::from((code % 256) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use interrexx_rxapi::message::Registered;

    fn words(line: &str) -> Vec<Vec<u8>> {
        line.split_whitespace()
            .map(|word| word.as_bytes().to_vec())
            .collect()
    }

    #[test]
    fn each_action_takes_its_operands_caselessly() {
        assert_eq!(
            parse(&words("Register env lib proc")),
            Ok(Action::Register {
                name: b"env".to_vec(),
                library: b"lib".to_vec(),
                procedure: b"proc".to_vec(),
            })
        );
        assert_eq!(
            parse(&words("query env")),
            Ok(Action::Query {
                name: b"env".to_vec(),
                library: Vec::new(),
            })
        );
        assert_eq!(
            parse(&words("DROP env lib")),
            Ok(Action::Drop {
                name: b"env".to_vec(),
                library: b"lib".to_vec(),
            })
        );
        assert_eq!(
            parse(&words("load env ignored")),
            Ok(Action::Load {
                name: b"env".to_vec()
            })
        );
    }

    #[test]
    fn wrong_operands_name_the_action() {
        let sub = |line| parse(&words(line)).unwrap_err();
        assert_eq!(sub(""), SUBCOM_GENERAL);
        assert_eq!(sub("bogus env"), SUBCOM_GENERAL);
        assert_eq!(sub("register env"), SUBCOM_REGISTER);
        assert_eq!(sub("query"), SUBCOM_QUERY);
        assert_eq!(sub("query a b c"), SUBCOM_QUERY);
        assert_eq!(sub("drop"), SUBCOM_DROP);
        assert_eq!(sub("load"), SUBCOM_LOAD);
    }

    #[test]
    fn a_load_fails_for_a_library_that_does_not_load() {
        let found = |library: &[u8]| Reply::Registration {
            status: OK,
            registered: Some(Registered {
                library: library.to_vec(),
                procedure: b"proc".to_vec(),
                user_area: [0; 16],
            }),
        };
        assert_eq!(status(found(b""), true), OK);
        assert_eq!(status(found(b"lib"), true), NOT_REGISTERED);
        assert_eq!(status(found(b"lib"), false), OK);
        let missing = Reply::Registration {
            status: NOT_REGISTERED,
            registered: None,
        };
        assert_eq!(status(missing, true), NOT_REGISTERED);
    }
}
