//! Array slot and subscript arithmetic, and `Array`'s primitive methods.

use super::{
    BehaviourId, Body, Cleared, Decoded, Failure, INIT, Interp, ObjRef, Raised, class_receiver,
    collection, required_string_argument,
};
use interrexx_cast::CastFrom;

use super::{Arity, NativeMethod};
use interrexx_core::ArraySlots;

// `Array`'s sort family, chained into `ObjectModel::build`.
pub(super) mod sort;

// `Array`'s collection surface, chained into `ObjectModel::build` after its
// sort family.
pub(super) mod surface;

/// An array receiver's own slots, borrowed, or the refusal for a receiver that
/// is not one.
#[inline]
pub(super) fn array_slots(interp: &Interp, receiver: ObjRef) -> Result<&[Option<ObjRef>], Failure> {
    // An array itself, which is what nearly every caller holds, in one lookup
    // rather than [`collection_store`]'s check and then another.
    if let Some(slots) = interp.array_slots(receiver) {
        return Ok(slots);
    }
    stored_array_slots(interp, receiver)
}

/// [`array_slots`] for a receiver that is not an array itself.
#[cold]
#[inline(never)]
fn stored_array_slots(interp: &Interp, receiver: ObjRef) -> Result<&[Option<ObjRef>], Failure> {
    let receiver = collection_store(interp, receiver);
    interp
        .array_slots(receiver)
        .ok_or_else(|| crate::Raised::receiver_kind("a value that is not an array").into())
}

/// The array that actually holds `receiver`'s slots: the receiver itself when
/// it carries a `Body::Array`, and otherwise the store its own pool holds.
pub(super) fn collection_store(interp: &Interp, receiver: ObjRef) -> ObjRef {
    if interp.array_slots(receiver).is_some() {
        return receiver;
    }
    let Some(model) = interp.object_model.as_ref() else {
        return receiver;
    };
    let scope = model.array;
    match interp.heap.get(receiver).map(|object| &object.body) {
        Some(Body::Instance { pools, .. }) => pools.get(scope, b"ITEMS").unwrap_or(receiver),
        _ => receiver,
    }
}

/// [`array_slots`] as an owned copy, for a caller that renders the slots and
/// so needs `interp` back.
pub(super) fn array_slots_owned(
    interp: &Interp,
    receiver: ObjRef,
) -> Result<Vec<Option<ObjRef>>, Failure> {
    let slots = array_slots(interp, receiver)?;
    #[cfg(test)]
    note_slots_copied(slots.len());
    Ok(slots.to_vec())
}

#[cfg(test)]
thread_local! {
    /// How many slots [`array_slots_owned`] and `collection::occupied` have
    /// copied on this thread.
    static SLOTS_COPIED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Adds `count` to [`SLOTS_COPIED`].
#[cfg(test)]
pub(super) fn note_slots_copied(count: usize) {
    SLOTS_COPIED.with(|copied| copied.set(copied.get() + count));
}

/// [`SLOTS_COPIED`]'s count.
#[cfg(test)]
pub(crate) fn slots_copied() -> usize {
    SLOTS_COPIED.with(std::cell::Cell::get)
}

/// An array receiver's dimensions array as an owned copy, or the refusal
/// [`array_slots`] gives for a receiver that is not an array.
pub(super) fn array_dimensions(
    interp: &Interp,
    receiver: ObjRef,
) -> Result<Option<Vec<usize>>, Failure> {
    let receiver = collection_store(interp, receiver);
    match interp.array_body(receiver) {
        Some((_, dimensions)) => Ok(dimensions.map(<[usize]>::to_vec)),
        None => Err(crate::Raised::receiver_kind("a value that is not an array").into()),
    }
}

/// An array that can no longer take a shape from a subscript list.
fn array_is_fixed_dimension(interp: &Interp, receiver: ObjRef) -> Result<bool, Failure> {
    let receiver = collection_store(interp, receiver);
    match interp.array_body(receiver) {
        Some((slots, dimensions)) => Ok(dimensions.is_some() || !slots.is_empty()),
        None => Err(crate::Raised::receiver_kind("a value that is not an array").into()),
    }
}

/// The largest size an array may have. It is the same on every target: a size
/// at or below it that the target cannot address is refused as storage the
/// allocator cannot give ([`fixed_array_size`]), not as a size past this.
pub(super) const MAX_FIXED_ARRAY_SIZE: u64 = 100_000_000_000_000_000;

/// The product of `size` and `extent`, or 93.959 past [`MAX_FIXED_ARRAY_SIZE`].
/// Both are the language's numbers, not yet bounded by what the target
/// addresses, so that a product past the limit is told from one the target
/// cannot hold ([`fixed_array_size`]).
fn grown_array_size(size: u64, extent: u64) -> Result<u64, Failure> {
    if extent > MAX_FIXED_ARRAY_SIZE {
        return Err(Raised::array_too_big(MAX_FIXED_ARRAY_SIZE).into());
    }
    size.checked_mul(extent)
        .filter(|size| *size <= MAX_FIXED_ARRAY_SIZE)
        .ok_or_else(|| Failure::from(Raised::array_too_big(MAX_FIXED_ARRAY_SIZE)))
}

/// An index of at least 1 as a slot position. On a 64-bit target it is the
/// index itself. On a narrower one, an index the target cannot address is held
/// at `isize::MAX` when it is within [`MAX_FIXED_ARRAY_SIZE`], past every
/// array's end and refused as storage by a growth, and is `usize::MAX` past
/// it, which no index within the limit there becomes and which
/// [`past_fixed_array_size`] reads as past the limit.
fn index_position(index: u64) -> usize {
    match usize::try_from(index) {
        Ok(index) if index <= isize::MAX.unsigned_abs() => index,
        _ if index > MAX_FIXED_ARRAY_SIZE => usize::MAX,
        _ => isize::MAX.unsigned_abs(),
    }
}

/// Whether a position [`index_position`] answered is past
/// [`MAX_FIXED_ARRAY_SIZE`].
fn past_fixed_array_size(position: usize) -> bool {
    u64::cast_from(position) > MAX_FIXED_ARRAY_SIZE || position == usize::MAX
}

/// A size [`grown_array_size`] answered, as a slot count, or the 5.0 that an
/// allocator refusing it raises where the target cannot address that many.
fn fixed_array_size(size: u64) -> Result<usize, Failure> {
    usize::try_from(size).map_err(|_| Failure::from(Raised::system_resources()))
}

/// `size` empty slots, or the 5.0 raised when the heap limit or the allocator
/// refuses -- [`Raised::system_resources`] carries why that refusal is asked of
/// the allocator rather than of a size limit.
fn empty_slots(interp: &Interp, size: usize) -> Result<Vec<Option<ObjRef>>, Failure> {
    interp.admit_bytes(size.saturating_mul(interrexx_core::SLOT_BYTES))?;
    let mut slots = Vec::new();
    slots
        .try_reserve_exact(size)
        .map_err(|_| Failure::from(Raised::system_resources()))?;
    slots.resize(size, None);
    Ok(slots)
}

/// [`empty_slots`] as an array body's slots.
fn empty_array_slots(interp: &Interp, size: usize) -> Result<ArraySlots, Failure> {
    interp.admit_bytes(size.saturating_mul(interrexx_core::SLOT_BYTES))?;
    ArraySlots::empty(size).map_err(|_| Failure::from(Raised::system_resources()))
}

/// The bounds policy a subscript list is validated under -- `IndexAccess` and
/// `IndexUpdate`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum IndexUse {
    /// `Array~at`: a subscript past the end answers `.nil`.
    Get,
    /// `putRexx`: a subscript past the end grows the array.
    Put,
}

impl IndexUse {
    /// The argument list position the subscript list starts at, which every
    /// refusal a subscript raises counts from: `put`'s first argument is the
    /// value, so its list starts one later than `at`'s.
    pub(super) fn arg_position(self) -> usize {
        match self {
            IndexUse::Get => 1,
            IndexUse::Put => 2,
        }
    }
}

/// The flattened 1-based slot `args` names in `receiver`, or `None` for a
/// subscript out of bounds under [`IndexUse::Get`].
pub(super) fn array_position(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    index_use: IndexUse,
) -> Result<Option<usize>, Failure> {
    let spread;
    let subscripts = match args {
        [Some(only)] => match index_list(interp, *only, index_use.arg_position())? {
            Some(list) => {
                spread = list;
                &spread[..]
            }
            None => args,
        },
        _ => args,
    };
    match array_dimensions(interp, receiver)? {
        Some(dimensions) if dimensions.len() != 1 => {
            multi_dimension_position(interp, receiver, subscripts, index_use, &dimensions)
        }
        _ => single_dimension_position(interp, receiver, subscripts, index_use),
    }
}

/// A lone `Array` argument read as the subscript list it stands for, or `None`
/// for any other argument. The list is every slot of a single-dimensional
/// array, so a hole in it is 98.975 naming the hole, and an array of more
/// dimensions is 93.939 naming `position`.
pub(super) fn index_list(
    interp: &Interp,
    value: ObjRef,
    position: usize,
) -> Result<Option<Vec<Option<ObjRef>>>, Failure> {
    let Some((slots, dimensions)) = interp.array_body(value) else {
        return Ok(None);
    };
    if dimensions.is_some_and(|dimensions| dimensions.len() != 1) {
        return Err(Raised::index_list_not_single_dimensional(position).into());
    }
    if let Some(hole) = slots.iter().position(Option::is_none) {
        return Err(Raised::missing_array_element(hole + 1).into());
    }
    Ok(Some(slots.to_vec()))
}

fn single_dimension_position(
    interp: &mut Interp,
    receiver: ObjRef,
    subscripts: &[Option<ObjRef>],
    index_use: IndexUse,
) -> Result<Option<usize>, Failure> {
    match subscripts {
        [] => Err(Raised::not_enough_method_arguments(index_use.arg_position()).into()),
        [Some(only)] => {
            let position = positive_index(interp, *only, index_use.arg_position())?;
            if position <= array_slots(interp, receiver)?.len() {
                return Ok(Some(position));
            }
            match index_use {
                IndexUse::Get => Ok(None),
                IndexUse::Put if past_fixed_array_size(position) => {
                    Err(Raised::array_too_big(MAX_FIXED_ARRAY_SIZE).into())
                }
                IndexUse::Put => {
                    array_resize(interp, receiver, position)?;
                    Ok(Some(position))
                }
            }
        }
        // [`index_list`] refuses a list with a hole, and an argument list of
        // its own drops a trailing omission, so `~at(,)` arrives as no
        // argument at all and is 93.901 above.
        [None] => Err(Raised::missing_array_element(1).into()),
        _ => {
            if array_is_fixed_dimension(interp, receiver)? {
                return Err(Raised::too_many_subscripts(1).into());
            }
            match index_use {
                IndexUse::Get => Ok(None),
                IndexUse::Put => {
                    let dimensions = array_extend_multi(interp, receiver, subscripts)?;
                    multi_dimension_position(interp, receiver, subscripts, index_use, &dimensions)
                }
            }
        }
    }
}

/// The offset takes the **first** subscript as the fastest-moving one.
fn multi_dimension_position(
    interp: &mut Interp,
    receiver: ObjRef,
    subscripts: &[Option<ObjRef>],
    index_use: IndexUse,
    dimensions: &[usize],
) -> Result<Option<usize>, Failure> {
    if subscripts.len() < dimensions.len() {
        return Err(Raised::not_enough_subscripts(dimensions.len()).into());
    }
    if subscripts.len() > dimensions.len() {
        return Err(Raised::too_many_subscripts(dimensions.len()).into());
    }
    let mut multiplier = 1;
    let mut offset = 0;
    for (at, (subscript, dimension)) in subscripts.iter().zip(dimensions).enumerate() {
        let position = position_index(interp, *subscript, index_use.arg_position() + at + 1)?;
        if position > u64::cast_from(*dimension) {
            return match index_use {
                IndexUse::Get => Ok(None),
                IndexUse::Put => {
                    let grown = array_extend_multi(interp, receiver, subscripts)?;
                    multi_dimension_position(interp, receiver, subscripts, index_use, &grown)
                }
            };
        }
        let position = usize::try_from(position).expect("within a dimension");
        offset += multiplier * (position - 1);
        multiplier *= dimension;
    }
    Ok(Some(offset + 1))
}

/// A subscript converted at the argument precision rather than under the
/// activation's own `NUMERIC DIGITS`, or `None` for one that is not a whole
/// number of at least 1.
pub(super) fn whole_index(interp: &mut Interp, value: ObjRef) -> Result<Option<usize>, Failure> {
    if let Some(index) = small_index(value) {
        return Ok(Some(index));
    }
    Ok(whole_subscript(interp, value)?.map(index_position))
}

/// [`whole_index`]'s whole number as the language reads it, before the
/// target's bound: what a growth checks against [`MAX_FIXED_ARRAY_SIZE`].
fn whole_subscript(interp: &mut Interp, value: ObjRef) -> Result<Option<u64>, Failure> {
    if let Some(index) = small_index(value) {
        return Ok(Some(u64::cast_from(index)));
    }
    Ok(index_number(interp, value)?
        .and_then(|number| number.whole_value(interrexx_num::ARGUMENT_DIGITS))
        .filter(|whole| *whole > 0)
        .map(i64::unsigned_abs))
}

/// A tagged integer that already is the answer, being narrow enough that the
/// rounding rule would change nothing: the shortcut `builtin::whole_number`
/// takes against the same width.
#[inline(always)]
fn small_index(value: ObjRef) -> Option<usize> {
    if let Decoded::SmallInt(small) = value.decode()
        && let Some(whole) = interrexx_num::whole_i64(small, interrexx_num::ARGUMENT_DIGITS)
        && let Ok(index) = usize::try_from(whole)
        && index > 0
    {
        return Some(index);
    }
    None
}

/// An index's number: the value itself where it is one, else its requested
/// string's. No string value fails the index's check (r[conv.string.argument-kind]).
fn index_number(
    interp: &mut Interp,
    value: ObjRef,
) -> Result<Option<interrexx_num::Number>, Failure> {
    match interp.to_number(value) {
        Ok(number) => Ok(Some(number)),
        Err(_) => interp.requested_argument_number(value),
    }
}

/// [`whole_index`]'s conversion admitting zero, for the one index in the
/// language that counts from it: a list handle is read under the same argument
/// precision, and a list's first handle is `0`.
pub(super) fn unsigned_index(interp: &mut Interp, value: ObjRef) -> Result<Option<usize>, Failure> {
    if let Decoded::SmallInt(small) = value.decode()
        && let Some(whole) = interrexx_num::whole_i64(small, interrexx_num::ARGUMENT_DIGITS)
    {
        return Ok(usize::try_from(whole).ok());
    }
    Ok(index_number(interp, value)?
        .and_then(|number| number.whole_value(interrexx_num::ARGUMENT_DIGITS))
        .and_then(|whole| usize::try_from(whole).ok()))
}

/// A comparison result as a number is read -- the conversion both sort
/// comparators make on what the Rexx method answered, at the default precision
/// rather than at the subscript precision.
fn whole_comparison(interp: &mut Interp, value: ObjRef) -> Option<i64> {
    let digits = crate::DEFAULT_DIGITS;
    if let Decoded::SmallInt(small) = value.decode() {
        return interrexx_num::whole_i64(small, digits);
    }
    interp.to_number(value).ok()?.whole_value(digits)
}

/// One subscript as a required positive whole number, `position` naming its
/// place in the method's own argument list.
pub(super) fn positive_index(
    interp: &mut Interp,
    value: ObjRef,
    position: usize,
) -> Result<usize, Failure> {
    if let Some(index) = small_index(value) {
        return Ok(index);
    }
    match whole_index(interp, value)? {
        Some(index) => Ok(index),
        None => {
            let found = interp.string_value_text(value)?;
            Err(Raised::method_argument_not_positive(position, &found).into())
        }
    }
}

/// One subscript of a multidimensional index as a position argument is read --
/// the same conversion [`positive_index`] makes, under a different pair of
/// errors.
fn position_index(
    interp: &mut Interp,
    subscript: Option<ObjRef>,
    position: usize,
) -> Result<u64, Failure> {
    let Some(value) = subscript else {
        return Err(Raised::missing_method_argument(position).into());
    };
    match whole_subscript(interp, value)? {
        Some(index) => Ok(index),
        None => {
            let found = interp.string_value_text(value)?;
            Err(Raised::invalid_position(&found).into())
        }
    }
}

/// Grow the receiver to `size` slots, the added ones empty.
fn array_resize(interp: &mut Interp, receiver: ObjRef, size: usize) -> Result<(), Failure> {
    let receiver = collection_store(interp, receiver);
    interp.admit_bytes(size.saturating_mul(interrexx_core::SLOT_BYTES))?;
    let grown = match interp.heap.get_mut(receiver).map(|object| &mut object.body) {
        Some(Body::Array { slots, .. }) => {
            let before = slots.capacity();
            if let Some(extra) = size.checked_sub(slots.len()) {
                slots
                    .try_reserve_exact(extra)
                    .map_err(|_| Failure::from(Raised::system_resources()))?;
            }
            slots.resize(size);
            slots.capacity().saturating_sub(before)
        }
        _ => return Err(crate::Raised::receiver_kind("a value that is not an array").into()),
    };
    interp.charge_growth(grown * interrexx_core::SLOT_BYTES);
    Ok(())
}

/// Grow the receiver so that every subscript is within bounds, answering the
/// shape it now has.
fn array_extend_multi(
    interp: &mut Interp,
    receiver: ObjRef,
    subscripts: &[Option<ObjRef>],
) -> Result<Vec<usize>, Failure> {
    let held = array_dimensions(interp, receiver)?;
    let old = held.filter(|dimensions| dimensions.len() == subscripts.len());
    let mut dimensions = Vec::with_capacity(subscripts.len());
    let mut size = 1u64;
    for (at, subscript) in subscripts.iter().enumerate() {
        let position = position_index(interp, *subscript, at + 1)?;
        let dimension = match &old {
            Some(old) => position.max(u64::cast_from(old[at])),
            None => position,
        };
        size = grown_array_size(size, dimension)?;
        dimensions.push(dimension);
    }
    // Each dimension is at least 1, so none exceeds the product, and a target
    // that addresses the product addresses each.
    let size = fixed_array_size(size)?;
    let dimensions = dimensions
        .into_iter()
        .map(fixed_array_size)
        .collect::<Result<Vec<usize>, Failure>>()?;
    array_reshape(interp, receiver, &dimensions, size, old.as_deref())?;
    Ok(dimensions)
}

/// The element move array growth performs: each filled slot goes to the offset
/// its multidimensional index has under `dimensions`.
fn array_reshape(
    interp: &mut Interp,
    receiver: ObjRef,
    dimensions: &[usize],
    size: usize,
    old: Option<&[usize]>,
) -> Result<(), Failure> {
    let slots = array_slots_owned(interp, receiver)?;
    debug_assert!(
        old.is_some() || slots.iter().all(Option::is_none),
        "a reshape with no source shape must have nothing to move"
    );
    let mut grown = empty_slots(interp, size)?;
    if let Some(old) = old {
        for (position, item) in slots.iter().enumerate() {
            if item.is_none() {
                continue;
            }
            let mut rest = position;
            let mut offset = 0;
            let mut multiplier = 1;
            for (extent, dimension) in old.iter().zip(dimensions) {
                offset += multiplier * (rest % extent);
                rest /= extent;
                multiplier *= dimension;
            }
            grown[offset] = *item;
        }
    }
    let (before, after) = match interp.heap.get_mut(receiver).map(|object| &mut object.body) {
        Some(Body::Array {
            slots,
            dimensions: held,
        }) => {
            let before = slots.capacity();
            slots.replace(grown);
            *held = Some(dimensions.into());
            (before, slots.capacity())
        }
        _ => return Err(crate::Raised::receiver_kind("a value that is not an array").into()),
    };
    // The replacement can hold fewer slots than the extended original did.
    interp.charge_growth(after.saturating_sub(before) * interrexx_core::SLOT_BYTES);
    interp
        .heap
        .release_body_bytes(before.saturating_sub(after) * interrexx_core::SLOT_BYTES);
    Ok(())
}

/// `Array~at(index)` and `Array~[index]`: the item at `index`, or `.nil` for
/// an empty slot and for a subscript past the end.
pub(super) fn native_array_at(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    native_array_at_for(interp, cleared, receiver, args)
}

/// [`native_array_at`] against a named store, which is what `Queue`'s own row
/// needs: its slots live in an `Array` the instance holds rather than in the
/// receiver.
pub(super) fn native_array_at_for(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // The subscript first, so the slots are borrowed rather than copied: the
    // conversion needs `&mut interp` and the read does not, and reading one
    // slot must not cost a copy of the whole array.
    let Some(index) = array_position(interp, receiver, args, IndexUse::Get)? else {
        return Ok(Some(ObjRef::NIL));
    };
    Ok(Some(match array_slots(interp, receiver)?.get(index - 1) {
        Some(Some(item)) => *item,
        Some(None) | None => ObjRef::NIL,
    }))
}

/// `Array~put(value, index...)` and `Array~[index...] = value`: requires the
/// value, validates the rest as the subscript list for an update and answers
/// nothing.
pub(super) fn native_array_put(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if args.len() < 2 {
        return Err(Raised::not_enough_method_arguments(2).into());
    }
    let Some(value) = args[0] else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let position = array_position(interp, receiver, &args[1..], IndexUse::Put)?
        .expect("IndexUse::Put grows the array rather than answering out of bounds");
    let receiver = collection_store(interp, receiver);
    match interp.heap.get_mut(receiver).map(|object| &mut object.body) {
        Some(Body::Array { slots, .. }) => {
            slots.set(position - 1, Some(value));
            Ok(None)
        }
        _ => Err(crate::Raised::receiver_kind("a value that is not an array").into()),
    }
}

/// `Array~dimension([n])`: how many dimensions the array has, or the extent of
/// dimension `n`, which is `0` for a dimension the array does not have.
pub(super) fn native_array_dimension(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let size = array_slots(interp, receiver)?.len();
    let dimensions = array_dimensions(interp, receiver)?;
    let answer = match args.first().copied().flatten() {
        None => match &dimensions {
            Some(dimensions) => dimensions.len(),
            None if size == 0 => 0,
            None => 1,
        },
        Some(target) => {
            let position = positive_index(interp, target, 1)?;
            match &dimensions {
                Some(dimensions) if dimensions.len() != 1 => {
                    dimensions.get(position - 1).copied().unwrap_or(0)
                }
                _ if position == 1 => size,
                _ => 0,
            }
        }
    };
    Ok(Some(interp.counted(answer)))
}

/// `.Array~new([size])` and `.Array~new(dimension...)`.
pub(super) fn native_array_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<super::NativeStarted, Failure> {
    let class = class_receiver(interp, receiver)?;
    let spread;
    let body = match args {
        [] => Body::array(Vec::new()),
        [only] => match only.and_then(|value| interp.array_slots_of(value)) {
            Some(slots) => {
                let items = slots.iter().flatten().count();
                spread = slots[..items].to_vec();
                multidimensional_body(interp, &spread)?
            }
            None => {
                let size = fixed_array_size(array_size_argument(interp, *only, 1)?)?;
                Body::Array {
                    slots: empty_array_slots(interp, size)?,
                    // `newRexx`'s own `if (totalSize == 0)`, whose one entry
                    // nothing reads: an explicit zero size fixes the shape,
                    // and the entry is not the extent.
                    dimensions: (size == 0).then(|| Box::from([0].as_slice())),
                }
            }
        },
        _ => multidimensional_body(interp, args)?,
    };
    let object = array_of_class(interp, class, body)?;
    super::begin_init(interp, object, &[])
}

/// `body` as an object of `class`: a bare `Body::Array` for `.Array` itself,
/// and an instance carrying it as a store for any subclass.
fn array_of_class(interp: &mut Interp, class: ObjRef, body: Body) -> Result<ObjRef, Failure> {
    let store = interp.alloc_charged(BehaviourId::ARRAY, body);
    if class == interp.object_model().array {
        interp.roots.activity_mut().push_temp(store);
        return Ok(store);
    }
    collection::instance_over_store(interp, class, store)
}

/// `.Array~of(item, ...)`: the arguments as an array's slots, in order. Its
/// `INIT` send carries no arguments.
pub(super) fn native_array_of(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let body = Body::Array {
        slots: args.to_vec().into(),
        dimensions: args.is_empty().then(|| Box::from([0].as_slice())),
    };
    let object = array_of_class(interp, class, body)?;
    let caller = interp.caller();
    interp.send_message(object, INIT, None, &[], caller)?;
    Ok(Some(object))
}

/// A body whose shape is `dimensions` and whose slots are all empty.
fn multidimensional_body(
    interp: &mut Interp,
    dimensions: &[Option<ObjRef>],
) -> Result<Body, Failure> {
    let mut shape = Vec::with_capacity(dimensions.len());
    let mut size = 1u64;
    for (at, dimension) in dimensions.iter().enumerate() {
        let extent = array_size_argument(interp, *dimension, at + 1)?;
        size = grown_array_size(size, extent)?;
        shape.push(extent);
    }
    let slots = empty_array_slots(interp, fixed_array_size(size)?)?;
    // A zero extent leaves the product zero, so an extent can still be one the
    // target does not address.
    let shape = shape
        .into_iter()
        .map(fixed_array_size)
        .collect::<Result<Box<[usize]>, Failure>>()?;
    Ok(Body::Array {
        slots,
        dimensions: Some(shape),
    })
}

/// A size check: a whole number of at least zero, with the fixed-array maximum
/// as its upper bound, before the target's bound ([`fixed_array_size`]).
fn array_size_argument(
    interp: &mut Interp,
    argument: Option<ObjRef>,
    position: usize,
) -> Result<u64, Failure> {
    let Some(value) = argument else {
        return Err(Raised::missing_method_argument(position).into());
    };
    let size = index_number(interp, value)?
        .and_then(|number| number.whole_value(interrexx_num::ARGUMENT_DIGITS))
        .and_then(|whole| u64::try_from(whole).ok());
    match size {
        Some(size) if size <= MAX_FIXED_ARRAY_SIZE => Ok(size),
        Some(_) => Err(Raised::array_too_big(MAX_FIXED_ARRAY_SIZE).into()),
        None => {
            let found = interp.string_value_text(value)?;
            Err(Raised::argument_not_non_negative(position, &found).into())
        }
    }
}

/// `Array~size`: how many slots the array has, empty ones included.
pub(super) fn native_array_size(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let size = array_slots(interp, receiver)?.len();
    Ok(Some(interp.counted(size)))
}

/// `Array~items`: how many slots hold an object.
pub(super) fn native_array_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let items = array_slots(interp, receiver)?.iter().flatten().count();
    Ok(Some(interp.counted(items)))
}

/// `Array~makeString(format, separator)` and `Array~toString(format,
/// separator)`: the array's items as one string. `MakeString` and `ToString`
/// both name one function at the same arity, which is why one function answers
/// both rows.
pub(super) fn native_array_make_string(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // The option argument: its first character, upcased, with the whole
    // argument omitted meaning `L`.
    let form = match args.first().copied().flatten() {
        None => b'L',
        Some(argument) => {
            let argument = required_string_argument(interp, argument, 1)?;
            let text = interp.to_text(argument).to_vec();
            match text.first().copied().map(|byte| byte.to_ascii_uppercase()) {
                Some(byte @ (b'L' | b'C')) => byte,
                _ => return Err(Raised::method_option_not_recognised("CL", &text).into()),
            }
        }
    };
    let separator = args.get(1).copied().flatten();
    if form == b'C' && separator.is_some() {
        return Err(Raised::too_many_method_arguments(1).into());
    }
    let separator = match separator {
        None if form == b'L' => b"\n".to_vec(),
        None => Vec::new(),
        Some(argument) => {
            let argument = required_string_argument(interp, argument, 2)?;
            interp.to_text(argument).to_vec()
        }
    };
    let slots = array_slots_owned(interp, receiver)?;
    // The join itself is `Interp::array_string` and not a second loop here:
    // `makeString` with no arguments is what a string context asks an array
    // for, so the two must not be able to disagree about an empty slot or
    // about a nested array's rendering.
    let out = interp.array_string(&slots, &separator)?;
    Ok(Some(interp.text_built(out)))
}
