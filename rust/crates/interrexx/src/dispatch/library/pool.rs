//! The classic API's reach into a running program from a handler it called:
//! `RexxVariablePool` and `RexxCallBack`, and the native frame a classic
//! handler runs in so that they find it.

use interrexx_api::callbacks::{CallBack, PoolAnswer, PoolRequest};
use interrexx_api::values::{Activation, CStringPool, Conversion};
use interrexx_core::{Body, ObjRef};

use super::condition_of;
use crate::activation::CallType;
use crate::builtin::datatype::{SymbolKind, classify};
use crate::run::{CallEntry, Ended, Resolved};
use crate::{Failure, Interp, Loud};

/// The longest name the variable pool accepts, a symbol's limit.
const LONGEST_NAME: usize = 250;

/// What a variable-pool name names.
enum Target {
    Simple(Vec<u8>),
    Stem(Vec<u8>),
    Compound {
        stem: Vec<u8>,
        key: Vec<u8>,
    },
    /// An environment symbol named symbolically, which no request changes and
    /// a fetch answers as unassigned (measured, oracle: `RXSHV_SYFET` of
    /// `.nil` answers `RXSHV_NEWV`, `RXSHV_FETCH` of `.NIL` `RXSHV_BADN`).
    Constant,
}

impl Interp {
    /// Runs `call`, a classic handler's, in a native frame of its own whose
    /// call opens the variable pool to the running activation. `call` answers
    /// `None` where a `RexxCallBack` ended it, and the condition or `EXIT`
    /// that did is the error.
    pub(crate) fn run_classic_handler<R>(
        &mut self,
        name: &[u8],
        arguments: &[Option<ObjRef>],
        call: impl FnOnce() -> Option<R>,
    ) -> Result<R, Failure> {
        self.push_native_frame(ObjRef::NIL, ObjRef::NIL, None, name, arguments, None);
        pin_enter!(self, crate::pinning::PinKind::NativeApiCallback);
        let mut strings = CStringPool::new();
        let thread = self.thread_context();
        let inbox = self.timer.inbox();
        let (answered, pending) = {
            let frame = self.native_token();
            let activation = Activation::new(Conversion {
                host: self.into(),
                strings: &mut strings,
            });
            activation.set_frame(frame);
            activation.open_variable_pool();
            let _reachable = crate::embedding::OtherRun::enter(inbox);
            let answered = thread.enter(&activation, |_| call());
            (answered, activation.pending())
        };
        let popped = self.pop_native_frame();
        pin_leave!(self);
        if let Some(number) = pending {
            return Err(condition_of(number));
        }
        match (answered, popped.raised) {
            (Some(value), None) => Ok(value),
            (_, Some(raised)) => Err(raised),
            (None, None) => Err(Loud::scheduler_inconsistency(
                "a classic handler unwound with nothing held",
            )
            .into()),
        }
    }

    /// Runs the system exit `handler` for `request` in a native frame of its
    /// own whose call opens the variable pool to the running activation, as
    /// `Activity::run` runs an exit-handler dispatcher. A condition a context
    /// handler raised is raised once it has returned, as a native call raises
    /// one.
    ///
    /// # Errors
    /// The condition the handler raised, and [`Loud`] for an interface member it reached that
    /// this crate has not written.
    pub(crate) fn run_exit(
        &mut self,
        handler: interrexx_api::load::ExitHandler,
        request: interrexx_api::load::ExitRequest<'_, Option<ObjRef>>,
    ) -> Result<interrexx_api::load::ExitOutcome<Option<ObjRef>>, Failure> {
        self.push_native_frame(ObjRef::NIL, ObjRef::NIL, None, b"", &[], None);
        pin_enter!(self, crate::pinning::PinKind::NativeApiCallback);
        let mut strings = CStringPool::new();
        let thread = self.thread_context();
        let inbox = self.timer.inbox();
        let (answered, pending) = {
            let frame = self.native_token();
            let activation = Activation::new(Conversion {
                host: self.into(),
                strings: &mut strings,
            });
            activation.set_frame(frame);
            activation.open_variable_pool();
            let _reachable = crate::embedding::OtherRun::enter(inbox);
            let answered = thread.enter(&activation, |contexts| {
                interrexx_api::invoke::exit(handler, contexts, &activation, request)
            });
            (answered, activation.pending())
        };
        let popped = self.pop_native_frame();
        pin_leave!(self);
        if let Some(number) = pending {
            return Err(condition_of(number));
        }
        let outcome = match answered {
            Ok(outcome) => outcome,
            Err(interrexx_api::values::Failure::Raised) => {
                return Err(popped
                    .raised
                    .expect("a host answering Raised holds the condition it raised"));
            }
            Err(refused) => return Err(self.refusal(refused, true, false)),
        };
        if let interrexx_api::load::ExitOutcome::Handled(reply) = &outcome
            && let Some(object) = reply.object
        {
            self.roots.activity_mut().push_temp(object);
        }
        if let Some(raised) = popped.raised {
            let held = (raised, popped.additional, popped.result);
            self.raise_held_condition(held, None, None)?;
        }
        Ok(outcome)
    }

    /// One `RexxVariablePool` request against the running activation.
    pub(crate) fn pool_request(&mut self, request: PoolRequest<'_>) -> PoolAnswer {
        match request {
            PoolRequest::Private { name } => self.pool_private(name),
            PoolRequest::Fetch { name, symbolic } => match self.pool_target(name, symbolic) {
                None => PoolAnswer::BadName,
                Some(target) => self.pool_fetch(target),
            },
            PoolRequest::Set {
                name,
                symbolic,
                value,
            } => match self.pool_target(name, symbolic) {
                None | Some(Target::Constant) => PoolAnswer::BadName,
                Some(target) => self.pool_set(target, value),
            },
            PoolRequest::Drop { name, symbolic } => match self.pool_target(name, symbolic) {
                None => PoolAnswer::BadName,
                Some(target) => self.pool_drop(target),
            },
        }
    }

    /// What `name` names: symbolically, upper-cased with its tail's symbols
    /// replaced by their values, as `VALUE` reads a name; directly, as
    /// spelled, its stem part an upper-case symbol and its tail taken as it
    /// stands. `None` for a name that is no variable's.
    fn pool_target(&mut self, name: &[u8], symbolic: bool) -> Option<Target> {
        if name.is_empty() || name.len() > LONGEST_NAME {
            return None;
        }
        if symbolic {
            let upper = name.to_ascii_uppercase();
            return match classify(&upper) {
                SymbolKind::Name => Some(Target::Simple(upper)),
                SymbolKind::Stem => Some(Target::Stem(upper)),
                SymbolKind::CompoundName => {
                    let (stem, key) = self.compound_parts(&upper);
                    Some(Target::Compound { stem, key })
                }
                SymbolKind::Literal if upper[0] == b'.' => Some(Target::Constant),
                SymbolKind::Bad
                | SymbolKind::Numeric
                | SymbolKind::Literal
                | SymbolKind::LiteralDot => None,
            };
        }
        let (head, tail) = match name.iter().position(|&byte| byte == b'.') {
            Some(dot) => (&name[..=dot], Some(&name[dot + 1..])),
            None => (name, None),
        };
        if head.iter().any(u8::is_ascii_lowercase) {
            return None;
        }
        match (classify(head), tail) {
            (SymbolKind::Name, None) => Some(Target::Simple(head.to_vec())),
            (SymbolKind::Stem, Some([])) => Some(Target::Stem(head.to_vec())),
            (SymbolKind::Stem, Some(tail)) => Some(Target::Compound {
                stem: head.to_vec(),
                key: tail.to_vec(),
            }),
            _ => None,
        }
    }

    /// The string `value` answers, or `None` with its condition held on the
    /// running native call.
    fn pool_text(&mut self, value: ObjRef) -> Option<Vec<u8>> {
        let text = self.string_value_text(value);
        self.held(text)
    }

    /// The value `stem`'s variable holds as its default, `None` where it is
    /// unset or has none.
    fn stem_default(&mut self, stem: &[u8]) -> Option<ObjRef> {
        let slot = self.slot_of(stem);
        let frame = self.activation().frame;
        let object = self.variable(frame, slot)?;
        match self.heap.get(object).map(|object| &object.body) {
            Some(Body::Stem { default, .. }) => *default,
            _ => Some(object),
        }
    }

    /// Whether the compound `stem` `key` holds a value of its own or the
    /// stem's default.
    fn compound_value(&mut self, stem: &[u8], key: &[u8]) -> Option<ObjRef> {
        match self.stem_get(stem, key) {
            (value, crate::Novalue::Set) => Some(value),
            (_, crate::Novalue::Unset) => None,
        }
    }

    fn pool_fetch(&mut self, target: Target) -> PoolAnswer {
        let (found, name) = match target {
            Target::Constant => (None, Vec::new()),
            Target::Simple(name) => {
                let slot = self.slot_of(&name);
                let frame = self.activation().frame;
                (self.variable(frame, slot), name)
            }
            Target::Stem(name) => (self.stem_default(&name), name),
            Target::Compound { stem, key } => {
                let found = self.compound_value(&stem, &key);
                let mut name = stem;
                name.extend_from_slice(&key);
                (found, name)
            }
        };
        match found {
            Some(value) => PoolAnswer::Served {
                new: false,
                value: self.pool_text(value),
            },
            None => PoolAnswer::Served {
                new: true,
                value: Some(name),
            },
        }
    }

    fn pool_set(&mut self, target: Target, value: &[u8]) -> PoolAnswer {
        let object = self.text(value);
        self.roots.activity_mut().push_temp(object);
        let existed = match target {
            Target::Constant => return PoolAnswer::BadName,
            Target::Simple(name) => {
                let slot = self.slot_of(&name);
                let frame = self.activation().frame;
                let existed = self.variable(frame, slot).is_some();
                self.set_variable(frame, slot, object);
                existed
            }
            Target::Stem(name) => {
                let existed = self.stem_default(&name).is_some();
                self.stem_assign(&name, object);
                existed
            }
            Target::Compound { stem, key } => {
                let existed = self.compound_value(&stem, &key).is_some();
                self.stem_set(&stem, &key, object);
                existed
            }
        };
        PoolAnswer::Served {
            new: !existed,
            value: None,
        }
    }

    fn pool_drop(&mut self, target: Target) -> PoolAnswer {
        let existed = match target {
            Target::Constant => false,
            Target::Simple(name) => {
                let slot = self.slot_of(&name);
                let frame = self.activation().frame;
                let existed = self.variable(frame, slot).is_some();
                self.clear_variable(frame, slot);
                existed
            }
            Target::Stem(name) => {
                let existed = self.stem_default(&name).is_some();
                self.stem_drop(&name);
                existed
            }
            Target::Compound { stem, key } => {
                let existed = self.compound_value(&stem, &key).is_some();
                self.stem_drop_tail(&stem, &key);
                existed
            }
        };
        PoolAnswer::Served {
            new: !existed,
            value: None,
        }
    }

    /// `RXSHV_PRIV`: `PARM`, `PARM.n`, `SOURCE`, `VERSION` and `QUENAME`,
    /// spelled in upper case.
    fn pool_private(&mut self, name: &[u8]) -> PoolAnswer {
        let value = match name {
            b"VERSION" => crate::version::VERSION.to_vec(),
            b"SOURCE" => {
                let mut source = crate::version::PLATFORM.to_vec();
                source.push(b' ');
                source.extend_from_slice(self.activation().call_type.token());
                source.push(b' ');
                let program = self.activation().program_id;
                source.extend_from_slice(self.program_display_name(program));
                source
            }
            b"QUENAME" => match self.current_queue_name() {
                Ok(name) => name,
                Err(failure) => {
                    self.hold_native_condition(failure);
                    return PoolAnswer::Served {
                        new: false,
                        value: None,
                    };
                }
            },
            b"PARM" => self
                .activation()
                .invoked_with()
                .len()
                .to_string()
                .into_bytes(),
            _ => {
                let Some(index) = name
                    .strip_prefix(b"PARM.")
                    .filter(|digits| !digits.is_empty() && digits.iter().all(u8::is_ascii_digit))
                    .and_then(|digits| std::str::from_utf8(digits).ok()?.parse::<usize>().ok())
                    .filter(|&index| index > 0)
                else {
                    return PoolAnswer::BadName;
                };
                let argument = self
                    .activation()
                    .invoked_with()
                    .get(index - 1)
                    .copied()
                    .flatten();
                match argument {
                    Some(argument) => match self.pool_text(argument) {
                        Some(text) => text,
                        None => {
                            return PoolAnswer::Served {
                                new: false,
                                value: None,
                            };
                        }
                    },
                    None => Vec::new(),
                }
            }
        };
        PoolAnswer::Served {
            new: false,
            value: Some(value),
        }
    }

    /// Every variable of the running activation as `RXSHV_NEXTV` answers it.
    /// A stem answers its default under its own name where it has one, then
    /// each tail holding a value; measured, the oracle answers a stem with a
    /// default and no such tail twice.
    pub(crate) fn pool_variables(&mut self) -> Vec<(Vec<u8>, Vec<u8>)> {
        let entries = match crate::dispatch::context::local_variable_entries(self, 0) {
            Ok(entries) => entries,
            Err(failure) => {
                self.hold_native_condition(failure);
                return Vec::new();
            }
        };
        let mut answered = Vec::new();
        for (name, value) in entries {
            let stem = match self.heap.get(value).map(|object| &object.body) {
                Some(Body::Stem { default, tails, .. }) => {
                    let mut held: Vec<(usize, Vec<u8>, ObjRef)> = tails
                        .iter()
                        .filter_map(|(tail, (ordinal, value))| {
                            value.map(|value| (*ordinal, tail.clone(), value))
                        })
                        .collect();
                    held.sort_by_key(|(ordinal, _, _)| *ordinal);
                    Some((*default, held))
                }
                _ => None,
            };
            let Some((default, tails)) = stem else {
                if let Some(text) = self.pool_text(value) {
                    answered.push((name, text));
                }
                continue;
            };
            let default = default.and_then(|default| self.pool_text(default));
            if let Some(text) = &default {
                answered.push((name.clone(), text.clone()));
            }
            let lone = tails.is_empty();
            for (_, tail, value) in tails {
                if let Some(text) = self.pool_text(value) {
                    let mut compound = name.clone();
                    compound.extend_from_slice(&tail);
                    answered.push((compound, text));
                }
            }
            if lone && let Some(text) = default {
                answered.push((name, text));
            }
        }
        answered
    }

    /// `RexxCallBack`: the running activation's label `label`, upper-cased,
    /// run as an internal function over `arguments`. A condition or an
    /// `EXIT` is held on the classic handler's native frame, which re-raises
    /// it once the handler is unwound.
    pub(crate) fn call_back(&mut self, label: &[u8], arguments: &[Option<Vec<u8>>]) -> CallBack {
        let upper = label.to_ascii_uppercase();
        let Ok(Some(Resolved::Label(target))) = self.resolve_fixed_call(&upper, true) else {
            return CallBack::NoLabel;
        };
        let mut values = Vec::with_capacity(arguments.len());
        for argument in arguments {
            values.push(argument.as_deref().map(|bytes| {
                let object = self.text(bytes);
                self.roots.activity_mut().push_temp(object);
                object
            }));
        }
        let ended = self.invoke_call_over(
            Resolved::Label(target),
            &upper,
            values,
            CallType::Function,
            CallEntry::Written,
        );
        let failure = match ended {
            Ok(Ended::Returned(value)) => {
                let Some(value) = value else {
                    return CallBack::Returned(None);
                };
                self.roots.activity_mut().push_temp(value);
                match self.string_value_text(value) {
                    Ok(text) => return CallBack::Returned(Some(text)),
                    Err(failure) => failure,
                }
            }
            Ok(Ended::Exited(value)) => Failure::Exited(value),
            Err(failure) => failure,
        };
        self.hold_native_condition(failure);
        CallBack::Ended
    }
}
