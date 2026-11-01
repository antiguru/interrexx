use super::*;

/// `Class~defineMethods(methods)`: install a whole table of instance methods
/// in one mutation -- `RexxClass::defineMethodsRexx`
/// (`classes/ClassClass.cpp:518`).
pub(in crate::dispatch) fn native_define_methods(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    rexx_defined_lock(interp, class)?;
    let Some(Some(table)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("methods").into());
    };
    let pairs = method_table_pairs(interp, table)?;
    let mut entries: Vec<(Box<[u8]>, Option<ObjRef>)> = Vec::with_capacity(pairs.len());
    for (name, value) in pairs {
        if value == ObjRef::NIL {
            entries.push((name, None));
            continue;
        }
        let value = if interp.receiver_kind(value) == Ok(Primitive::Method) {
            value
        } else {
            // The entry's own index is the method's name, as stored:
            // `createMethodDictionary` builds the object under
            // `supplier->index()->requestString()` and keys the dictionary
            // under its upcase (`classes/ClassClass.cpp:1255`-`:1258`).
            compile_method_source(interp, &name, value, "method source", SourceTaker::Method)?
        };
        entries.push((name, Some(value)));
    }
    interp
        .define_method_table(class, &entries)
        .ok_or_else(|| Failure::from(Loud::unrecorded_executable()))?;
    Ok(None)
}

/// Whether `value` is a table `~defineMethods`, `~enhanced` and `~setMethod`
/// can walk: one of this crate's own `Body::Native` directories, or a
/// `Directory` or `StringTable` a program made, which since Phase 5h Task 4
/// is a collection with a hash store.
fn string_keyed_table(interp: &mut Interp, value: ObjRef) -> bool {
    if matches!(
        interp.receiver_kind(value),
        Ok(Primitive::Directory | Primitive::StringTable(_))
    ) {
        return true;
    }
    hash::owns(interp, value) && hash::string_keyed(interp, value)
}

/// The (name, value) pairs `createMethodDictionary`
/// (`classes/ClassClass.cpp:1242`) obtains from `table`'s `SUPPLIER`, each
/// index as its string value, in supplier order. Native directories using
/// the compatibility path return their own entries sorted by name.
pub(super) fn method_table_pairs(
    interp: &mut Interp,
    table: ObjRef,
) -> Result<Vec<(Box<[u8]>, ObjRef)>, Failure> {
    if string_keyed_table(interp, table) {
        // **Asked before the walk**, because the walk cannot see the
        // difference: a collection whose entries this crate answers per name
        // rather than storing looks empty to `native_keys`, and a mutation
        // over an empty walk is a mutation that silently does nothing.
        if let Some(owner) = interp.unbuilt_collection_owner(table) {
            return Err(Loud::unreadable_collection(owner).into());
        }
    }
    // A user SUPPLIER can send protected methods or install methods
    // recursively. Run that method under the normal dispatch context.
    // Native directories have no hash store for the primitive supplier to
    // walk. Keep their existing walk only for an unrestricted primitive.
    let native_table = matches!(
        interp.heap.get(table).map(|object| &object.body),
        Some(Body::Native(_))
    ) && string_keyed_table(interp, table);
    let plain_supplier = native_table
        && interp
            .lookup(table, b"SUPPLIER", None)
            .is_some_and(|found| {
                interp.object_model().natives.get(found.method).is_some()
                    && interp.access_scope_of(found.method).is_none()
            });
    if plain_supplier {
        let mut names = interp.native_keys(table);
        names.sort();
        let mut pairs = Vec::with_capacity(names.len());
        for name in names {
            let value = interp.native_entry(table, &name).unwrap_or(ObjRef::NIL);
            pairs.push((name, value));
        }
        return Ok(pairs);
    }
    let caller = interp.caller();
    let supplier = interp
        .send_message(table, b"SUPPLIER", None, &[], caller)?
        .unwrap_or(ObjRef::NIL);
    interp.roots.activity_mut().push_temp(supplier);
    let mut pairs = Vec::new();
    for (index, item) in crate::dispatch::collection::supplier::drain_supplier(interp, supplier)? {
        let name = interp.string_value_text(index)?.into_boxed_slice();
        pairs.push((name, item));
    }
    Ok(pairs)
}
