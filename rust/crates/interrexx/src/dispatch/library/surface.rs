//! The [`Surface`] an extension's callbacks reach this interpreter through.

use std::borrow::Cow;

use interrexx_api::callbacks::{BufferKind, Surface};
use interrexx_api::layout::POINTER;
use interrexx_api::values::Numeric;
use interrexx_core::{
    BehaviourId, Body, BufferState, Bytes, Decoded, NativeState, ObjRef, ScopePools, VarRef,
    VarRefHome,
};

use super::super::hash;
use crate::Novalue;
use crate::builtin::datatype::{SymbolKind, classify};

use crate::error::{Raised, displayable};
use crate::plan::Package;
use crate::{Failure, Interp};

impl Surface for Interp {
    fn clear_condition(&mut self) {
        let frame = self.native_frame_mut();
        frame.raised = None;
        frame.additional = None;
        frame.result = None;
        frame.condition = None;
        frame.callee = false;
        frame.levels = None;
    }

    fn new_array(&mut self, items: &[Option<ObjRef>]) -> ObjRef {
        self.alloc_charged(
            BehaviourId::ARRAY,
            Body::Array {
                dimensions: None,
                slots: items.to_vec().into(),
            },
        )
    }

    fn raise_exception(&mut self, number: usize, substitutions: Option<ObjRef>) {
        // Each item's `stringValue()`, the default name where that raises, and
        // nothing for an empty slot, which is what message substitution does.
        let items = substitutions
            .and_then(|array| self.array_slots_of(array))
            .unwrap_or_default();
        let texts: Vec<Vec<u8>> = items
            .into_iter()
            .map(|item| {
                item.map(|object| self.native_found(object))
                    .unwrap_or_default()
            })
            .collect();
        let raised = crate::run::raised_for_code(number as u64, texts);
        // An unknown code raises 98.941, whose one substitution is the code.
        let additional = if raised.number == 98 && raised.sub == 941 && number != 98_941 {
            None
        } else {
            substitutions
        };
        let frame = self.native_frame_mut();
        frame.raised = Some(raised.into());
        frame.additional = additional;
        frame.result = None;
        frame.condition = None;
        frame.callee = false;
        frame.levels = None;
    }

    fn raise_condition(
        &mut self,
        name: &[u8],
        description: Option<ObjRef>,
        additional: Option<ObjRef>,
        result: Option<ObjRef>,
    ) {
        let name = name.to_ascii_uppercase();
        // `RaiseCondition` has no error number to give a `SYNTAX` condition.
        if name == b"SYNTAX" {
            self.hold_api_misuse(
                b"RaiseCondition cannot raise a SYNTAX condition; use RaiseException or RaiseConditionObject",
            );
            return;
        }
        let description = description.map(|text| self.unsent_string_value_text(text));
        if let Some(escalated) = self.options_escalation(&name, description.as_deref(), result) {
            self.hold_raised(escalated, None, None, None);
            return;
        }
        let raised = Raised {
            description,
            ..Raised::condition(Cow::Owned(String::from_utf8_lossy(&name).into_owned()))
        };
        self.hold_raised(raised, additional, result, None);
    }

    fn has_condition(&mut self) -> bool {
        self.native_frame().raised.is_some()
    }

    fn condition_object(&mut self) -> Option<ObjRef> {
        let frame = self.native_frame();
        if let Some(object) = frame.condition {
            return Some(object);
        }
        let Some(Failure::Raised(raised)) = &frame.raised else {
            return None;
        };
        let (raised, additional, result) = (raised.clone(), frame.additional, frame.result);
        let levels = frame.levels.clone();
        self.activity.pending_additional = additional;
        self.activity.pending_result = result;
        let object = self
            .build_native_condition_object(&raised, levels.as_ref())
            .ok()?;
        self.native_frame_mut().condition = Some(object);
        Some(object)
    }

    fn display_condition(&mut self) -> isize {
        let Some(object) = self.condition_object() else {
            return 0;
        };
        let condition = self.condition_entry(object, b"CONDITION");
        if condition
            .map(|name| self.unsent_string_value_text(name))
            .as_deref()
            != Some(b"SYNTAX")
        {
            return 0;
        }
        let report = self.condition_report(object);
        self.write_trace_report(&report);
        // The error number divided by 1000, where `RC` is not a number.
        self.condition_entry(object, b"RC")
            .and_then(|rc| {
                std::str::from_utf8(&self.unsent_string_value_text(rc))
                    .ok()?
                    .trim()
                    .parse()
                    .ok()
            })
            .unwrap_or(49)
    }

    fn directory_entry(&mut self, directory: ObjRef, name: &[u8]) -> Option<ObjRef> {
        self.condition_entry(directory, name)
    }

    fn request_string(&mut self, object: ObjRef) -> Option<ObjRef> {
        let string = match self.required_string_or_nil(object) {
            Ok(string) => string,
            Err(failure) => {
                self.hold_native_condition(failure);
                return None;
            }
        };
        if self.string_bytes_of(string) {
            return Some(string);
        }
        let bytes = self.to_text(string).into_owned();
        let string = self.text(&bytes);
        self.roots.activity_mut().push_temp(string);
        Some(string)
    }

    fn is_of_class(&mut self, object: ObjRef, id: &str) -> bool {
        let Some(class) = self.classes().lookup(id) else {
            return false;
        };
        self.class_of_value(object) == Some(class)
    }

    fn is_string(&mut self, object: ObjRef) -> bool {
        // A number is a `RexxInteger` or a `NumberString` in the oracle, which
        // `isString` does not count: measured, `IsString(1+1)` is `0` where
        // `IsString('abc')` is `1`.
        let text = match object.decode() {
            Decoded::Text(_) => true,
            Decoded::Heap { .. } => matches!(
                self.heap.get(object).map(|held| &held.body),
                Some(Body::Text { .. })
            ),
            _ => false,
        };
        text && self.is_of_class(object, "String")
    }

    fn new_raw_string(&mut self, length: usize) -> ObjRef {
        let string = self.alloc_with(
            BehaviourId::STRING,
            Body::Text {
                bytes: Bytes::from_slice(&vec![0; length]),
                num: None,
            },
        );
        self.roots.activity_mut().push_temp(string);
        self.charge_text(length);
        string
    }

    fn finish_string(&mut self, string: ObjRef, made: &[u8], length: usize) {
        if let Some(body) = self.heap.get_mut(string).map(|held| &mut held.body) {
            let before = body.held_bytes();
            if let Body::Text { bytes, num } = body {
                *bytes = Bytes::from_slice(&made[..length]);
                *num = None;
            }
            let after = body.held_bytes();
            self.heap.rehold_body_bytes(before, after);
        }
        // Rewritten where it lies, as the oracle's string data is: a call may
        // hold the address `StringData` answered for the unfinished string.
        // Every byte made is copied and the terminator at the made length
        // stays, as finishing sets only the length.
        match self.kept_strings.get_mut(&string) {
            Some(kept) if made.len() < kept.len() => {
                kept[..made.len()].copy_from_slice(made);
            }
            _ => {
                self.kept_strings.remove(&string);
            }
        }
    }

    fn new_buffer(&mut self, length: usize) -> ObjRef {
        self.native_state_instance("Buffer", NativeState::zeroed(length))
    }

    fn buffer_data(&mut self, buffer: ObjRef) -> Option<(POINTER, usize)> {
        let state = self.native_state_mut(buffer)?;
        let length = state.data()?.len();
        Some((state.data_address()?, length))
    }

    fn new_mutable_buffer(&mut self, capacity: usize) -> ObjRef {
        let mut bytes = Vec::new();
        // An allocation this large fails here as `new_buffer` fails in the
        // oracle; the empty buffer is what a failed reservation leaves.
        let capacity = match bytes.try_reserve_exact(capacity) {
            Ok(()) => capacity,
            Err(_) => 0,
        };
        let buffer = self.native_state_instance(
            "MutableBuffer",
            NativeState::Buffer(BufferState {
                bytes: bytes.into(),
                capacity,
                default_size: capacity,
            }),
        );
        self.charge_growth(capacity);
        buffer
    }

    fn mutable_buffer(&mut self, buffer: ObjRef) -> Option<(POINTER, usize, usize)> {
        let state = self.lent_buffer(buffer)?;
        let before = state.capacity;
        let (address, capacity) = state.writable();
        let length = state.bytes.len();
        // A failed reservation drops the capacity to what the storage holds.
        self.heap.rehold_body_bytes(before, capacity);
        Some((address.cast(), length, capacity))
    }

    fn set_mutable_buffer_length(&mut self, buffer: ObjRef, length: usize) -> Option<usize> {
        let state = self.buffer_mut(buffer)?;
        // Capped at the capacity, and padded with NULs where it grows.
        let length = length.min(state.capacity);
        state.bytes.resize(length, 0);
        Some(length)
    }

    fn set_mutable_buffer_capacity(&mut self, buffer: ObjRef, capacity: usize) -> Option<POINTER> {
        let state = self.lent_buffer(buffer)?;
        // The capacity change, which asks for room for the difference over the
        // length.
        let grown = state.set_capacity(capacity).unwrap_or(0);
        let reserved = state.capacity;
        let address = state.writable().0.cast();
        let shrunk = reserved - state.capacity;
        self.charge_growth(grown);
        // A failed reservation drops the capacity to what the storage holds.
        self.heap.release_body_bytes(shrunk);
        Some(address)
    }

    fn object_cself(&mut self, object: ObjRef, scope: Option<ObjRef>) -> Option<POINTER> {
        let owner = self.pool_owner(object).ok()?;
        let held = match scope {
            None => self.pools_of(owner)?.find(b"CSELF"),
            Some(mut scope) => loop {
                if scope == ObjRef::NIL {
                    break None;
                }
                if let Some(held) = self
                    .pools_of(owner)
                    .and_then(|pools| pools.get(scope, b"CSELF"))
                {
                    break Some(held);
                }
                scope = self.super_scope_of(object, scope)?;
            },
        }?;
        let state = self.native_state_mut(held)?;
        state.pointer().or_else(|| state.data_address())
    }

    fn send(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        scope: Option<ObjRef>,
        arguments: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, ()> {
        let caller = self.caller();
        match self.send_message(receiver, name, scope, arguments, caller) {
            Ok(answer) => {
                if let Some(answer) = answer {
                    self.roots.activity_mut().push_temp(answer);
                }
                Ok(answer)
            }
            Err(failure) => {
                self.hold_native_condition(failure);
                Err(())
            }
        }
    }

    fn store_get(&mut self, table: ObjRef, index: &[u8]) -> Option<Result<Option<ObjRef>, ()>> {
        if !hash::owns(self, table) {
            return None;
        }
        let index = self.text(index);
        self.roots.activity_mut().push_temp(index);
        let answered = hash::store_get(self, table, index);
        Some(self.served(answered))
    }

    fn store_put(&mut self, table: ObjRef, index: &[u8], item: ObjRef) -> Option<Result<(), ()>> {
        if !hash::owns(self, table) {
            return None;
        }
        match hash::native_put(self, table, index, item) {
            Ok(()) => Some(Ok(())),
            Err(failure) => {
                self.hold_native_condition(failure);
                Some(Err(()))
            }
        }
    }

    fn store_remove(&mut self, table: ObjRef, index: &[u8]) -> Option<Result<Option<ObjRef>, ()>> {
        if !hash::owns(self, table) {
            return None;
        }
        let index = self.text(index);
        self.roots.activity_mut().push_temp(index);
        let answered = hash::store_remove(self, table, index);
        Some(self.served(answered))
    }

    fn class_object(&mut self, id: &str) -> Option<ObjRef> {
        self.classes().lookup(id)
    }

    fn context_variable(&mut self, name: &[u8]) -> Option<ObjRef> {
        if self.native_callerless() {
            return None;
        }
        let upper = name.to_ascii_uppercase();
        match classify(&upper) {
            SymbolKind::Bad => None,
            // A constant symbol is its own value.
            SymbolKind::Numeric | SymbolKind::Literal => Some(self.text(&upper)),
            SymbolKind::LiteralDot => self.dot_variable(&upper).ok(),
            SymbolKind::Name => {
                let slot = self.slot_of(&upper);
                let frame = self.activation().frame;
                self.variable(frame, slot)
            }
            SymbolKind::Stem => Some(self.read_stem(&upper)),
            SymbolKind::CompoundName => {
                let (stem, key) = self.compound_parts(&upper);
                match self.stem_get(&stem, &key) {
                    (value, Novalue::Set) => Some(value),
                    _ => None,
                }
            }
        }
    }

    fn set_context_variable(&mut self, name: &[u8], value: ObjRef) {
        if self.native_callerless() {
            return;
        }
        let upper = name.to_ascii_uppercase();
        match classify(&upper) {
            SymbolKind::Name => {
                let slot = self.slot_of(&upper);
                let frame = self.activation().frame;
                self.set_variable(frame, slot, value);
            }
            SymbolKind::Stem => self.stem_assign(&upper, value),
            SymbolKind::CompoundName => {
                let (stem, key) = self.compound_parts(&upper);
                self.stem_set(&stem, &key, value);
            }
            _ => {}
        }
    }

    fn drop_context_variable(&mut self, name: &[u8]) {
        if self.native_callerless() {
            return;
        }
        let upper = name.to_ascii_uppercase();
        match classify(&upper) {
            SymbolKind::Name => {
                let slot = self.slot_of(&upper);
                let frame = self.activation().frame;
                self.clear_variable(frame, slot);
            }
            SymbolKind::Stem => self.stem_drop(&upper),
            SymbolKind::CompoundName => {
                let (stem, key) = self.compound_parts(&upper);
                self.stem_drop_tail(&stem, &key);
            }
            _ => {}
        }
    }

    fn context_variables(&mut self) -> Option<ObjRef> {
        let listed = match self.native_callerless() {
            true => crate::dispatch::hash::new_directory(self, 0),
            false => crate::dispatch::context::local_variables(self, 0),
        };
        match listed {
            Ok(directory) => Some(directory),
            Err(failure) => {
                self.hold_native_condition(failure);
                None
            }
        }
    }

    fn variable_pool(
        &mut self,
        request: interrexx_api::callbacks::PoolRequest<'_>,
    ) -> interrexx_api::callbacks::PoolAnswer {
        self.pool_request(request)
    }

    fn pool_variables(&mut self) -> Vec<(Vec<u8>, Vec<u8>)> {
        Interp::pool_variables(self)
    }

    fn call_back(
        &mut self,
        label: &[u8],
        arguments: &[Option<Vec<u8>>],
    ) -> interrexx_api::callbacks::CallBack {
        Interp::call_back(self, label, arguments)
    }

    fn share_session_queue(&mut self) {
        let shared = Interp::share_session_queue(self);
        self.held(shared);
    }

    fn in_caller(&mut self, frame: u64, serve: &mut dyn FnMut(&mut dyn Surface)) {
        match self.suspended_caller(frame) {
            Some(index) => serve(&mut *CallerSwap::new(self, index)),
            None => serve(self),
        }
    }

    fn object_variable(&mut self, name: &[u8]) -> Option<ObjRef> {
        let frame = self.native_frame();
        let (owner, scope) = (frame.owner, frame.scope);
        let name = super::pool_variable_name(name)?;
        if let Some(value) = self.pools_of(owner)?.get(scope, &name) {
            return Some(value);
        }
        // A stem variable always has a value, made on first read.
        if crate::run::shape_of(&name) != crate::run::NameShape::Stem {
            return None;
        }
        let fresh = self.empty_stem(&name);
        self.set_pool_variable(owner, scope, &name, fresh);
        Some(fresh)
    }

    fn set_guard(&mut self, on: bool) {
        if let Err(failure) = self.native_guard(on) {
            self.hold_native_condition(failure);
        }
    }

    fn guard_when_updated(&mut self, name: &[u8], on: bool) -> Option<ObjRef> {
        let frame = self.native_frame();
        if !frame.method {
            return None;
        }
        let (owner, scope) = (frame.owner, frame.scope);
        let upper = name.to_ascii_uppercase();
        if !matches!(classify(&upper), SymbolKind::Name | SymbolKind::Stem) {
            return None;
        }
        self.watch_variable(owner, scope, &upper);
        let waited = self
            .native_guard(on)
            .and_then(|()| self.native_guard_wait());
        self.unwatch_variable(owner, scope, &upper);
        if let Err(failure) = waited {
            self.hold_native_condition(failure);
            return None;
        }
        self.object_variable(&upper)
    }

    fn variable_reference(&mut self, name: &[u8], object: bool) -> Option<ObjRef> {
        let upper = name.to_ascii_uppercase();
        if !matches!(classify(&upper), SymbolKind::Name | SymbolKind::Stem) {
            return None;
        }
        if !object && self.native_callerless() {
            return None;
        }
        let home = if object {
            let frame = self.native_frame();
            VarRefHome::Instance {
                owner: frame.owner,
                scope: frame.scope,
            }
        } else {
            let slot = self.slot_of(&upper);
            let frame = self.activation().frame;
            match self.exposure(frame, slot) {
                Some(var) => VarRefHome::Instance {
                    owner: var.owner,
                    scope: var.scope,
                },
                None => VarRefHome::Cell(self.promote_slot(frame, slot, &upper)),
            }
        };
        let reference = self.alloc_with(
            BehaviourId::OBJECT,
            Body::VarRef(Box::new(VarRef {
                name: upper.into(),
                home,
            })),
        );
        self.roots.activity_mut().push_temp(reference);
        // As `variable_reference` arms it for every reference.
        self.reqstr_armed = true;
        Some(reference)
    }

    fn find_class(&mut self, name: &[u8], executable: bool) -> Option<ObjRef> {
        let frame = self.native_frame();
        // A routine a library registered has no package of its own.
        let unpackaged = executable
            && !frame.method
            && frame
                .code
                .is_none_or(|code| self.library_code_package_path(code).is_none());
        if unpackaged {
            let found = self.system_symbol(name)?;
            return self.is_class_object(found).then_some(found);
        }
        // A method's executable is its own package's, which is the package
        // that defined its scope.
        if frame.method
            && let Some(crate::plan::ClassPackage::Program(program)) =
                self.class_packages.get(&frame.scope).copied()
        {
            let lookup = self.class_in_program(program, name);
            let found = self.held(lookup)??;
            return self.is_class_object(found).then_some(found);
        }
        self.class_named(name)
    }

    fn environment(&mut self, local: bool) -> Option<ObjRef> {
        let name: &[u8] = if local { b".LOCAL" } else { b".ENVIRONMENT" };
        self.dot_variable(name).ok()
    }

    fn executable(&mut self) -> Option<ObjRef> {
        let row = self.activity.native_handles.len().checked_sub(1)?;
        self.executable_at(row)
    }

    fn kept_numeric(&mut self, frame: u64) -> Option<Numeric> {
        self.with_native_owner(frame, |interp| {
            let activation = interp.running_activation()?;
            Some(super::numeric_of(&activation.settings))
        })
    }

    fn kept_executable(&mut self, frame: u64) -> Option<ObjRef> {
        let object = self.with_native_owner(frame, |interp| {
            let running = interp.running_activation()?.id;
            let top = interp.activity.native_handles.len().checked_sub(1);
            match top.filter(|&row| interp.activity.native_handles[row].caller == Some(running)) {
                Some(row) => interp.executable_at(row),
                None => crate::dispatch::context::executable_at(interp, 0).ok(),
            }
        })?;
        self.roots.activity_mut().push_temp(object);
        Some(object)
    }

    fn caller_context(&mut self) -> Option<ObjRef> {
        self.context_object_at(0)
    }

    fn array_items(&mut self, array: ObjRef) -> Option<Vec<Option<ObjRef>>> {
        self.array_slots_of(array)
    }

    fn load_package(&mut self, name: &[u8]) -> Option<ObjRef> {
        let loaded = self.load_package_global(name);
        let program = self.held(loaded)?;
        let package = self.package_object(Package::Program(program));
        self.held(package)
    }

    fn load_package_source(&mut self, name: &[u8], lines: &[Vec<u8>]) -> Option<ObjRef> {
        let loaded = self.package_from_source(name, lines, None, true);
        let program = self.held(loaded)?;
        let package = self.package_object(Package::Program(program));
        self.held(package)
    }

    fn compile_buffer(
        &mut self,
        kind: BufferKind,
        name: &[u8],
        data: &[u8],
        lines: &[Vec<u8>],
    ) -> Option<Option<ObjRef>> {
        let source = match kind {
            // Method creation from a buffer parses even lines `.Method~new`
            // would restore.
            BufferKind::Method => {
                let borrowed: Vec<&[u8]> = lines.iter().map(Vec::as_slice).collect();
                Ok(interrexx_parse::ProgramSource::from_lines(&borrowed))
            }
            BufferKind::Routine => crate::compiled::program_source(data.to_vec(), name),
            // A package this instance already loaded under the name, from
            // data or a file, is answered as it is.
            BufferKind::Package if let Some(&program) = self.required_packages.get(name) => {
                let package = self.package_object(Package::Program(program));
                return Some(self.held(package));
            }
            BufferKind::Package => match crate::compiled::restore(data) {
                crate::compiled::Restore::Source => return None,
                _ => crate::compiled::program_source(data.to_vec(), name),
            },
        };
        Some(self.compile_buffer_source(kind, name, source))
    }

    fn load_library(&mut self, name: &[u8]) -> bool {
        match self.resolve_library(name) {
            crate::LibraryLoad::Loaded(_) => true,
            crate::LibraryLoad::Missing => false,
            crate::LibraryLoad::Version => {
                let raised = Raised::library_version(name);
                self.hold_native_condition(raised.into());
                false
            }
            crate::LibraryLoad::Raised(failure) => {
                self.hold_native_condition(failure);
                false
            }
        }
    }

    fn call_program(&mut self, name: &[u8], arguments: &[Option<ObjRef>]) -> Option<ObjRef> {
        let Some(path) = self.resolve_program_name(None, name, false) else {
            let raised = Raised::syntax(3, 901, vec![name.to_vec()]);
            self.hold_native_condition(raised.into());
            return None;
        };
        let routine = self.new_file_executable(&path, true, None, true);
        let routine = self.held(routine)?;
        self.roots.activity_mut().push_temp(routine);
        let (program, directive) = self.executable_sources.get(&routine)?.routine?;
        let installed = crate::InstalledRoutine { program, directive };
        let answered =
            self.on_api_base(|interp| interp.run_routine_as_program(installed, arguments.to_vec()));
        self.held(answered).flatten()
    }

    fn call_routine(&mut self, routine: ObjRef, arguments: Option<ObjRef>) -> Option<ObjRef> {
        let answered = self.on_api_base(|interp| {
            crate::dispatch::executable::call_routine_directly(interp, routine, arguments)
        });
        let answer = self.held(answered).flatten();
        if let Some(answer) = answer {
            self.roots.activity_mut().push_temp(answer);
        }
        answer
    }

    fn global_reference(&mut self, object: ObjRef) {
        self.global_references.register(object);
    }

    fn allocate_object_memory(&mut self, size: usize) -> Option<POINTER> {
        let buffer = Surface::new_buffer(self, size);
        let mut table = self.object_memory()?;
        table.push(Some(buffer));
        self.set_object_memory(table)?;
        self.native_state_mut(buffer)?.data_address()
    }

    fn free_object_memory(&mut self, pointer: POINTER) {
        let Some(mut table) = self.object_memory() else {
            return;
        };
        table.retain(|buffer| {
            buffer.and_then(|buffer| self.buffer_address(buffer)) != Some(pointer)
        });
        self.set_object_memory(table);
    }

    fn reallocate_object_memory(&mut self, pointer: POINTER, size: usize) -> Option<POINTER> {
        let table = self.object_memory()?;
        let old = table
            .iter()
            .flatten()
            .copied()
            .find(|buffer| self.buffer_address(*buffer) == Some(pointer))?;
        let bytes = self.native_state_mut(old)?.data()?.to_vec();
        if size <= bytes.len() {
            return Some(pointer);
        }
        let grown = Surface::allocate_object_memory(self, size)?;
        let buffer = self.object_memory()?.last().copied().flatten()?;
        if let Some(NativeState::Data(data)) = self.native_state_mut(buffer) {
            data.write(0, &bytes);
        }
        Surface::free_object_memory(self, pointer);
        Some(grown)
    }

    fn register_library(
        &mut self,
        name: &[u8],
        library: Result<Option<interrexx_api::load::Library>, interrexx_api::load::Refused>,
    ) -> bool {
        if self.libraries.get(name).is_some() {
            return false;
        }
        match self.settle_library(name, library) {
            crate::LibraryLoad::Loaded(_) => true,
            crate::LibraryLoad::Missing => false,
            crate::LibraryLoad::Version => {
                let raised = Raised::library_version(name);
                self.hold_native_condition(raised.into());
                false
            }
            crate::LibraryLoad::Raised(failure) => {
                self.hold_native_condition(failure);
                false
            }
        }
    }

    /// the handler replaces whatever the name had, stacked handlers included.
    fn add_command_handler(&mut self, name: &[u8], handler: interrexx_api::load::CommandHandler) {
        let upper = name.to_ascii_uppercase();
        self.environments.replace(&upper);
        self.command_handlers
            .insert(upper.into_boxed_slice(), std::rc::Rc::new(handler));
    }

    /// Covers the handler in use under `name`, as `.CommandHandler~register`
    /// covers it: what uncovers it again is `.CommandHandler~deregister`'s.
    fn stack_command_handler(&mut self, name: &[u8], handler: interrexx_api::load::CommandHandler) {
        self.stack_native_environment(name, handler);
    }

    fn halt(&mut self, all: bool) {
        if all {
            self.halt_all();
        } else {
            self.halt_running();
        }
    }

    /// The running Rexx frame's external trace, `TRACE ?R` on and `TRACE O`
    /// off. An idle activity's frame is not reached.
    fn set_trace(&mut self, _all: bool, on: bool) {
        self.set_external_trace(on);
    }

    /// The condition a copy of `condition` describes, with its description,
    /// additional information, result and `RC`; a `SYNTAX` one by its `CODE`,
    /// with the `RC`, `ERRORTEXT` and `MESSAGE` that follow from it. An object
    /// that describes no condition is 93.900.
    fn raise_condition_object(&mut self, condition: ObjRef) -> bool {
        if !Surface::is_of_class(self, condition, "Directory") {
            return false;
        }
        let entry =
            |interp: &mut Interp, name: &[u8]| Surface::directory_entry(interp, condition, name);
        let Some(name) = entry(self, b"CONDITION").filter(|&name| self.is_string(name)) else {
            self.hold_api_misuse(b"the condition object has no CONDITION name");
            return true;
        };
        let name = self.unsent_string_value_text(name).to_ascii_uppercase();
        let description =
            entry(self, b"DESCRIPTION").map(|text| self.unsent_string_value_text(text));
        let additional = entry(self, b"ADDITIONAL");
        let result = entry(self, b"RESULT");
        let raised = if name == b"SYNTAX" {
            let code = entry(self, b"CODE")
                .filter(|&code| self.is_string(code))
                .map(|code| self.unsent_string_value_text(code))
                .and_then(|code| valid_error_code(&code));
            let Some((major, minor)) = code else {
                self.hold_api_misuse(
                    b"the CODE of a SYNTAX condition object must be a valid error number",
                );
                return true;
            };
            let substitutions = match additional {
                None => Vec::new(),
                Some(array) => match self.array_slots_of(array) {
                    Some(items) if Surface::is_of_class(self, array, "Array") => items
                        .into_iter()
                        .map(|item| {
                            item.map(|object| self.native_found(object))
                                .unwrap_or_default()
                        })
                        .collect(),
                    _ => {
                        self.hold_api_misuse(
                            b"the ADDITIONAL of a SYNTAX condition object must be an array",
                        );
                        return true;
                    }
                },
            };
            if interrexx_inventory::errors::lookup(major, 0).is_none() {
                let code = format!("{major}.{minor}").into_bytes();
                self.hold_raised(Raised::syntax(98, 941, vec![code]), None, None, None);
                return true;
            }
            Raised {
                description,
                ..Raised::syntax(major, minor, substitutions)
            }
        } else {
            if let Some(escalated) = self.options_escalation(&name, description.as_deref(), result)
            {
                self.hold_raised(escalated, None, None, None);
                return true;
            }
            let rc = entry(self, b"RC").map(|rc| self.unsent_string_value_text(rc));
            Raised {
                description,
                rc,
                ..Raised::condition(Cow::Owned(String::from_utf8_lossy(&name).into_owned()))
            }
        };
        self.hold_raised(raised, additional, result, None);
        true
    }
}

/// `code` as an error number, a major part from 1 to 99 and a minor part from
/// 0 to 999.
fn valid_error_code(code: &[u8]) -> Option<(u16, u16)> {
    let whole = |part: &[u8]| {
        interrexx_num::Number::parse_bytes(part).and_then(|number| number.int64_value(9))
    };
    let (major, minor) = match code.iter().position(|&byte| byte == b'.') {
        Some(point) => (&code[..point], Some(&code[point + 1..])),
        None => (code, None),
    };
    let major = u16::try_from(whole(major)?)
        .ok()
        .filter(|major| (1..100).contains(major))?;
    let minor = match minor {
        Some(minor) => u16::try_from(whole(minor)?)
            .ok()
            .filter(|&minor| minor < 1000)?,
        None => 0,
    };
    Some((major, minor))
}

impl Interp {
    /// Holds `raised` on the running native frame, with the objects its
    /// condition object names, for the call to raise once it returns.
    fn hold_raised(
        &mut self,
        raised: Raised,
        additional: Option<ObjRef>,
        result: Option<ObjRef>,
        condition: Option<ObjRef>,
    ) {
        let frame = self.native_frame_mut();
        frame.raised = Some(raised.into());
        frame.additional = additional;
        frame.result = result;
        frame.condition = condition;
        frame.callee = false;
        frame.levels = None;
    }

    /// The 93.900 user-defined-method error with `message`, which the
    /// condition APIs raise for a request they cannot serve.
    fn hold_api_misuse(&mut self, message: &[u8]) {
        self.hold_raised(
            Raised::syntax(93, 900, vec![message.to_vec()]),
            None,
            None,
            None,
        );
    }

    /// the `SYNTAX` error the calling activation's `::OPTIONS` make of the
    /// condition `name` a native call raises, with its description and, for
    /// `ERROR` and `FAILURE`, its result as the return code.
    fn options_escalation(
        &mut self,
        name: &[u8],
        description: Option<&[u8]>,
        result: Option<ObjRef>,
    ) -> Option<Raised> {
        let options = matches!(
            name,
            b"ERROR" | b"FAILURE" | b"LOSTDIGITS" | b"NOSTRING" | b"NOTREADY"
        );
        if !options || self.native_callerless() || !self.condition_raises_syntax(name) {
            return None;
        }
        let rc = match result {
            Some(result) if matches!(name, b"ERROR" | b"FAILURE") => {
                self.unsent_string_value_text(result)
            }
            _ => Vec::new(),
        };
        Some(crate::command::escalated(
            name,
            description.unwrap_or_default(),
            &rc,
        ))
    }

    /// `buffer`'s state, lent to the innermost native call: its storage is
    /// not reallocated in place until that call ends.
    fn lent_buffer(&mut self, buffer: ObjRef) -> Option<&mut BufferState> {
        self.buffer(buffer)?;
        let mut lend = false;
        if let Some(frame) = self.activity.native_handles.last_mut()
            && !frame.lent.contains(&buffer)
        {
            frame.lent.push(buffer);
            lend = true;
        }
        let state = self.buffer_mut(buffer)?;
        if lend {
            state.bytes.lend();
        }
        Some(state)
    }

    /// A direct collection member's answer as the surface reports it, rooted,
    /// or its condition held.
    fn served(&mut self, answered: Result<Option<ObjRef>, Failure>) -> Result<Option<ObjRef>, ()> {
        match answered {
            Ok(answer) => {
                if let Some(answer) = answer {
                    self.roots.activity_mut().push_temp(answer);
                }
                Ok(answer)
            }
            Err(failure) => {
                self.hold_native_condition(failure);
                Err(())
            }
        }
    }

    /// The buffers the running method's receiver keeps for
    /// `AllocateObjectMemory`, which the oracle keeps in the receiver's
    /// `Object`-scope pool under the empty name, or `None` outside a method.
    fn object_memory(&mut self) -> Option<Vec<Option<ObjRef>>> {
        let frame = self.native_frame();
        if !frame.method {
            return None;
        }
        let owner = frame.owner;
        let scope = self.classes().lookup("Object")?;
        match self.pools_of(owner).and_then(|pools| pools.get(scope, b"")) {
            Some(table) => self.array_slots_of(table),
            None => Some(Vec::new()),
        }
    }

    /// Replaces [`Interp::object_memory`]'s buffers.
    fn set_object_memory(&mut self, buffers: Vec<Option<ObjRef>>) -> Option<()> {
        let owner = self.native_frame().owner;
        let scope = self.classes().lookup("Object")?;
        let table = Surface::new_array(self, &buffers);
        self.roots.activity_mut().push_temp(table);
        self.set_pool_variable(owner, scope, b"", table);
        Some(())
    }

    /// The address of a `Buffer`'s bytes.
    fn buffer_address(&mut self, buffer: ObjRef) -> Option<POINTER> {
        self.native_state_mut(buffer)?.data_address()
    }

    /// [`Surface::compile_buffer`] past its choice of source: the executable
    /// or package `source` becomes, or `None` with the condition held.
    #[cold]
    #[inline(never)]
    fn compile_buffer_source(
        &mut self,
        kind: BufferKind,
        name: &[u8],
        source: Result<interrexx_parse::ProgramSource, Raised>,
    ) -> Option<ObjRef> {
        let parsed = match source {
            Ok(source) => interrexx_parse::program_from(source)
                .map_err(|rejected| self.raise_parse_failure(&rejected, Some(name))),
            Err(raised) => Err(raised.into()),
        };
        // `NewMethod` and `NewRoutine` translate with no source context, so
        // the executable's package has no parent.
        let built = parsed.and_then(|parsed| match kind {
            BufferKind::Method => {
                super::super::class_protocol::method_in_package(self, name, parsed, None)
            }
            BufferKind::Routine => {
                super::super::class_protocol::routine_in_package(self, name, parsed, None)
            }
            BufferKind::Package => self
                .package_from_program(name, std::rc::Rc::new(parsed), None, true)
                .and_then(|program| self.package_object(Package::Program(program))),
        });
        self.held(built)
    }

    /// The value `result` carries, or `None` with its condition held on the
    /// running native call.
    pub(super) fn held<T>(&mut self, result: Result<T, Failure>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(failure) => {
                self.hold_native_condition(failure);
                None
            }
        }
    }
}

impl Interp {
    /// Runs `run`, the code a `CallRoutine` or `CallProgram` call runs, on a
    /// fresh activation stack ([`crate::activity::ApiBase`]).
    fn on_api_base<T>(&mut self, run: impl FnOnce(&mut Interp) -> T) -> T {
        let below = self.running_activation().map(|activation| activation.id);
        let native = self.activity.native_handles.len() - 1;
        self.activity
            .api_bases
            .push(crate::activity::ApiBase { below, native });
        let ran = run(self);
        self.activity.api_bases.pop();
        ran
    }

    /// The class `.name` resolves to from the calling activation, or `None`
    /// where it resolves to something that is not a class or the lookup
    /// raised, whose condition is then held for the native call.
    fn class_named(&mut self, name: &[u8]) -> Option<ObjRef> {
        let mut dotted = Vec::with_capacity(name.len() + 1);
        dotted.push(b'.');
        dotted.extend_from_slice(name);
        let lookup = self.dot_variable(&dotted);
        let found = self.held(lookup)?;
        self.is_class_object(found).then_some(found)
    }

    /// A compound name's stem, with its period, and its tail resolved as
    /// `VALUE` resolves one: each piece that is a symbol replaced by the
    /// variable's value.
    pub(super) fn compound_parts(&mut self, upper: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let dot = upper
            .iter()
            .position(|&byte| byte == b'.')
            .expect("a compound name has a period");
        let stem = upper[..=dot].to_vec();
        let key = crate::builtin::datatype::resolve_compound_key(self, &upper[dot + 1..]);
        (stem, key)
    }

    /// [`Surface::executable`] for the native call at `row`, left on the
    /// temps.
    fn executable_at(&mut self, row: usize) -> Option<ObjRef> {
        let frame = &self.activity.native_handles[row];
        let (method, scope, name, code) =
            (frame.method, frame.scope, frame.name.clone(), frame.code);
        let object = if method {
            self.method_executable(scope, &name).ok()?
        } else {
            self.library_routine_object(code?)
        };
        self.roots.activity_mut().push_temp(object);
        Some(object)
    }

    /// `serve` with the activation `id` of this activity swapped in as the
    /// running one, as a native callback reaches its caller's variables
    /// (`Activity::setCallbackParent`), or `None` where this activity holds no
    /// such activation.
    pub(crate) fn in_activation<R>(
        &mut self,
        id: crate::activation::ActivationId,
        serve: impl FnOnce(&mut Interp) -> R,
    ) -> Option<R> {
        if self.activity.running.as_ref()?.id == id {
            return Some(serve(self));
        }
        let index = self
            .activity
            .suspended
            .iter()
            .position(|held| held.id == id)?;
        let mut swapped = CallerSwap::new(self, index);
        Some(serve(&mut swapped))
    }

    /// The index in [`Activity::suspended`] of the activation that made the
    /// native call [`Interp::native_token`] named `frame`, or `None` where it
    /// is the running one or that call has returned.
    pub(crate) fn suspended_caller(&self, frame: u64) -> Option<usize> {
        let row = usize::try_from(frame >> 32).ok()?;
        let caller = self
            .activity
            .native_handles
            .get(row)
            .filter(|native| u64::from(native.id) == frame & u64::from(u32::MAX))?
            .caller?;
        if self.activity.running.as_ref()?.id == caller {
            return None;
        }
        self.activity
            .suspended
            .iter()
            .position(|held| held.id == caller)
    }
}

impl Interp {
    /// Whether `string` is a string's own value, which a tagged integer, an
    /// inline string and a string or number body are.
    fn string_bytes_of(&self, string: ObjRef) -> bool {
        match string.decode() {
            Decoded::SmallInt(_) | Decoded::Text(_) => true,
            Decoded::Heap { .. } => matches!(
                self.heap.get(string).map(|held| &held.body),
                Some(Body::Text { .. } | Body::Num { .. })
            ),
            _ => false,
        }
    }

    /// An instance of the native class `id` carrying `state`, made as the
    /// C++ constructors make one, with no `INIT` sent.
    fn native_state_instance(&mut self, id: &str, state: NativeState) -> ObjRef {
        let class = self
            .classes()
            .lookup(id)
            .unwrap_or_else(|| unreachable!("{id} is a native class"));
        let behaviour = self.classes().instance_behaviour_handle(class);
        let object = self.alloc_with(
            BehaviourId::OBJECT,
            Body::Instance {
                class,
                behaviour,
                name: None,
                pools: ScopePools::new(),
                own: None,
                native: Some(Box::new(state)),
            },
        );
        self.roots.activity_mut().push_temp(object);
        // As `new_instance` arms it for every instance.
        self.reqstr_armed = true;
        object
    }

    /// The native state `object` carries, for a caller that writes it.
    pub(super) fn native_state_mut(&mut self, object: ObjRef) -> Option<&mut NativeState> {
        match &mut self.heap.get_mut(object)?.body {
            Body::Instance {
                native: Some(state),
                ..
            } => Some(state),
            _ => None,
        }
    }
}

impl Interp {
    /// the condition object's traceback lines, then the error lines its
    /// entries spell.
    fn condition_report(&mut self, object: ObjRef) -> Vec<u8> {
        let mut report = Vec::new();
        if let Some(traceback) = self.condition_entry(object, b"TRACEBACK") {
            let caller = self.caller();
            let lines = self
                .send_message(traceback, b"MAKEARRAY", None, &[], caller)
                .ok()
                .flatten()
                .and_then(|array| self.array_slots_of(array))
                .unwrap_or_default();
            for line in lines.into_iter().flatten() {
                if line != ObjRef::NIL {
                    report.extend_from_slice(&self.unsent_string_value_text(line));
                    report.push(b'\n');
                }
            }
        }
        let entry = |interp: &mut Interp, name: &[u8]| {
            interp
                .condition_entry(object, name)
                .map(|value| interp.unsent_string_value_text(value))
        };
        let rc = entry(self, b"RC").unwrap_or_default();
        report.extend_from_slice(b"Error ");
        report.extend_from_slice(&rc);
        if let Some(program) = entry(self, b"PROGRAM").filter(|program| !program.is_empty()) {
            report.extend_from_slice(b" running ");
            report.extend_from_slice(&program);
            if let Some(position) = entry(self, b"POSITION") {
                report.extend_from_slice(b" line ");
                report.extend_from_slice(&position);
            }
        }
        report.extend_from_slice(b":  ");
        report.extend_from_slice(&entry(self, b"ERRORTEXT").unwrap_or_default());
        report.push(b'\n');
        if let Some(message) = entry(self, b"MESSAGE") {
            report.extend_from_slice(b"Error ");
            report.extend_from_slice(&entry(self, b"CODE").unwrap_or_default());
            report.extend_from_slice(b":  ");
            report.extend_from_slice(&message);
            report.push(b'\n');
        }
        displayable(&mut report);
        report
    }
}

/// The caller of a kept outer call swapped in as [`Activity::running`] for as
/// long as this lives, and swapped back when it drops, an unwind included.
/// Both activations stay in `running` and `suspended`, where the collector
/// reads them.
struct CallerSwap<'a> {
    interp: &'a mut Interp,
    index: usize,
}

impl<'a> CallerSwap<'a> {
    fn new(interp: &'a mut Interp, index: usize) -> CallerSwap<'a> {
        let running = interp
            .activity
            .running
            .as_mut()
            .expect("a native call has a caller");
        std::mem::swap(running, &mut interp.activity.suspended[index]);
        CallerSwap { interp, index }
    }
}

impl Drop for CallerSwap<'_> {
    fn drop(&mut self) {
        let running = self
            .interp
            .activity
            .running
            .as_mut()
            .expect("the swapped-in caller");
        std::mem::swap(running, &mut self.interp.activity.suspended[self.index]);
    }
}

impl std::ops::Deref for CallerSwap<'_> {
    type Target = Interp;
    fn deref(&self) -> &Interp {
        self.interp
    }
}

impl std::ops::DerefMut for CallerSwap<'_> {
    fn deref_mut(&mut self) -> &mut Interp {
        self.interp
    }
}
