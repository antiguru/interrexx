//! `Supplier`'s primitive methods, and the constructor every collection's
//! `supplier` method builds one with.

use super::super::array::collection_store;
use super::{
    Arity, ArrayArgument, Cleared, Decoded, Failure, Interp, NativeMethod, ObjRef, Raised,
    array_argument_object, array_slots, is_queue, new_instance, pool_variable,
};

/// The `Supplier` state's three names, bound in the receiver's own pool under
/// the `Supplier` class as scope.
const SUPPLIER_ITEMS: &[u8] = b"ITEMS";

const SUPPLIER_INDEXES: &[u8] = b"INDEXES";

const SUPPLIER_POSITION: &[u8] = b"POSITION";

/// The pair of arrays `allItems` or `allIndexes` collected, held by the
/// library's Rexx code through [`collected`]; `.nil` or absent before the
/// first call and after every `init` (r[class.supplier.init],
/// r[class.supplier.deviation.138]). A copy carries it, so the two answer the
/// same arrays (r[class.supplier.copy]).
const SUPPLIER_COLLECTED: &[u8] = b"COLLECTED";

// ---- Supplier ----

/// A `Supplier` over `items` and `indexes`, positioned at the first pair.
pub(in crate::dispatch) fn new_supplier(
    interp: &mut Interp,
    items: ObjRef,
    indexes: ObjRef,
) -> Result<ObjRef, Failure> {
    let class = interp
        .classes()
        .lookup("Supplier")
        .expect("Supplier is a native class");
    let object = new_instance(interp, class)?;
    interp.roots.activity_mut().push_temp(object);
    store_supplier(interp, object, items, indexes, 0);
    Ok(object)
}

fn store_supplier(
    interp: &mut Interp,
    receiver: ObjRef,
    items: ObjRef,
    indexes: ObjRef,
    position: usize,
) {
    let scope = interp
        .classes()
        .lookup("Supplier")
        .expect("Supplier is a native class");
    let position = interp.counted(position);
    interp.set_pool_variable(receiver, scope, SUPPLIER_ITEMS, items);
    interp.set_pool_variable(receiver, scope, SUPPLIER_INDEXES, indexes);
    interp.set_pool_variable(receiver, scope, SUPPLIER_POSITION, position);
}

/// The supplier's two arrays and its 0-based position, or the refusal a
/// receiver that is not one gets.
fn supplier_state(
    interp: &mut Interp,
    receiver: ObjRef,
) -> Result<(ObjRef, ObjRef, usize), Failure> {
    let scope = interp
        .classes()
        .lookup("Supplier")
        .expect("Supplier is a native class");
    let (Some(items), Some(indexes), Some(position)) = (
        pool_variable(interp, receiver, scope, SUPPLIER_ITEMS),
        pool_variable(interp, receiver, scope, SUPPLIER_INDEXES),
        pool_variable(interp, receiver, scope, SUPPLIER_POSITION),
    ) else {
        return Err(crate::Raised::receiver_kind("a value that is not a supplier").into());
    };
    let Decoded::SmallInt(position) = position.decode() else {
        unreachable!("the position this crate stored is a small integer")
    };
    Ok((items, indexes, position as usize))
}

/// The pairs `supplier` has left, index then item with an empty slot read as
/// `.nil`, leaving it at its end: `createMethodDictionary`'s walk, which
/// calls `SupplierClass`'s own `available`, `item`, `index` and `next` rather
/// than sending them.
pub(in crate::dispatch) fn drain_supplier(
    interp: &mut Interp,
    supplier: ObjRef,
) -> Result<Vec<(ObjRef, ObjRef)>, Failure> {
    let (items, indexes, position) = supplier_state(interp, supplier)?;
    let item_slots = array_slots(interp, items)?.to_vec();
    let index_slots = array_slots(interp, indexes)?;
    let pairs = (position..item_slots.len())
        .map(|at| {
            let index = index_slots.get(at).copied().flatten();
            let item = item_slots[at];
            (index.unwrap_or(ObjRef::NIL), item.unwrap_or(ObjRef::NIL))
        })
        .collect();
    store_supplier(interp, supplier, items, indexes, item_slots.len());
    Ok(pairs)
}

/// How many pairs the supplier still has, measured from the current position
/// against its ITEMS array.
fn supplier_remaining(interp: &mut Interp, receiver: ObjRef) -> Result<usize, Failure> {
    let (items, _, position) = supplier_state(interp, receiver)?;
    let items = array_slots(interp, items)?.len();
    Ok(items.saturating_sub(position))
}

/// `Supplier~available`: whether a pair is still there.
fn native_supplier_available(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let remaining = supplier_remaining(interp, receiver)?;
    Ok(Some(crate::eval::logical(remaining > 0)))
}

/// `Supplier~item` and `Supplier~index`: the current pair's halves.
fn supplier_half(
    interp: &mut Interp,
    receiver: ObjRef,
    take_items: bool,
) -> Result<Option<ObjRef>, Failure> {
    if supplier_remaining(interp, receiver)? == 0 {
        return Err(Raised::no_more_supplier_items().into());
    }
    let (items, indexes, position) = supplier_state(interp, receiver)?;
    let side = if take_items { items } else { indexes };
    Ok(Some(match array_slots(interp, side)?.get(position) {
        Some(Some(value)) => *value,
        Some(None) | None => ObjRef::NIL,
    }))
}

fn native_supplier_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    supplier_half(interp, receiver, true)
}

fn native_supplier_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    supplier_half(interp, receiver, false)
}

/// `Supplier~next`: steps to the next pair, raising 93.937 past the end.
fn native_supplier_next(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if supplier_remaining(interp, receiver)? == 0 {
        return Err(Raised::no_more_supplier_items().into());
    }
    let (items, indexes, position) = supplier_state(interp, receiver)?;
    store_supplier(interp, receiver, items, indexes, position + 1);
    Ok(None)
}

/// One of `init`'s arguments as the array the supplier holds. An answer of
/// `request` with `ARRAY` that is an instance of an `Array` subclass is kept
/// as it is, which [`array_argument_object`] would refuse.
fn supplier_array(interp: &mut Interp, value: ObjRef) -> Result<ObjRef, Failure> {
    if interp.array_body(value).is_some() {
        return array_argument_object(interp, value, ArrayArgument::Positional);
    }
    if let Some(answer) = interp.request_array_value(value)? {
        interp.roots.activity_mut().push_temp(answer);
        let store = collection_store(interp, answer);
        // A `Queue` or `CircularQueue` keeps its items in an array store too,
        // and is refused all the same.
        if !is_queue(interp, answer)
            && let Some((_, dimensions)) = interp.array_body(store)
            && dimensions.is_none_or(|held| held.len() == 1)
        {
            return Ok(answer);
        }
    }
    let found = interp.string_value_text(value)?;
    Err(Raised::object_not_single_dimensional(&found).into())
}

/// `Supplier~init(items, indexes)`.
fn native_supplier_init(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let mut sides = Vec::with_capacity(2);
    for position in [1, 2] {
        let Some(value) = args.get(position - 1).copied().flatten() else {
            return Err(Raised::missing_method_argument(position).into());
        };
        // A `List` or a `Queue` is converted and the CONVERTED array is what
        // the supplier keeps -- measured, `.Supplier~new(.List~of('a'),
        // .List~of('h'))` answers `1 a h`. An argument that already is an
        // array is kept as it stands, object identity and all.
        sides.push(supplier_array(interp, value)?);
    }
    store_supplier(interp, receiver, sides[0], sides[1], 0);
    let scope = interp
        .classes()
        .lookup("Supplier")
        .expect("Supplier is a native class");
    interp.set_pool_variable(receiver, scope, SUPPLIER_COLLECTED, ObjRef::NIL);
    Ok(None)
}

/// `interrexx_supplier_collected([pair])`: the library's cache of what
/// `allItems` and `allIndexes` collected (r[class.supplier.allitems]). With
/// no argument it answers the pair stored last, or `.nil` when there is none;
/// with one it stores it, and `.nil` drops what was stored. It reads no other
/// state, so a `StreamSupplier`, whose entries are not in the arrays of this
/// file, keeps its pair the same way.
pub(in crate::dispatch) fn collected(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let scope = interp
        .classes()
        .lookup("Supplier")
        .expect("Supplier is a native class");
    match args.first().copied().flatten() {
        Some(pair) => {
            interp.set_pool_variable(receiver, scope, SUPPLIER_COLLECTED, pair);
            Ok(None)
        }
        None => Ok(Some(
            pool_variable(interp, receiver, scope, SUPPLIER_COLLECTED).unwrap_or(ObjRef::NIL),
        )),
    }
}

/// `Supplier`'s primitive methods, chained into `ObjectModel::build` beside
/// [`super::super::NATIVE_METHODS`].
pub(in crate::dispatch) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    (
        "Supplier",
        "AVAILABLE",
        Arity::Fixed(0),
        native_supplier_available,
    ),
    ("Supplier", "INIT", Arity::Fixed(2), native_supplier_init),
    ("Supplier", "INDEX", Arity::Fixed(0), native_supplier_index),
    ("Supplier", "ITEM", Arity::Fixed(0), native_supplier_item),
    ("Supplier", "NEXT", Arity::Fixed(0), native_supplier_next),
];
