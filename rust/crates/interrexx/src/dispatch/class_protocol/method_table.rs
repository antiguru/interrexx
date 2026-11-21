use super::*;

/// `Class~defineMethods(methods)`: install a whole table of instance methods
/// in one mutation.
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
            // The entry's own index is the method's name, as stored: the
            // method object is built under the index's string value and the
            // dictionary is keyed under its upcase.
            compile_method_source(interp, &name, value, "method source", SourceTaker::Method)?
        };
        entries.push((name, Some(value)));
    }
    interp
        .define_method_table(class, &entries)
        .ok_or_else(|| Failure::from(Loud::unrecorded_executable()))?;
    Ok(None)
}

/// The (name, value) pairs `createMethodDictionary` obtains from `table`'s
/// `SUPPLIER`, each index as its string value, in supplier order.
pub(super) fn method_table_pairs(
    interp: &mut Interp,
    table: ObjRef,
) -> Result<Vec<(Box<[u8]>, ObjRef)>, Failure> {
    // A walk cannot see entries the interpreter answers by name rather than
    // storing. A mutation over that empty walk would silently do nothing.
    if hash::owns(interp, table)
        && hash::string_keyed(interp, table)
        && interp.holds_unbuilt_entry(table)
    {
        return Err(Loud::unreadable_collection().into());
    }
    // A user SUPPLIER can send protected methods or install methods
    // recursively. Run that method under the normal dispatch context.
    let caller = interp.caller();
    let supplier = interp
        .send_message(table, b"SUPPLIER", None, &[], caller)?
        .unwrap_or(ObjRef::NIL);
    interp.roots.activity_mut().push_temp(supplier);
    // Only a `Supplier`, an instance of a subclass included, is read: any
    // other answer, `.nil` or none among them, is 98.994 naming the object
    // the methods were taken from (`r[class.class.methods-supplier]`).
    if !is_supplier(interp, supplier) {
        let shown = interp.string_value_text(table)?;
        return Err(Raised::no_supplier(&shown).into());
    }
    let walked = crate::dispatch::collection::supplier::drain_supplier(interp, supplier)?;
    let mut pairs = Vec::new();
    for (index, item) in walked {
        let name = interp.string_value_text(index)?.into_boxed_slice();
        pairs.push((name, item));
    }
    Ok(pairs)
}

/// Whether `value` is a `Supplier`, an instance of a subclass included,
/// which the walk reads directly.
fn is_supplier(interp: &mut Interp, value: ObjRef) -> bool {
    let class = interp.class_of_value(value);
    let wanted = interp.classes().lookup("Supplier");
    matches!((class, wanted), (Some(class), Some(wanted)) if interp.classes().is_a(class, wanted))
}
