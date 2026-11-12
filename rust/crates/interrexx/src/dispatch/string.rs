//! `String`'s primitive methods.

use super::{
    Arity, Cleared, Failure, Interp, NativeMethod, ObjRef, abbrev_arguments, backward_search,
    backward_search_arguments, bit_arguments, case_shift_arguments, changestr_arguments,
    compare_arguments, compare_to_arguments, conversion_length_argument, copies_argument,
    count_method_argument, datatype_option_argument, delete_arguments, delword_arguments,
    ends_with, equals_argument, forward_search, forward_search_arguments, insert_arguments,
    match_region_arguments, match_region_over, overlay_arguments, pad_arguments, replace_at_bytes,
    replace_at_plan, space_arguments, starts_with, strip_arguments, substr_arguments,
    translate_arguments, verify_arguments, wordpos_arguments,
};
use super::{
    BehaviourId, Body, INIT, refuse_method_argument, usize_or_refuse, whole_method_argument,
};
use crate::builtin::convert;
use crate::builtin::numeric;
use crate::error::Raised;
use interrexx_parse::Operator;

fn native_string_pos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_pos(interp, receiver, args, crate::builtin::string::find_forward)?;
    Ok(Some(interp.counted(found)))
}

fn native_string_caselesspos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_pos(
        interp,
        receiver,
        args,
        crate::builtin::string::caseless_find_forward,
    )?;
    Ok(Some(interp.counted(found)))
}

/// `POS`'s search reported as a boolean.
fn native_string_contains(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_pos(interp, receiver, args, crate::builtin::string::find_forward)?;
    Ok(Some(crate::eval::logical(found > 0)))
}

fn native_string_caselesscontains(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_pos(
        interp,
        receiver,
        args,
        crate::builtin::string::caseless_find_forward,
    )?;
    Ok(Some(crate::eval::logical(found > 0)))
}

/// [`forward_search`] over the receiver's own text.
fn string_pos(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    scan: fn(&[u8], &[u8], usize, usize) -> usize,
) -> Result<usize, Failure> {
    let (needle, start, range) = forward_search_arguments(interp, args)?;
    let found = {
        let bytes = interp.to_text(receiver);
        forward_search(&bytes, &needle, start, range, scan)
    };
    interp.give_result_buffer(needle);
    Ok(found)
}

fn native_string_lastpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_lastpos(
        interp,
        receiver,
        args,
        crate::builtin::string::find_backward,
    )?;
    Ok(Some(interp.counted(found)))
}

fn native_string_caselesslastpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_lastpos(
        interp,
        receiver,
        args,
        crate::builtin::string::caseless_find_backward,
    )?;
    Ok(Some(interp.counted(found)))
}

/// [`backward_search`] over the receiver's own text.
fn string_lastpos(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    scan: fn(&[u8], &[u8], usize, usize) -> usize,
) -> Result<usize, Failure> {
    let (needle, start, range) = backward_search_arguments(interp, args)?;
    let found = {
        let bytes = interp.to_text(receiver);
        backward_search(&bytes, &needle, start, range, scan)
    };
    interp.give_result_buffer(needle);
    Ok(found)
}

fn native_string_countstr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let count = string_countstr(
        interp,
        receiver,
        args,
        crate::builtin::string::count_occurrences,
    )?;
    Ok(Some(interp.counted(count)))
}

fn native_string_caselesscountstr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let count = string_countstr(
        interp,
        receiver,
        args,
        crate::builtin::string::caseless_count_occurrences,
    )?;
    Ok(Some(interp.counted(count)))
}

/// The non-overlapping count of `needle` in the receiver's own text.
fn string_countstr(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    count: fn(&[u8], &[u8], usize) -> usize,
) -> Result<usize, Failure> {
    let needle = super::string_method_argument(interp, args, 0)?;
    let found = {
        let bytes = interp.to_text(receiver);
        count(&bytes, &needle, usize::MAX)
    };
    interp.give_result_buffer(needle);
    Ok(found)
}

fn native_string_wordpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_wordpos(interp, receiver, args, crate::builtin::word::wordpos_bytes)?;
    Ok(Some(interp.counted(found)))
}

fn native_string_caselesswordpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_wordpos(
        interp,
        receiver,
        args,
        crate::builtin::word::caseless_wordpos_bytes,
    )?;
    Ok(Some(interp.counted(found)))
}

/// `WORDPOS`'s search reported as a boolean.
fn native_string_containsword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_wordpos(interp, receiver, args, crate::builtin::word::wordpos_bytes)?;
    Ok(Some(crate::eval::logical(found > 0)))
}

fn native_string_caselesscontainsword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_wordpos(
        interp,
        receiver,
        args,
        crate::builtin::word::caseless_wordpos_bytes,
    )?;
    Ok(Some(crate::eval::logical(found > 0)))
}

/// [`wordpos_arguments`]' search over the receiver's own text.
fn string_wordpos(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    scan: fn(&[u8], &[u8], usize) -> usize,
) -> Result<usize, Failure> {
    let (phrase, start) = wordpos_arguments(interp, args)?;
    let found = {
        let bytes = interp.to_text(receiver);
        scan(&phrase, &bytes, start)
    };
    interp.give_result_buffer(phrase);
    Ok(found)
}

fn native_string_startswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_at_an_end(interp, receiver, args, starts_with, <[u8]>::eq)
}

fn native_string_caselessstartswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_at_an_end(
        interp,
        receiver,
        args,
        starts_with,
        crate::builtin::string::caseless_eq,
    )
}

fn native_string_endswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_at_an_end(interp, receiver, args, ends_with, <[u8]>::eq)
}

fn native_string_caselessendswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_at_an_end(
        interp,
        receiver,
        args,
        ends_with,
        crate::builtin::string::caseless_eq,
    )
}

/// [`starts_with`] or [`ends_with`] over the receiver's own text.
fn string_at_an_end(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    at: fn(&[u8], &[u8], fn(&[u8], &[u8]) -> bool) -> bool,
    matches: fn(&[u8], &[u8]) -> bool,
) -> Result<Option<ObjRef>, Failure> {
    let needle = super::named_string_argument(interp, args, 0, "match")?;
    let answer = {
        let bytes = interp.to_text(receiver);
        at(&bytes, &needle, matches)
    };
    interp.give_result_buffer(needle);
    Ok(Some(crate::eval::logical(answer)))
}

fn native_string_match(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_match(interp, receiver, args, <[u8]>::eq)
}

fn native_string_caselessmatch(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_match(interp, receiver, args, crate::builtin::string::caseless_eq)
}

/// [`match_region_over`] over the receiver's own text.
fn string_match(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    matches: fn(&[u8], &[u8]) -> bool,
) -> Result<Option<ObjRef>, Failure> {
    let start = super::required_position_argument(interp, args, 0)?;
    if start > interp.text_len(receiver) {
        return Ok(Some(crate::eval::logical(false)));
    }
    let other = super::string_method_argument(interp, args, 1)?;
    let region = match_region_arguments(interp, args, other.len())?;
    let answer = {
        let bytes = interp.to_text(receiver);
        match_region_over(&bytes, start, &other, region, matches)
    };
    interp.give_result_buffer(other);
    Ok(Some(crate::eval::logical(answer)))
}

fn native_string_matchchar(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_matchchar(interp, receiver, args, u8::eq)
}

fn native_string_caselessmatchchar(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_matchchar(interp, receiver, args, u8::eq_ignore_ascii_case)
}

/// Whether the byte at `position` is in the set, over the receiver's own text.
fn string_matchchar(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    matches: fn(&u8, &u8) -> bool,
) -> Result<Option<ObjRef>, Failure> {
    let position = super::required_position_argument(interp, args, 0)?;
    if position > interp.text_len(receiver) {
        return Ok(Some(crate::eval::logical(false)));
    }
    let set = super::string_method_argument(interp, args, 1)?;
    let answer = {
        let bytes = interp.to_text(receiver);
        bytes
            .get(position - 1)
            .is_some_and(|byte| set.iter().any(|member| matches(member, byte)))
    };
    interp.give_result_buffer(set);
    Ok(Some(crate::eval::logical(answer)))
}

fn native_string_substr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (start, length, pad) = substr_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        crate::builtin::string::substr_bytes(&mut out, &bytes, start, length, pad)?;
    }
    Ok(Some(interp.text_built(out)))
}

/// `substr`'s two-argument form, the length defaulting to one byte, capped at
/// the end of the receiver, and never padding.
fn native_string_brackets(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let start = super::required_position_argument(interp, args, 0)? - 1;
    let length = super::optional_length_argument(interp, args, 1)?.unwrap_or(1);
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        let capped = length.min(bytes.len().saturating_sub(start));
        crate::builtin::string::substr_bytes(&mut out, &bytes, start, Some(capped), b' ')?;
    }
    Ok(Some(interp.text_built(out)))
}

/// The one byte at `position`, or the empty string past the end.
fn native_string_subchar(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = super::required_position_argument(interp, args, 0)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        out.extend(bytes.get(position - 1));
    }
    Ok(Some(interp.text_built(out)))
}

fn native_string_subword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = super::required_position_argument(interp, args, 0)?;
    let count = super::optional_length_argument(interp, args, 1)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        let found = crate::builtin::word::subword_range(&bytes, position, count);
        out.extend_from_slice(&bytes[found]);
    }
    Ok(Some(interp.text_built(out)))
}

fn native_string_subwords(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = super::optional_position_argument(interp, args, 0)?.unwrap_or(1);
    let count = super::optional_length_argument(interp, args, 1)?.unwrap_or(usize::MAX);
    let words: Vec<Vec<u8>> = {
        let bytes = interp.to_text(receiver);
        crate::builtin::word::word_slices(&bytes)
            .into_iter()
            .skip(position - 1)
            .take(count)
            .map(<[u8]>::to_vec)
            .collect()
    };
    Ok(Some(super::array_of_texts(interp, words)?))
}

fn native_string_word(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = super::required_position_argument(interp, args, 0)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        let found = crate::builtin::word::word_range(&bytes, position).unwrap_or(0..0);
        out.extend_from_slice(&bytes[found]);
    }
    Ok(Some(interp.text_built(out)))
}

/// The 1-based byte at which the word starts, or 0.
fn native_string_wordindex(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = super::required_position_argument(interp, args, 0)?;
    let index = {
        let bytes = interp.to_text(receiver);
        crate::builtin::word::word_range(&bytes, position).map_or(0, |word| word.start + 1)
    };
    Ok(Some(interp.counted(index)))
}

fn native_string_wordlength(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = super::required_position_argument(interp, args, 0)?;
    let length = {
        let bytes = interp.to_text(receiver);
        crate::builtin::word::word_range(&bytes, position).map_or(0, |word| word.len())
    };
    Ok(Some(interp.counted(length)))
}

fn native_string_words(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let count = {
        let bytes = interp.to_text(receiver);
        crate::builtin::word::word_count(&bytes)
    };
    Ok(Some(interp.counted(count)))
}

fn native_string_verify(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (reference, option, start, range) = verify_arguments(interp, args)?;
    let answer = {
        let bytes = interp.to_text(receiver);
        crate::builtin::string::verify_bytes(&bytes, &reference, option, start, range)
    };
    interp.give_result_buffer(reference);
    Ok(Some(interp.counted(answer)))
}

fn native_string_insert(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (new, begin, length, pad) = insert_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        crate::builtin::string::insert_bytes(&mut out, &bytes, &new, begin, length, pad)?;
    }
    interp.give_result_buffer(new);
    Ok(Some(interp.text_built(out)))
}

fn native_string_overlay(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (new, begin, length, pad) = overlay_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        crate::builtin::string::overlay_bytes(&mut out, &bytes, &new, begin, length, pad)?;
    }
    interp.give_result_buffer(new);
    Ok(Some(interp.text_built(out)))
}

fn native_string_replaceat(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let new = super::string_method_argument(interp, args, 0)?;
    let begin = super::required_position_argument(interp, args, 1)? - 1;
    let length = super::optional_length_argument(interp, args, 2)?;
    let pad = super::pad_method_argument(interp, args, 3)?.unwrap_or(b' ');
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        let (replaced, _) = replace_at_plan(bytes.len(), begin, length, new.len());
        replace_at_bytes(&mut out, &bytes, &new, begin, replaced, pad)?;
    }
    interp.give_result_buffer(new);
    Ok(Some(interp.text_built(out)))
}

fn native_string_delstr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (begin, range) = delete_arguments(interp, args)?;
    if begin >= interp.text_len(receiver) {
        return Ok(Some(receiver));
    }
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        out.extend_from_slice(&bytes);
    }
    crate::builtin::string::delete_range(&mut out, begin, range);
    Ok(Some(interp.text_built(out)))
}

fn native_string_delword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (position, count) = delword_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        out.extend_from_slice(&bytes);
    }
    let length = out.len();
    crate::builtin::word::delword_bytes(&mut out, position, count);
    // `delword_bytes` removes at least one byte whenever it removes anything.
    if length > 0 && out.len() == length {
        interp.give_result_buffer(out);
        return Ok(Some(receiver));
    }
    Ok(Some(interp.text_built(out)))
}

fn native_string_changestr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_changestr(
        interp,
        receiver,
        args,
        crate::builtin::string::changestr_bytes,
        crate::builtin::string::find_forward,
    )
}

fn native_string_caselesschangestr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_changestr(
        interp,
        receiver,
        args,
        crate::builtin::string::caseless_changestr_bytes,
        crate::builtin::string::caseless_find_forward,
    )
}

/// Every occurrence of the needle replaced, into a new string.
type ChangeStr = fn(&mut Vec<u8>, &[u8], &[u8], &[u8], usize) -> Result<(), super::Raised>;

/// [`changestr_arguments`]' replacement over the receiver's own text.
fn string_changestr(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    change: ChangeStr,
    find: fn(&[u8], &[u8], usize, usize) -> usize,
) -> Result<Option<ObjRef>, Failure> {
    let (needle, replacement, limit) = changestr_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        // `String~changeStr` answers the receiver itself when it changes
        // nothing.
        if limit == 0 || find(&bytes, &needle, 0, bytes.len()) == 0 {
            drop(bytes);
            interp.give_result_buffer(out);
            interp.give_result_buffer(needle);
            interp.give_result_buffer(replacement);
            return Ok(Some(receiver));
        }
        change(&mut out, &bytes, &needle, &replacement, limit)?;
    }
    interp.give_result_buffer(needle);
    interp.give_result_buffer(replacement);
    Ok(Some(interp.text_built(out)))
}

fn native_string_translate(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if args.iter().take(3).all(Option::is_none) {
        return string_case_shift(interp, receiver, args, 3, u8::to_ascii_uppercase);
    }
    let (out_table, in_table, pad, start, range) = translate_arguments(interp, args)?;
    if untouched_window(interp.text_len(receiver), start, range) {
        return Ok(Some(receiver));
    }
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        out.extend_from_slice(&bytes);
    }
    // A supplied input table is one even when it is empty, which matches no
    // byte: `String~translate` reads the table only where it was omitted,
    // where `MutableBuffer`'s reads its length.
    let supplied = args.get(1).copied().flatten().is_some();
    crate::builtin::string::translate_bytes(
        &mut out,
        &out_table,
        supplied.then_some(in_table.as_slice()),
        pad,
        start,
        range,
    );
    Ok(Some(interp.text_built(out)))
}

fn native_string_lower(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_case_shift(interp, receiver, args, 0, u8::to_ascii_lowercase)
}

/// A case shift over the receiver's own text, answering a new string.
fn string_case_shift(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    first: usize,
    shift: fn(&u8) -> u8,
) -> Result<Option<ObjRef>, Failure> {
    let (start, range) = case_shift_arguments(interp, args, first)?;
    if untouched_window(interp.text_len(receiver), start, range) {
        return Ok(Some(receiver));
    }
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        out.extend_from_slice(&bytes);
    }
    crate::builtin::string::case_shift_bytes(&mut out, start, range, shift);
    Ok(Some(interp.text_built(out)))
}

fn native_string_space(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (gap, pad) = space_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        crate::builtin::string::space_bytes(&mut out, &bytes, gap, pad)?;
    }
    Ok(Some(interp.text_built(out)))
}

/// `String~append`, which is **`String~concat` under a second name**, so it
/// concatenates and takes exactly one argument. It routes to
/// [`Interp::apply_binary`] for that reason rather than reading its argument
/// as a string: concatenation *renders* what it is given, so `.nil` becomes
/// `The NIL object` where a string argument would be refused at 88.909.
fn native_string_append(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(tail) = args.first().copied().flatten() else {
        return Err(super::Raised::missing_method_argument(1).into());
    };
    concatenation_or_this(interp, Operator::Concatenate, receiver, tail)
}

/// Every binary operator `String` answers as a message, and the
/// [`interrexx_parse::Operator`] each one is.
#[cfg(test)]
const BINARY_OPERATORS: &[(&str, Operator)] = &[
    ("+", Operator::Plus),
    ("-", Operator::Subtract),
    ("*", Operator::Multiply),
    ("**", Operator::Power),
    ("/", Operator::Divide),
    ("%", Operator::IntDiv),
    ("//", Operator::Remainder),
    ("", Operator::Abuttal),
    ("||", Operator::Concatenate),
    (" ", Operator::Blank),
    ("=", Operator::Equal),
    ("\\=", Operator::BackslashEqual),
    ("<>", Operator::LessThanGreaterThan),
    ("><", Operator::GreaterThanLessThan),
    (">", Operator::GreaterThan),
    ("<", Operator::LessThan),
    (">=", Operator::GreaterThanEqual),
    ("\\<", Operator::BackslashLessThan),
    ("<=", Operator::LessThanEqual),
    ("\\>", Operator::BackslashGreaterThan),
    ("==", Operator::StrictEqual),
    ("\\==", Operator::StrictBackslashEqual),
    (">>", Operator::StrictGreaterThan),
    ("<<", Operator::StrictLessThan),
    (">>=", Operator::StrictGreaterThanEqual),
    ("\\<<", Operator::StrictBackslashLessThan),
    ("<<=", Operator::StrictLessThanEqual),
    ("\\>>", Operator::StrictBackslashGreaterThan),
    ("&", Operator::And),
    ("|", Operator::Or),
    ("&&", Operator::Xor),
];

/// `String~"+"`, [`Operator::Plus`] through the arithmetic pair.
fn native_string_op_plus(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_arithmetic_operator(interp, receiver, args, Operator::Plus)
}

/// `String~"-"`, [`Operator::Subtract`] through the arithmetic pair.
fn native_string_op_subtract(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_arithmetic_operator(interp, receiver, args, Operator::Subtract)
}

/// `String~"*"`, [`Operator::Multiply`] through the arithmetic pair.
fn native_string_op_multiply(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_arithmetic_operator(interp, receiver, args, Operator::Multiply)
}

/// `String~"**"`, [`Operator::Power`] through the arithmetic pair.
fn native_string_op_power(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_arithmetic_operator(interp, receiver, args, Operator::Power)
}

/// `String~"/"`, [`Operator::Divide`] through the arithmetic pair.
fn native_string_op_divide(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_arithmetic_operator(interp, receiver, args, Operator::Divide)
}

/// `String~"%"`, [`Operator::IntDiv`] through the arithmetic pair.
fn native_string_op_intdiv(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_arithmetic_operator(interp, receiver, args, Operator::IntDiv)
}

/// `String~"//"`, [`Operator::Remainder`] through the arithmetic pair.
fn native_string_op_remainder(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_arithmetic_operator(interp, receiver, args, Operator::Remainder)
}

/// `String~""`, [`Operator::Abuttal`] through [`Interp::apply_binary`].
fn native_string_op_abuttal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::Abuttal)
}

/// `String~"||"`, [`Operator::Concatenate`] through [`Interp::apply_binary`].
fn native_string_op_concatenate(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::Concatenate)
}

/// `String~" "`, [`Operator::Blank`] through [`Interp::apply_binary`].
fn native_string_op_blank(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::Blank)
}

/// `String~"="`, [`Operator::Equal`] through [`Interp::apply_binary`].
fn native_string_op_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::Equal)
}

/// `String~"\\="`, [`Operator::BackslashEqual`] through [`Interp::apply_binary`].
fn native_string_op_backslash_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::BackslashEqual)
}

/// `String~"<>"`, [`Operator::LessThanGreaterThan`] through [`Interp::apply_binary`].
fn native_string_op_less_greater(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::LessThanGreaterThan)
}

/// `String~"><"`, [`Operator::GreaterThanLessThan`] through [`Interp::apply_binary`].
fn native_string_op_greater_less(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::GreaterThanLessThan)
}

/// `String~">"`, [`Operator::GreaterThan`] through [`Interp::apply_binary`].
fn native_string_op_greater(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::GreaterThan)
}

/// `String~"<"`, [`Operator::LessThan`] through [`Interp::apply_binary`].
fn native_string_op_less(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::LessThan)
}

/// `String~">="`, [`Operator::GreaterThanEqual`] through [`Interp::apply_binary`].
fn native_string_op_greater_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::GreaterThanEqual)
}

/// `String~"\\<"`, [`Operator::BackslashLessThan`] through [`Interp::apply_binary`].
fn native_string_op_backslash_less(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::BackslashLessThan)
}

/// `String~"<="`, [`Operator::LessThanEqual`] through [`Interp::apply_binary`].
fn native_string_op_less_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::LessThanEqual)
}

/// `String~"\\>"`, [`Operator::BackslashGreaterThan`] through [`Interp::apply_binary`].
fn native_string_op_backslash_greater(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::BackslashGreaterThan)
}

/// `String~"=="`, [`Operator::StrictEqual`] through [`Interp::apply_binary`].
fn native_string_op_strict_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictEqual)
}

/// `String~"\\=="`, [`Operator::StrictBackslashEqual`] through [`Interp::apply_binary`].
fn native_string_op_strict_backslash_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictBackslashEqual)
}

/// `String~">>"`, [`Operator::StrictGreaterThan`] through [`Interp::apply_binary`].
fn native_string_op_strict_greater(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictGreaterThan)
}

/// `String~"<<"`, [`Operator::StrictLessThan`] through [`Interp::apply_binary`].
fn native_string_op_strict_less(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictLessThan)
}

/// `String~">>="`, [`Operator::StrictGreaterThanEqual`] through [`Interp::apply_binary`].
fn native_string_op_strict_greater_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictGreaterThanEqual)
}

/// `String~"\\<<"`, [`Operator::StrictBackslashLessThan`] through [`Interp::apply_binary`].
fn native_string_op_strict_backslash_less(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictBackslashLessThan)
}

/// `String~"<<="`, [`Operator::StrictLessThanEqual`] through [`Interp::apply_binary`].
fn native_string_op_strict_less_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictLessThanEqual)
}

/// `String~"\\>>"`, [`Operator::StrictBackslashGreaterThan`] through [`Interp::apply_binary`].
fn native_string_op_strict_backslash_greater(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictBackslashGreaterThan)
}

/// `String~"&"`, [`Operator::And`] through [`Interp::apply_binary`].
fn native_string_op_and(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::And)
}

/// `String~"|"`, [`Operator::Or`] through [`Interp::apply_binary`].
fn native_string_op_or(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::Or)
}

/// `String~"&&"`, [`Operator::Xor`] through [`Interp::apply_binary`].
fn native_string_op_xor(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::Xor)
}

/// One arithmetic operator sent as a message.
fn string_arithmetic_operator(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    op: Operator,
) -> Result<Option<ObjRef>, Failure> {
    let receiver = base_string(interp, receiver);
    if args.first().copied().flatten().is_none() {
        match op {
            // **`+` and `-` sent with no operand are the prefix forms, not an
            // error.** `'12'~'+'()` is `12` and `'12'~'-'()` is `-12`, so the
            // unary reading belongs to exactly the two operators the language
            // has a prefix form for.
            Operator::Plus => {
                return Ok(Some(
                    interp.apply_prefix(interrexx_parse::PrefixOp::Plus, receiver)?,
                ));
            }
            Operator::Subtract => {
                return Ok(Some(
                    interp.apply_prefix(interrexx_parse::PrefixOp::Minus, receiver)?,
                ));
            }
            // The other five convert the receiver *before* they complain about
            // the operand. For example, `.String~new('abc')~'*'()` is 41.1
            // naming `"abc"`, while `'12'~'*'()` is 93.903 -- so a receiver
            // that is not a number is what the send reports, and the missing
            // argument only surfaces once the receiver converts.
            _ => {
                interp.arith_operand(receiver)?;
            }
        }
    }
    let other = super::operator_argument(args)?;
    let value = match interp.arith_small_int(op, receiver, other) {
        Some(value) => value,
        None => interp.arith_general(op, receiver, other)?,
    };
    Ok(Some(value))
}

/// One binary operator sent as a message.
fn string_binary_operator(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    op: Operator,
) -> Result<Option<ObjRef>, Failure> {
    let other = super::operator_argument(args)?;
    concatenation_or_this(interp, op, receiver, other)
}

/// `op` applied to `receiver`'s string value and `other`, where a
/// concatenation that adds nothing answers the receiver itself.
fn concatenation_or_this(
    interp: &mut Interp,
    op: Operator,
    receiver: ObjRef,
    other: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    let base = base_string(interp, receiver);
    let answer = interp.apply_binary(op, base, other)?;
    if base != receiver
        && matches!(op, Operator::Concatenate | Operator::Abuttal)
        && interp.text_len(answer) == interp.text_len(base)
    {
        return Ok(Some(receiver));
    }
    Ok(Some(answer))
}

/// Whether a range of `length` bytes from 0-based `start` changes nothing: the
/// case `upper`, `lower` and `translate` answer the receiver itself for.
fn untouched_window(length: usize, start: usize, range: Option<usize>) -> bool {
    start >= length || range == Some(0)
}

/// `String~"\\"`, the prefix `\` as a message.
fn native_string_op_not(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let receiver = base_string(interp, receiver);
    Ok(Some(
        interp.apply_prefix(interrexx_parse::PrefixOp::Not, receiver)?,
    ))
}

/// `String~"?"`.
fn native_string_op_choice(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let on_true = args
        .first()
        .copied()
        .flatten()
        .ok_or_else(|| Raised::missing_named_argument("true value"))?;
    let on_false = args
        .get(1)
        .copied()
        .flatten()
        .ok_or_else(|| Raised::missing_named_argument("false value"))?;
    let holds = interp.truth(receiver, Raised::not_logical)?;
    Ok(Some(if holds { on_true } else { on_false }))
}

/// `String~"CENTER"` and `String~"CENTRE"`.
fn native_string_center(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (width, pad) = pad_arguments(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    match crate::builtin::string::center_bytes(interp, &bytes, width, pad)? {
        Some(out) => Ok(Some(interp.text_built(out))),
        None => Ok(Some(receiver)),
    }
}

/// `String~"LEFT"`.
fn native_string_left(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (size, pad) = pad_arguments(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    let out = crate::builtin::string::left_bytes(interp, &bytes, size, pad)?;
    Ok(Some(interp.text_built(out)))
}

/// `String~"RIGHT"`.
fn native_string_right(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (size, pad) = pad_arguments(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    match crate::builtin::string::right_bytes(interp, &bytes, size, pad)? {
        Some(out) => Ok(Some(interp.text_built(out))),
        None => Ok(Some(receiver)),
    }
}

/// `String~"COPIES"`.
fn native_string_copies(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let count = copies_argument(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    match crate::builtin::string::copies_bytes(interp, &bytes, count)? {
        Some(out) => Ok(Some(interp.text_built(out))),
        None => Ok(Some(receiver)),
    }
}

/// `String~"STRIP"`.
fn native_string_strip(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (option, set) = strip_arguments(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    let kept = crate::builtin::string::strip_bytes(&bytes, option, set.as_deref());
    if kept.len() == bytes.len() && !kept.is_empty() {
        return Ok(Some(receiver));
    }
    Ok(Some(interp.text(kept)))
}

/// `String~"ABBREV"`.
fn native_string_abbrev(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (info, minimum) = abbrev_arguments(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    let holds = crate::builtin::string::abbrev_holds(&bytes, &info, minimum);
    let answer = interp.text(if holds { b"1" } else { b"0" });
    interp.give_result_buffer(info);
    Ok(Some(answer))
}

/// `String~"COMPARE"`.
fn native_string_compare(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (other, pad) = compare_arguments(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    let at = crate::builtin::string::compare_at(&bytes, &other, pad);
    let answer = interp.counted(at);
    interp.give_result_buffer(other);
    Ok(Some(answer))
}

/// `String~"B2X"`.
fn native_string_b2x(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let bytes = interp.to_text(receiver).to_vec();
    let out = crate::builtin::convert::b2x_bytes(interp, &bytes)?;
    Ok(Some(interp.text_built(out)))
}

/// `String~"C2X"`.
fn native_string_c2x(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let bytes = interp.to_text(receiver).to_vec();
    let out = crate::builtin::convert::c2x_bytes(interp, &bytes)?;
    Ok(Some(interp.text_built(out)))
}

/// `String~"X2B"`.
fn native_string_x2b(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let bytes = interp.to_text(receiver).to_vec();
    let out = crate::builtin::convert::x2b_bytes(interp, &bytes)?;
    Ok(Some(interp.text_built(out)))
}

/// `String~"X2C"`.
fn native_string_x2c(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let bytes = interp.to_text(receiver).to_vec();
    let out = crate::builtin::convert::x2c_bytes(&bytes)?;
    Ok(Some(interp.text_built(out)))
}

/// `String~"C2D"`.
fn native_string_c2d(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let requested = conversion_length_argument(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    let value = crate::builtin::convert::x2d_c2d_over(interp, &bytes, requested, true)?;
    Ok(Some(value))
}

/// `String~"X2D"`.
fn native_string_x2d(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let requested = conversion_length_argument(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    let value = crate::builtin::convert::x2d_c2d_over(interp, &bytes, requested, false)?;
    Ok(Some(value))
}

/// `String~"D2C"`.
fn native_string_d2c(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_d2x_d2c(interp, receiver, args, true)
}

/// `String~"D2X"`.
fn native_string_d2x(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_d2x_d2c(interp, receiver, args, false)
}

/// `D2C` and `D2X` as messages, which differ only in the flag.
fn string_d2x_d2c(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    character: bool,
) -> Result<Option<ObjRef>, Failure> {
    let receiver = base_string(interp, receiver);
    let numeric = interp.to_number(receiver).is_ok();
    let text = interp.to_text(receiver).to_vec();
    let value =
        crate::builtin::convert::d2x_d2c_over(interp, &text, numeric, character, |interp| {
            conversion_length_argument(interp, args)
        })?;
    Ok(Some(value))
}

/// `String~"BITAND"`.
fn native_string_bitand(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_bit_operation(interp, receiver, args, |a, b| a & b, convert::BITAND_PAD)
}

/// `String~"BITOR"`.
fn native_string_bitor(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_bit_operation(interp, receiver, args, |a, b| a | b, convert::BITOR_PAD)
}

/// `String~"BITXOR"`.
fn native_string_bitxor(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_bit_operation(interp, receiver, args, |a, b| a ^ b, convert::BITOR_PAD)
}

/// One bit operation sent as a message.
fn string_bit_operation(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    operation: fn(u8, u8) -> u8,
    default_pad: u8,
) -> Result<Option<ObjRef>, Failure> {
    let (other, pad) = bit_arguments(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    let out = convert::bit_operation_over(
        interp,
        &bytes,
        &other,
        pad.unwrap_or(default_pad),
        operation,
    )?;
    let answer = interp.text_built(out);
    interp.give_result_buffer(other);
    Ok(Some(answer))
}

/// `String~"DATATYPE"`.
fn native_string_datatype(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let letter = datatype_option_argument(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    let Some(letter) = letter else {
        let kind = crate::builtin::datatype::datatype_kind(&bytes);
        return Ok(Some(interp.text(kind)));
    };
    let Some(matched) = crate::builtin::datatype::datatype_matches(interp, &bytes, letter)? else {
        return Err(Raised::method_option_not_recognised(
            crate::builtin::datatype::DATATYPE_OPTIONS,
            &[letter],
        )
        .into());
    };
    Ok(Some(interp.text(if matched { b"1" } else { b"0" })))
}

/// The receiver of a numeric method, or 93.943 naming the method.
fn numeric_receiver(
    interp: &mut Interp,
    receiver: ObjRef,
    method: &[u8],
) -> Result<interrexx_num::Number, Failure> {
    let receiver = base_string(interp, receiver);
    match interp.to_number(receiver) {
        Ok(number) => Ok(number),
        Err(_) => {
            let found = interp.string_value_text(receiver)?;
            Err(Raised::method_target_not_a_number(method, &found).into())
        }
    }
}

/// `String~"ABS"`.
fn native_string_abs(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let value = numeric_receiver(interp, receiver, b"ABS")?;
    crate::builtin::numeric::abs_of(interp, &value).map(Some)
}

/// `String~"TRUNC"`.
fn native_string_trunc(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let value = numeric_receiver(interp, receiver, b"TRUNC")?;
    let places = count_method_argument(interp, args, 0)?;
    let answer = crate::builtin::numeric::trunc_of(interp, &value, places)?;
    Ok(Some(answer))
}

/// `String~"ENCODEBASE64"`.
fn native_string_encode_base64(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let source = interp.to_text(receiver).into_owned();
    let answer = convert::encode_base64_bytes(&source);
    Ok(Some(interp.text_built(answer)))
}

/// `String~"DECODEBASE64"`.
fn native_string_decode_base64(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let source = interp.to_text(receiver).into_owned();
    let Some(answer) = convert::decode_base64_bytes(&source) else {
        return Err(Raised::invalid_base64().into());
    };
    Ok(Some(interp.text_built(answer)))
}

/// `String~"FLOOR"`.
fn native_string_floor(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let value = numeric_receiver(interp, receiver, b"FLOOR")?;
    Ok(Some(numeric::integer_of(
        interp,
        &value,
        interrexx_num::Number::floor,
    )))
}

/// `String~"CEILING"`.
fn native_string_ceiling(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let value = numeric_receiver(interp, receiver, b"CEILING")?;
    Ok(Some(numeric::integer_of(
        interp,
        &value,
        interrexx_num::Number::ceiling,
    )))
}

/// `String~"ROUND"`.
fn native_string_round(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let value = numeric_receiver(interp, receiver, b"ROUND")?;
    Ok(Some(numeric::integer_of(
        interp,
        &value,
        interrexx_num::Number::round,
    )))
}

/// `String~"MODULO"`.
fn native_string_modulo(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let value = numeric_receiver(interp, receiver, b"MODULO")?;
    let digits = numeric::digits_setting(interp);
    if !value.is_integer(digits) {
        let found = interp.string_value_text(receiver)?;
        return Err(Raised::method_target_not_whole(b"MODULO", &found).into());
    }
    let Some(object) = args.first().copied().flatten() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let divisor = match interp.requested_number(object)? {
        Some(number) if number.is_integer(digits) && number.signum() == 1 => number,
        _ => {
            let found = interp.string_value_text(object)?;
            return Err(Raised::method_argument_not_positive(1, &found).into());
        }
    };
    Ok(Some(numeric::modulo_over(interp, &value, &divisor)?))
}

/// `String~"FORMAT"`.
fn native_string_format(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let value = numeric_receiver(interp, receiver, b"FORMAT")?;
    let before = count_method_argument(interp, args, 0)?;
    let after = count_method_argument(interp, args, 1)?;
    let expp = count_method_argument(interp, args, 2)?;
    let expt = count_method_argument(interp, args, 3)?;
    let answer = crate::builtin::numeric::format_over(interp, &value, before, after, expp, expt)?;
    Ok(Some(answer))
}

/// `String~"MAX"`.
fn native_string_max(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_max_min(interp, receiver, args, b"MAX", numeric::Extreme::Max)
}

/// `String~"MIN"`.
fn native_string_min(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_max_min(interp, receiver, args, b"MIN", numeric::Extreme::Min)
}

/// `MAX` and `MIN` as messages, which differ only in which end they want.
fn string_max_min(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    method: &'static [u8],
    want: numeric::Extreme,
) -> Result<Option<ObjRef>, Failure> {
    let value = numeric_receiver(interp, receiver, method)?;
    let text = interp.to_text(receiver).to_vec();
    let answer = numeric::max_min_over(interp, method, &value, text, args, args, want, true)?;
    Ok(Some(answer))
}

/// `String~"EQUALS"`.
fn native_string_equals(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_equals(interp, receiver, args, false)
}

/// `String~"CASELESSEQUALS"`.
fn native_string_caseless_equals(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_equals(interp, receiver, args, true)
}

/// The two equality tests, which differ only in case folding.
fn string_equals(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    caseless: bool,
) -> Result<Option<ObjRef>, Failure> {
    let other = equals_argument(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    let holds = other.is_some_and(|other| {
        if caseless {
            crate::builtin::string::caseless_eq(&bytes, &other)
        } else {
            bytes == other
        }
    });
    Ok(Some(interp.text(if holds { b"1" } else { b"0" })))
}

/// `String~"CASELESSABBREV"`.
fn native_string_caseless_abbrev(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (info, minimum) = abbrev_arguments(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    let holds = crate::builtin::string::caseless_abbrev_holds(&bytes, &info, minimum);
    let answer = interp.text(if holds { b"1" } else { b"0" });
    interp.give_result_buffer(info);
    Ok(Some(answer))
}

/// `String~"CASELESSCOMPARE"`.
fn native_string_caseless_compare(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (other, pad) = compare_arguments(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    let at = crate::builtin::string::caseless_compare_at(&bytes, &other, pad);
    let answer = interp.counted(at);
    interp.give_result_buffer(other);
    Ok(Some(answer))
}

/// `String~"COMPARETO"`.
fn native_string_compare_to(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_compare_to(interp, receiver, args, false)
}

/// `String~"CASELESSCOMPARETO"`.
fn native_string_caseless_compare_to(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_compare_to(interp, receiver, args, true)
}

/// The two three-way comparisons, which differ only in case folding.
fn string_compare_to(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    caseless: bool,
) -> Result<Option<ObjRef>, Failure> {
    let (other, start, length) = compare_to_arguments(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    let length = length.unwrap_or_else(|| {
        bytes
            .len()
            .max(other.len())
            .saturating_sub(start)
            .saturating_add(1)
    });
    let ordering = crate::builtin::string::compare_to_over(&bytes, &other, start, length, caseless);
    let answer = interp.text(match ordering {
        -1 => b"-1".as_slice(),
        0 => b"0",
        _ => b"1",
    });
    interp.give_result_buffer(other);
    Ok(Some(answer))
}

/// `String~sign`: the `SIGN` builtin's own computation on the receiver.
pub(super) fn native_string_sign(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Ok(value) = interp.to_number(receiver) else {
        let found = interp.string_value_text(receiver)?;
        return Err(Raised::method_target_not_a_number(b"SIGN", &found).into());
    };
    crate::builtin::numeric::sign_of(interp, &value).map(Some)
}

/// `String~makeString`: a string is its own string value, which
/// `Object~request("STRING")` is what finds. A `String` subclass instance
/// answers a new `String` holding its bytes, since only a base-class string is
/// its own string value.
pub(super) fn native_string_make_string(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(base_string(interp, receiver)))
}

/// A `String` subclass instance's bytes, or `None` for any other value.
pub(crate) fn subclass_text(interp: &Interp, value: ObjRef) -> Option<&[u8]> {
    if !matches!(value.decode(), super::Decoded::Heap { .. }) {
        return None;
    }
    match &interp.heap.get(value)?.body {
        Body::Instance {
            native: Some(state),
            ..
        } => state.text(),
        _ => None,
    }
}

/// `value` as a base-class string: a `String` subclass instance's bytes in a
/// new rooted `String`, and any other value unchanged.
pub(super) fn base_string(interp: &mut Interp, value: ObjRef) -> ObjRef {
    let Some(bytes) = subclass_text(interp, value) else {
        return value;
    };
    let bytes = bytes.to_vec();
    let text = interp.text_built(bytes);
    interp.roots.activity_mut().push_temp(text);
    text
}

/// `String~upper([n [, length]])`: the receiver with a range of it uppercased,
/// bound with a declared count of 2.
pub(super) fn native_string_upper(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let start = match whole_method_argument(interp, args, 0, Raised::invalid_position)? {
        Some(value) if value > 0 => {
            usize_or_refuse(interp, args, 0, value, Raised::invalid_position)?
        }
        Some(_) => {
            return Err(refuse_method_argument(
                interp,
                args,
                0,
                Raised::invalid_position,
            ));
        }
        None => 1,
    } - 1;
    let text = interp.to_text(receiver).into_owned();
    let range = match whole_method_argument(interp, args, 1, Raised::invalid_length)? {
        Some(value) if value >= 0 => {
            usize_or_refuse(interp, args, 1, value, Raised::invalid_length)?
        }
        Some(_) => {
            return Err(refuse_method_argument(
                interp,
                args,
                1,
                Raised::invalid_length,
            ));
        }
        None => text.len(),
    };
    if untouched_window(text.len(), start, Some(range)) {
        return Ok(Some(receiver));
    }
    let range = range.min(text.len() - start);
    let mut result = text;
    for byte in &mut result[start..start + range] {
        *byte = byte.to_ascii_uppercase();
    }
    Ok(Some(interp.text_built(result)))
}

/// `String~length`: the receiver's own byte count.
pub(super) fn native_length(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let length = interp.text_len(receiver);
    Ok(Some(interp.counted(length)))
}

/// `String~makeArray`: the receiver's lines, one array element each, or its
/// pieces between `separator`s where one is given.
pub(super) fn native_string_makearray(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let separator = super::method_arguments::optional_string_or_none_argument(interp, args, 0)?;
    let bytes = interp.to_text(receiver).to_vec();
    let lines = match &separator {
        Some(separator) => crate::builtin::string::split_slices(&bytes, separator),
        None => crate::builtin::string::line_slices(&bytes),
    };
    // **Each line is rooted as it is built.** The `Vec` gathering them is a
    // Rust local and invisible to the collector, so a line built earlier is
    // swept by the allocation of the next one -- and, for a single-line
    // string, by the `alloc_with` below. Measured under
    // `run_program_collect_every_alloc`: `'.RESOURCES'~makeArray` then
    // `a[1]` panicked at `not_in_arena`'s "a live value", while `'abc'`
    // survived only because a short string is inline and never a heap object
    // at all.
    let mut slots: Vec<Option<ObjRef>> = Vec::with_capacity(lines.len());
    for line in lines {
        let text = interp.text_built(line.to_vec());
        interp.roots.activity_mut().push_temp(text);
        slots.push(Some(text));
    }
    // For example, `''~makeArray~dimension` is 0, so an empty result carries no
    // dimensions rather than one of size 0.
    let body = Body::Array {
        dimensions: None,
        slots: slots.into(),
    };
    let object = interp.alloc_charged(BehaviourId::ARRAY, body);
    interp.roots.activity_mut().push_temp(object);
    let caller = interp.caller();
    interp.send_message(object, INIT, None, &[], caller)?;
    Ok(Some(object))
}

/// `String~reverse`: the receiver's own bytes, last to first.
pub(super) fn native_reverse(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if interp.text_len(receiver) <= 1 {
        return Ok(Some(receiver));
    }
    let mut bytes = interp.to_text(receiver).to_vec();
    bytes.reverse();
    Ok(Some(interp.text_built(bytes)))
}

/// One class-side character-class method: the ASCII bytes `members`
/// accepts, in ascending order (`r[class.string.constants]`). The
/// predicate is written at the one method that answers it, so no method
/// can be paired with another's class.
fn class_constant(interp: &mut Interp, members: fn(u8) -> bool) -> Result<Option<ObjRef>, Failure> {
    let text: Vec<u8> = (0..=0x7f_u8).filter(|&b| members(b)).collect();
    Ok(Some(interp.text_built(text)))
}

fn native_class_alnum(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| b.is_ascii_alphanumeric())
}

fn native_class_alpha(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| b.is_ascii_alphabetic())
}

fn native_class_blank(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| b == b' ' || b == b'\t')
}

fn native_class_cntrl(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| b < 0x20 || b == 0x7f)
}

fn native_class_cr(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| b == b'\r')
}

fn native_class_digit(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| b.is_ascii_digit())
}

fn native_class_graph(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| b.is_ascii_graphic())
}

fn native_class_lower(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| b.is_ascii_lowercase())
}

fn native_class_nl(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| b == b'\n')
}

fn native_class_null(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| b == 0)
}

fn native_class_print(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| (0x20..=0x7e).contains(&b))
}

fn native_class_punct(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| b.is_ascii_punctuation())
}

fn native_class_space(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| b == b' ' || (0x09..=0x0d).contains(&b))
}

fn native_class_tab(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| b == b'\t')
}

fn native_class_upper(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| b.is_ascii_uppercase())
}

fn native_class_xdigit(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_constant(interp, |b| b.is_ascii_hexdigit())
}

/// `String`'s class-side character-class methods, by the name each is
/// answered under (`r[class.string.constants]`).
const CHARACTER_CLASSES: [(&str, NativeMethod); 16] = [
    ("ALNUM", native_class_alnum),
    ("ALPHA", native_class_alpha),
    ("BLANK", native_class_blank),
    ("CNTRL", native_class_cntrl),
    ("CR", native_class_cr),
    ("DIGIT", native_class_digit),
    ("GRAPH", native_class_graph),
    ("LOWER", native_class_lower),
    ("NL", native_class_nl),
    ("NULL", native_class_null),
    ("PRINT", native_class_print),
    ("PUNCT", native_class_punct),
    ("SPACE", native_class_space),
    ("TAB", native_class_tab),
    ("UPPER", native_class_upper),
    ("XDIGIT", native_class_xdigit),
];

impl Interp {
    /// Puts `String`'s class-side character-class methods back as primitive
    /// rows, over whatever the library programs defined under those names
    /// (`r[class.string.constants]`). Runs inside the library bootstrap,
    /// after the programs and before they are sealed.
    pub(crate) fn reclaim_string_constants(&mut self) {
        let string = self
            .classes()
            .lookup("String")
            .expect("String is a native class");
        for (name, body) in CHARACTER_CLASSES {
            let method = self.classes().mint_method_id();
            self.classes().define_class_method(string, name, method);
            self.object_model().natives.insert(
                method,
                super::NativeEntry {
                    arity: Arity::Fixed(0),
                    body: super::NativeBody::Run(body),
                },
            );
            self.method_objects
                .remove(&(string, true, Box::from(name.as_bytes())));
        }
    }
}

/// `String`'s rows, in the shape [`super::NATIVE_METHODS`] uses.
pub(super) static NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    ("String", "", Arity::Fixed(1), native_string_op_abuttal),
    ("String", " ", Arity::Fixed(1), native_string_op_blank),
    ("String", "%", Arity::Fixed(1), native_string_op_intdiv),
    ("String", "&", Arity::Fixed(1), native_string_op_and),
    ("String", "&&", Arity::Fixed(1), native_string_op_xor),
    ("String", "*", Arity::Fixed(1), native_string_op_multiply),
    ("String", "**", Arity::Fixed(1), native_string_op_power),
    ("String", "+", Arity::Fixed(1), native_string_op_plus),
    ("String", "-", Arity::Fixed(1), native_string_op_subtract),
    ("String", "/", Arity::Fixed(1), native_string_op_divide),
    ("String", "//", Arity::Fixed(1), native_string_op_remainder),
    ("String", "<", Arity::Fixed(1), native_string_op_less),
    (
        "String",
        "<<",
        Arity::Fixed(1),
        native_string_op_strict_less,
    ),
    (
        "String",
        "<<=",
        Arity::Fixed(1),
        native_string_op_strict_less_equal,
    ),
    ("String", "<=", Arity::Fixed(1), native_string_op_less_equal),
    (
        "String",
        "<>",
        Arity::Fixed(1),
        native_string_op_less_greater,
    ),
    ("String", "=", Arity::Fixed(1), native_string_op_equal),
    (
        "String",
        "==",
        Arity::Fixed(1),
        native_string_op_strict_equal,
    ),
    ("String", ">", Arity::Fixed(1), native_string_op_greater),
    (
        "String",
        "><",
        Arity::Fixed(1),
        native_string_op_greater_less,
    ),
    (
        "String",
        ">=",
        Arity::Fixed(1),
        native_string_op_greater_equal,
    ),
    (
        "String",
        ">>",
        Arity::Fixed(1),
        native_string_op_strict_greater,
    ),
    (
        "String",
        ">>=",
        Arity::Fixed(1),
        native_string_op_strict_greater_equal,
    ),
    ("String", "?", Arity::Fixed(2), native_string_op_choice),
    ("String", "ABBREV", Arity::Fixed(2), native_string_abbrev),
    ("String", "ABS", Arity::Fixed(0), native_string_abs),
    ("String", "B2X", Arity::Fixed(0), native_string_b2x),
    (
        "String",
        "CASELESSABBREV",
        Arity::Fixed(2),
        native_string_caseless_abbrev,
    ),
    (
        "String",
        "CASELESSCOMPARE",
        Arity::Fixed(2),
        native_string_caseless_compare,
    ),
    (
        "String",
        "CASELESSCOMPARETO",
        Arity::Fixed(3),
        native_string_caseless_compare_to,
    ),
    (
        "String",
        "CASELESSEQUALS",
        Arity::Fixed(1),
        native_string_caseless_equals,
    ),
    ("String", "BITAND", Arity::Fixed(2), native_string_bitand),
    ("String", "BITOR", Arity::Fixed(2), native_string_bitor),
    ("String", "BITXOR", Arity::Fixed(2), native_string_bitxor),
    ("String", "APPEND", Arity::Fixed(1), native_string_append),
    ("String", "C2D", Arity::Fixed(1), native_string_c2d),
    ("String", "C2X", Arity::Fixed(0), native_string_c2x),
    ("String", "COMPARE", Arity::Fixed(2), native_string_compare),
    (
        "String",
        "COMPARETO",
        Arity::Fixed(3),
        native_string_compare_to,
    ),
    ("String", "CENTER", Arity::Fixed(2), native_string_center),
    ("String", "CENTRE", Arity::Fixed(2), native_string_center),
    ("String", "COPIES", Arity::Fixed(1), native_string_copies),
    ("String", "D2C", Arity::Fixed(1), native_string_d2c),
    ("String", "EQUALS", Arity::Fixed(1), native_string_equals),
    (
        "String",
        "DATATYPE",
        Arity::Fixed(1),
        native_string_datatype,
    ),
    ("String", "D2X", Arity::Fixed(1), native_string_d2x),
    ("String", "CEILING", Arity::Fixed(0), native_string_ceiling),
    (
        "String",
        "DECODEBASE64",
        Arity::Fixed(0),
        native_string_decode_base64,
    ),
    (
        "String",
        "ENCODEBASE64",
        Arity::Fixed(0),
        native_string_encode_base64,
    ),
    ("String", "FLOOR", Arity::Fixed(0), native_string_floor),
    ("String", "FORMAT", Arity::Fixed(4), native_string_format),
    ("String", "LEFT", Arity::Fixed(2), native_string_left),
    ("String", "MAX", Arity::Counted, native_string_max),
    ("String", "MODULO", Arity::Fixed(1), native_string_modulo),
    ("String", "MIN", Arity::Counted, native_string_min),
    ("String", "RIGHT", Arity::Fixed(2), native_string_right),
    ("String", "STRIP", Arity::Fixed(2), native_string_strip),
    ("String", "ROUND", Arity::Fixed(0), native_string_round),
    ("String", "TRUNC", Arity::Fixed(1), native_string_trunc),
    ("String", "X2B", Arity::Fixed(0), native_string_x2b),
    ("String", "X2C", Arity::Fixed(0), native_string_x2c),
    ("String", "X2D", Arity::Fixed(1), native_string_x2d),
    (
        "String",
        "CASELESSCHANGESTR",
        Arity::Fixed(3),
        native_string_caselesschangestr,
    ),
    (
        "String",
        "CASELESSCONTAINS",
        Arity::Fixed(3),
        native_string_caselesscontains,
    ),
    (
        "String",
        "CASELESSCONTAINSWORD",
        Arity::Fixed(2),
        native_string_caselesscontainsword,
    ),
    (
        "String",
        "CASELESSCOUNTSTR",
        Arity::Fixed(1),
        native_string_caselesscountstr,
    ),
    (
        "String",
        "CASELESSENDSWITH",
        Arity::Fixed(1),
        native_string_caselessendswith,
    ),
    (
        "String",
        "CASELESSLASTPOS",
        Arity::Fixed(3),
        native_string_caselesslastpos,
    ),
    (
        "String",
        "CASELESSMATCH",
        Arity::Fixed(4),
        native_string_caselessmatch,
    ),
    (
        "String",
        "CASELESSMATCHCHAR",
        Arity::Fixed(2),
        native_string_caselessmatchchar,
    ),
    (
        "String",
        "CASELESSPOS",
        Arity::Fixed(3),
        native_string_caselesspos,
    ),
    (
        "String",
        "CASELESSSTARTSWITH",
        Arity::Fixed(1),
        native_string_caselessstartswith,
    ),
    (
        "String",
        "CASELESSWORDPOS",
        Arity::Fixed(2),
        native_string_caselesswordpos,
    ),
    (
        "String",
        "CHANGESTR",
        Arity::Fixed(3),
        native_string_changestr,
    ),
    (
        "String",
        "CONTAINS",
        Arity::Fixed(3),
        native_string_contains,
    ),
    (
        "String",
        "CONTAINSWORD",
        Arity::Fixed(2),
        native_string_containsword,
    ),
    (
        "String",
        "COUNTSTR",
        Arity::Fixed(1),
        native_string_countstr,
    ),
    ("String", "DELSTR", Arity::Fixed(2), native_string_delstr),
    ("String", "DELWORD", Arity::Fixed(2), native_string_delword),
    (
        "String",
        "ENDSWITH",
        Arity::Fixed(1),
        native_string_endswith,
    ),
    ("String", "INSERT", Arity::Fixed(4), native_string_insert),
    ("String", "LASTPOS", Arity::Fixed(3), native_string_lastpos),
    ("String", "LOWER", Arity::Fixed(2), native_string_lower),
    ("String", "MATCH", Arity::Fixed(4), native_string_match),
    (
        "String",
        "MATCHCHAR",
        Arity::Fixed(2),
        native_string_matchchar,
    ),
    ("String", "OVERLAY", Arity::Fixed(4), native_string_overlay),
    ("String", "POS", Arity::Fixed(3), native_string_pos),
    (
        "String",
        "REPLACEAT",
        Arity::Fixed(4),
        native_string_replaceat,
    ),
    ("String", "SPACE", Arity::Fixed(2), native_string_space),
    (
        "String",
        "STARTSWITH",
        Arity::Fixed(1),
        native_string_startswith,
    ),
    ("String", "SUBCHAR", Arity::Fixed(1), native_string_subchar),
    ("String", "SUBSTR", Arity::Fixed(3), native_string_substr),
    ("String", "SUBWORD", Arity::Fixed(2), native_string_subword),
    (
        "String",
        "SUBWORDS",
        Arity::Fixed(2),
        native_string_subwords,
    ),
    (
        "String",
        "TRANSLATE",
        Arity::Fixed(5),
        native_string_translate,
    ),
    ("String", "VERIFY", Arity::Fixed(4), native_string_verify),
    ("String", "WORD", Arity::Fixed(1), native_string_word),
    (
        "String",
        "WORDINDEX",
        Arity::Fixed(1),
        native_string_wordindex,
    ),
    (
        "String",
        "WORDLENGTH",
        Arity::Fixed(1),
        native_string_wordlength,
    ),
    ("String", "WORDPOS", Arity::Fixed(2), native_string_wordpos),
    ("String", "WORDS", Arity::Fixed(0), native_string_words),
    ("String", "[]", Arity::Fixed(2), native_string_brackets),
    ("String", "\\", Arity::Fixed(0), native_string_op_not),
    (
        "String",
        "\\<",
        Arity::Fixed(1),
        native_string_op_backslash_less,
    ),
    (
        "String",
        "\\<<",
        Arity::Fixed(1),
        native_string_op_strict_backslash_less,
    ),
    (
        "String",
        "\\=",
        Arity::Fixed(1),
        native_string_op_backslash_equal,
    ),
    (
        "String",
        "\\==",
        Arity::Fixed(1),
        native_string_op_strict_backslash_equal,
    ),
    (
        "String",
        "\\>",
        Arity::Fixed(1),
        native_string_op_backslash_greater,
    ),
    (
        "String",
        "\\>>",
        Arity::Fixed(1),
        native_string_op_strict_backslash_greater,
    ),
    ("String", "|", Arity::Fixed(1), native_string_op_or),
    (
        "String",
        "||",
        Arity::Fixed(1),
        native_string_op_concatenate,
    ),
];

#[cfg(test)]
mod tests {
    use super::{BINARY_OPERATORS, NATIVE_METHODS};

    /// **The operator rows' names are the parser's own spellings**, so the
    /// table cannot name an operator the language does not have, and a
    /// spelling that changed would fail here rather than in a differential.
    #[test]
    fn operator_spellings_match_the_parser() {
        for (name, op) in BINARY_OPERATORS {
            assert_eq!(
                op.spelling(),
                *name,
                "the table spells {op:?} as {name:?} where the parser spells it {:?}",
                op.spelling()
            );
        }
    }

    /// Every operator [`BINARY_OPERATORS`] names has a row, so the two cannot
    /// come apart by an operator being listed and never registered.
    #[test]
    fn every_listed_operator_has_a_row() {
        for (name, _) in BINARY_OPERATORS {
            assert!(
                NATIVE_METHODS.iter().any(|(_, method, ..)| method == name),
                "{name:?} is listed as an operator and has no row"
            );
        }
    }
}
