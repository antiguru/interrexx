//! The per-code conversions the rows of `TABLE` name, and the helpers they share.

use std::ffi::c_int;

use interrexx_core::ObjRef;

use super::{Class, Conversion, Failure, MAX_WHOLENUMBER, RESULT_DIGITS, Value, pointer_string};

/// `REXX_VALUE_ARGLIST`.
pub(super) fn arglist_to_native(
    cx: &mut Conversion<'_>,
    _argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    let arguments = cx.host.arguments();
    Ok(Value::Object(cx.host.register_local(arguments)))
}

/// `REXX_VALUE_NAME`.
pub(super) fn name_to_native(
    cx: &mut Conversion<'_>,
    _argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    let name = cx.host.message_name();
    Ok(Value::CString(cx.c_string_name(&name)))
}

/// `REXX_VALUE_SCOPE`.
pub(super) fn scope_to_native(
    cx: &mut Conversion<'_>,
    _argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    if !cx.host.is_method() {
        return Err(Failure::Signature);
    }
    let scope = cx.host.scope();
    Ok(Value::Object(cx.host.register_local(scope)))
}

/// `REXX_VALUE_CSELF`.
pub(super) fn cself_to_native(
    cx: &mut Conversion<'_>,
    _argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    if !cx.host.is_method() {
        return Err(Failure::Signature);
    }
    Ok(Value::Pointer(
        cx.host.cself().unwrap_or(std::ptr::null_mut()),
    ))
}

/// `REXX_VALUE_OSELF`.
pub(super) fn oself_to_native(
    cx: &mut Conversion<'_>,
    _argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    if !cx.host.is_method() {
        return Err(Failure::Signature);
    }
    let receiver = cx.host.receiver();
    Ok(Value::Object(cx.host.register_local(receiver)))
}

/// `REXX_VALUE_SUPER`.
pub(super) fn super_to_native(
    cx: &mut Conversion<'_>,
    _argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    if !cx.host.is_method() {
        return Err(Failure::Signature);
    }
    let scope = cx.host.super_scope();
    Ok(Value::Object(cx.host.register_local(scope)))
}

/// `REXX_VALUE_RexxObjectPtr`.
pub(super) fn object_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    Ok(Value::Object(cx.host.register_local(argument)))
}

/// A row converting a signed integer within `min..=max`, answering the value as the C type `T`.
fn signed<T: TryFrom<i64>>(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
    min: i64,
    max: i64,
) -> Result<T, Failure> {
    cx.host
        .signed_integer(argument, min, max)?
        .filter(|number| (min..=max).contains(number))
        .and_then(|number| T::try_from(number).ok())
        .ok_or(Failure::OutOfRange {
            position,
            min: i128::from(min),
            max: i128::from(max),
            argument,
        })
}

/// A row converting an unsigned integer up to `max`, answering the value as the C type `T`.
fn unsigned<T: TryFrom<u64>>(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
    max: u64,
) -> Result<T, Failure> {
    cx.host
        .unsigned_integer(argument, max)?
        .filter(|number| *number <= max)
        .and_then(|number| T::try_from(number).ok())
        .ok_or(Failure::OutOfRange {
            position,
            min: 0,
            max: i128::from(max),
            argument,
        })
}

/// `REXX_VALUE_int`.
pub(super) fn int_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let (min, max) = (i64::from(c_int::MIN), i64::from(c_int::MAX));
    signed(cx, argument, position, min, max).map(Value::Int)
}

/// `REXX_VALUE_int8_t`.
pub(super) fn int8_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let (min, max) = (i64::from(i8::MIN), i64::from(i8::MAX));
    signed(cx, argument, position, min, max).map(Value::Int8)
}

/// `REXX_VALUE_int16_t`.
pub(super) fn int16_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let (min, max) = (i64::from(i16::MIN), i64::from(i16::MAX));
    signed(cx, argument, position, min, max).map(Value::Int16)
}

/// `REXX_VALUE_int32_t`.
pub(super) fn int32_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let (min, max) = (i64::from(i32::MIN), i64::from(i32::MAX));
    signed(cx, argument, position, min, max).map(Value::Int32)
}

/// `REXX_VALUE_int64_t`, which converts at the same twenty digits as the
/// other signed integer rows.
pub(super) fn int64_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    signed(cx, argument, position, i64::MIN, i64::MAX).map(Value::Int64)
}

/// `REXX_VALUE_ssize_t` and `REXX_VALUE_intptr_t`, both the whole range of a pointer-sized signed integer.
pub(super) fn signed_word_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    signed(cx, argument, position, i64::MIN, i64::MAX).map(Value::Isize)
}

/// `REXX_VALUE_wholenumber_t`.
pub(super) fn whole_number_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    signed(cx, argument, position, -MAX_WHOLENUMBER, MAX_WHOLENUMBER).map(Value::Isize)
}

/// `REXX_VALUE_uint8_t`.
pub(super) fn uint8_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    unsigned(cx, argument, position, u64::from(u8::MAX)).map(Value::Uint8)
}

/// `REXX_VALUE_uint16_t`.
pub(super) fn uint16_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    unsigned(cx, argument, position, u64::from(u16::MAX)).map(Value::Uint16)
}

/// `REXX_VALUE_uint32_t`.
pub(super) fn uint32_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    unsigned(cx, argument, position, u64::from(u32::MAX)).map(Value::Uint32)
}

/// `REXX_VALUE_uint64_t`, which converts at the same twenty digits as the
/// other unsigned integer rows.
pub(super) fn uint64_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    unsigned(cx, argument, position, u64::MAX).map(Value::Uint64)
}

/// `REXX_VALUE_size_t` and `REXX_VALUE_uintptr_t`, both the whole range of a pointer-sized unsigned integer.
pub(super) fn unsigned_word_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    unsigned(cx, argument, position, u64::MAX).map(Value::Usize)
}

/// `REXX_VALUE_stringsize_t`.
pub(super) fn string_size_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let max = MAX_WHOLENUMBER.unsigned_abs();
    unsigned(cx, argument, position, max).map(Value::Usize)
}

/// `REXX_VALUE_positive_wholenumber_t`.
pub(super) fn positive_whole_number_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    cx.host
        .signed_integer(argument, 1, MAX_WHOLENUMBER)?
        .filter(|number| (1..=MAX_WHOLENUMBER).contains(number))
        .and_then(|number| isize::try_from(number).ok())
        .map(Value::Isize)
        .ok_or(Failure::NotPositive { position, argument })
}

/// `REXX_VALUE_nonnegative_wholenumber_t`.
pub(super) fn nonnegative_whole_number_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    cx.host
        .signed_integer(argument, 0, MAX_WHOLENUMBER)?
        .filter(|number| (0..=MAX_WHOLENUMBER).contains(number))
        .and_then(|number| isize::try_from(number).ok())
        .map(Value::Isize)
        .ok_or(Failure::NotNonnegative { position, argument })
}

/// `REXX_VALUE_logical_t`.
pub(super) fn logical_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    match cx.host.logical(argument)? {
        Ok(truth) => Ok(Value::Usize(usize::from(truth))),
        Err(found) => Err(Failure::NotLogical { found }),
    }
}

/// `REXX_VALUE_double`.
pub(super) fn double_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    cx.host
        .double_value(argument)?
        .map(Value::Double)
        .ok_or(Failure::InvalidDouble { position, argument })
}

/// `REXX_VALUE_float`.
pub(super) fn float_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the C++'s own `(float)` cast, which is the conversion measured"
    )]
    let narrow = |value: f64| Value::Float(value as f32);
    cx.host
        .double_value(argument)?
        .map(narrow)
        .ok_or(Failure::InvalidDouble { position, argument })
}

/// `REXX_VALUE_CSTRING`.
pub(super) fn cstring_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let string = cx
        .host
        .string_value(argument)?
        .ok_or(Failure::NoStringValue { position })?;
    let bytes = cx
        .host
        .string_bytes(string)
        .ok_or(Failure::NoStringValue { position })?
        .into_owned();
    Ok(Value::CString(cx.c_string_for(string, &bytes)))
}

/// `REXX_VALUE_RexxStringObject`.
pub(super) fn string_object_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let string = cx
        .host
        .string_value(argument)?
        .ok_or(Failure::NoStringValue { position })?;
    // A local reference could be registered only for a string the conversion
    // had to create. Minting a handle is registering it, so here every one is
    // rooted and the "was it created" question does not arise.
    Ok(Value::Object(cx.host.register_local(string)))
}

/// `REXX_VALUE_RexxArrayObject`.
pub(super) fn array_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    let array = cx
        .host
        .array_value(argument)?
        .ok_or(Failure::NotArray { argument })?;
    Ok(Value::Object(cx.host.register_local(array)))
}

/// `REXX_VALUE_RexxStemObject`: a stem, or in a
/// call the name of the caller's.
pub(super) fn stem_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let stem = if cx.host.is_stem(argument) {
        argument
    } else if cx.host.is_method() {
        return Err(Failure::NoStem { position, argument });
    } else {
        cx.host
            .context_stem(argument)?
            .ok_or(Failure::NoStem { position, argument })?
    };
    Ok(Value::Object(cx.host.register_local(stem)))
}

/// An argument that must be an instance of `class`, handed over as it is.
fn instance(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
    class: Class,
) -> Result<Value, Failure> {
    if !cx.host.is_instance_of(argument, class) {
        return Err(Failure::NotInstance { position, class });
    }
    Ok(Value::Object(cx.host.register_local(argument)))
}

/// `REXX_VALUE_RexxClassObject`.
pub(super) fn class_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    instance(cx, argument, position, Class::Class)
}

/// `REXX_VALUE_RexxMutableBufferObject`.
pub(super) fn mutable_buffer_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    instance(cx, argument, position, Class::MutableBuffer)
}

/// `REXX_VALUE_RexxVariableReferenceObject`.
pub(super) fn variable_reference_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    instance(cx, argument, position, Class::VariableReference)
}

/// `REXX_VALUE_POINTER`.
pub(super) fn pointer_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    cx.host
        .pointer_value(argument)
        .map(Value::Pointer)
        .ok_or(Failure::NotInstance {
            position,
            class: Class::Pointer,
        })
}

/// `REXX_VALUE_POINTERSTRING`.
pub(super) fn pointer_string_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let text = cx.host.string_value_text(argument);
    pointer_string(&text)
        .map(|address| Value::Pointer(std::ptr::without_provenance_mut(address)))
        .ok_or(Failure::NotPointerString { position, argument })
}

/// The object an object code becomes.
pub(super) fn object_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Object(handle) = value else {
        return Err(Failure::ResultSignature);
    };
    if handle.is_null() {
        return Ok(None);
    }
    cx.host
        .resolve(handle)
        .map(Some)
        .ok_or(Failure::StaleHandle)
}

/// The object a `REXX_VALUE_POINTER` value becomes,
/// which wraps the address in a `.Pointer` and does not register it: the
/// answer is the call's result, which the caller roots.
pub(super) fn pointer_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Pointer(address) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.new_pointer(address)))
}

/// The object a `REXX_VALUE_POINTERSTRING` value becomes.
pub(super) fn pointer_string_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Pointer(address) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(
        cx.host
            .new_string(&interrexx_core::pointer_to_string(address)),
    ))
}

/// The object a `REXX_VALUE_int` value becomes, built
/// here rather than through the host: a `c_int` is a tagged value, and asking
/// the host for it would add an allocation point the call does not have.
///
/// # Panics
/// Never for a `c_int`, whose whole range is a small integer.
pub(super) fn int_from_native(
    _cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Int(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(
        ObjRef::small_int(i64::from(number)).expect("a c_int is a small integer"),
    ))
}

/// The object a `REXX_VALUE_int8_t` value becomes.
pub(super) fn int8_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Int8(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.whole_number(isize::from(number))))
}

/// The object a `REXX_VALUE_int16_t` value becomes.
pub(super) fn int16_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Int16(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.whole_number(isize::from(number))))
}

/// The object a `REXX_VALUE_int32_t` value becomes.
pub(super) fn int32_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Int32(number) = value else {
        return Err(Failure::ResultSignature);
    };
    let number = isize::try_from(number).map_err(|_| Failure::ResultSignature)?;
    Ok(Some(cx.host.whole_number(number)))
}

/// The object a `REXX_VALUE_int64_t` value becomes: what a pointer-sized
/// value becomes.
pub(super) fn int64_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Int64(number) = value else {
        return Err(Failure::ResultSignature);
    };
    let number = isize::try_from(number).map_err(|_| Failure::ResultSignature)?;
    Ok(Some(cx.host.whole_number(number)))
}

/// The object a pointer-sized signed code becomes.
pub(super) fn isize_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Isize(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.whole_number(number)))
}

/// The object a `REXX_VALUE_uint8_t` value becomes.
pub(super) fn uint8_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Uint8(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.unsigned_number(u64::from(number))))
}

/// The object a `REXX_VALUE_uint16_t` value becomes.
pub(super) fn uint16_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Uint16(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.unsigned_number(u64::from(number))))
}

/// The object a `REXX_VALUE_uint32_t` value becomes.
pub(super) fn uint32_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Uint32(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.unsigned_number(u64::from(number))))
}

/// The object a `REXX_VALUE_uint64_t` value becomes.
pub(super) fn uint64_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Uint64(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.unsigned_number(number)))
}

/// The object a pointer-sized unsigned code becomes.
pub(super) fn usize_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Usize(number) = value else {
        return Err(Failure::ResultSignature);
    };
    let number = u64::try_from(number).map_err(|_| Failure::ResultSignature)?;
    Ok(Some(cx.host.unsigned_number(number)))
}

/// The object a `REXX_VALUE_logical_t` value becomes:
/// any value but zero is true.
pub(super) fn logical_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Usize(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.whole_number(isize::from(number != 0))))
}

/// The object a `REXX_VALUE_double` value becomes.
pub(super) fn double_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Double(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.double_object(number, RESULT_DIGITS)))
}

/// The object a `REXX_VALUE_float` value becomes, which
/// widens to a `double`.
pub(super) fn float_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Float(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(
        cx.host.double_object(f64::from(number), RESULT_DIGITS),
    ))
}

/// The object a `REXX_VALUE_CSTRING` value becomes: no
/// object for a null pointer, and a string of the bytes before the first NUL
/// otherwise.
pub(super) fn cstring_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::CString(pointer) = value else {
        return Err(Failure::ResultSignature);
    };
    if pointer.is_null() {
        return Ok(None);
    }
    let Some(bytes) = cx.strings.bytes_at(pointer) else {
        return Err(Failure::StaleHandle);
    };
    let bytes = bytes.split(|byte| *byte == 0).next().unwrap_or_default();
    Ok(Some(cx.host.new_string(bytes)))
}
