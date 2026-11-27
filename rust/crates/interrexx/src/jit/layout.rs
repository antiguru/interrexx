//! **The one description of the memory generated code touches**, read by
//! Rust as types and by the code generator as offsets taken from those same
//! types.
//!
//! Generated code reaches three things: the [`Native`] block, the register
//! file it points at, and the window of variable slots it points at. Each
//! offset below is `offset_of!` of the field it names, and each encoding is
//! computed from the `interrexx-core` function that defines it, so a change
//! to a layout or an encoding changes what the code generator emits rather
//! than leaving it reading the old one. Nothing here is `unsafe`: raw
//! pointers are only held, and only generated code follows them.

use std::mem::offset_of;
use std::ptr;

use interrexx_core::{ObjRef, VarSlot};

/// The block generated code runs against: the interpreter's clause state and
/// countdown held here while generated code runs, and where its registers
/// and the running frame's slots are. [`super::Host::enter`] writes it back,
/// and [`super::Host::leave`] fills it.
#[repr(C)]
pub(crate) struct Native {
    /// Register 0 of the running frame.
    pub(crate) registers: *mut u64,
    /// The running frame's slots, from `RootSet::top_window`, or null with a
    /// `slot_count` of zero when generated code may not touch them.
    pub(crate) slots: *mut VarSlot,
    pub(crate) slot_count: u64,
    /// `crate::value::small_int_bound` of the `NUMERIC DIGITS` in force.
    pub(crate) small_int_bound: i64,
    /// Non-zero when generated code may open a clause itself: nothing is
    /// queued at a clause boundary and no setting asks for the full one.
    pub(crate) steady: u64,
    /// The interpreter's clause countdown.
    pub(crate) countdown: u64,
    /// The clause generated code entered last and the interpreter has not
    /// been told of: the pc of its `Op::Clause`, from which the host reads its
    /// instruction, line and indent; [`CLAUSE_HEADER`] for the loop's header
    /// clause at a pass boundary of the loop whose record is `k`, as
    /// [`clause_header`] encodes it, whose line, indent and instruction that
    /// record holds; or [`CLAUSE_NONE`].
    pub(crate) clause_at: u64,
    /// Each loop the pass holds, the compiled loop's own first and the inner
    /// loops its body runs after it.
    pub(crate) loops: [LoopRecord; MAX_LOOPS],
    /// Non-zero when `NUMERIC FUZZ` is 0 and `TRACE` shows no results: what
    /// the driver's own quick comparison and quick condition test ask.
    pub(crate) quick_values: u64,
    /// Non-zero when the template engine must emit PARSE value traces.
    pub(crate) parse_traced: u64,
    /// Non-zero when `TRACE` shows intermediates, which every value echo op's
    /// own gate asks (`Interp::tracing_intermediates`).
    pub(crate) intermediates: u64,
    /// Pass boundaries generated code has reached, the compiled loop's and
    /// its inner loops', which the host weighs the ops it ran against.
    pub(crate) passes: u64,
    /// The op a [`super::JUMP`] answer goes on at.
    pub(crate) jump_to: u64,
    /// Non-zero when generated code opened the running clause and the host
    /// has not taken it over, which is what makes its close free. The host's
    /// helpers clear it when they take the clause, and set it when an op
    /// hands the clause back.
    pub(crate) opened: u64,
}

impl Native {
    pub(crate) fn new(registers: *mut u64) -> Native {
        Native {
            registers,
            slots: ptr::null_mut(),
            slot_count: 0,
            small_int_bound: 0,
            steady: 0,
            countdown: 0,
            clause_at: CLAUSE_NONE,
            loops: [LoopRecord::default(); MAX_LOOPS],
            quick_values: 0,
            parse_traced: 0,
            intermediates: 0,
            passes: 0,
            jump_to: 0,
            opened: 0,
        }
    }
}

/// One loop's quick pass, for generated code to take its pass boundary
/// itself.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct LoopRecord {
    /// Non-zero when generated code may take the loop's next pass itself,
    /// which is `run::QuickPass`'s: the control variable in slot `slot`
    /// stepped by `step` and compared with `to`, in a header clause at
    /// `line` that leaves the value indent at `indent`.
    pub(crate) ready: u64,
    /// Which step: [`LOOP_CONTROLLED`], [`LOOP_COUNT`] (with the passes left in
    /// `value`) or [`LOOP_FOREVER`].
    pub(crate) kind: u64,
    pub(crate) slot: u64,
    pub(crate) step: i64,
    pub(crate) to: i64,
    pub(crate) line: u64,
    pub(crate) indent: u64,
    /// The header clause's instruction, the loop's `END`, which the host
    /// alone reads.
    pub(crate) end: u64,
    /// The control value generated code last wrote, which `dirty` says the
    /// loop's own state has not been given.
    pub(crate) value: i64,
    pub(crate) dirty: u64,
    /// Remaining passes of a controlled loop with `FOR`.
    pub(crate) for_remaining: u64,
}

/// How many loops a compiled pass holds at most: its own and the inner ones.
pub(crate) const MAX_LOOPS: usize = 4;

/// [`Native::clause_at`] for the header clause of the loop whose record is
/// `k`.
pub(crate) const fn clause_header(k: usize) -> u64 {
    u64::MAX - 1 - interrexx_cast::u64_from_usize(k)
}

/// The offset of `field` of record `k` in [`Native`].
pub(crate) const fn loop_field(k: usize, field: usize) -> i32 {
    assert!(k < MAX_LOOPS);
    at(offset_of!(Native, loops) + k * size_of::<LoopRecord>() + field)
}

/// An offset into [`Native`], as the code generator takes it.
const fn at(offset: usize) -> i32 {
    let wide = interrexx_cast::u64_from_usize(offset);
    assert!(wide < interrexx_cast::u64_from_u32(i32::MAX.cast_unsigned()));
    // Below `i32::MAX`, so its low four bytes are the whole of it.
    let [a, b, c, d, ..] = wide.to_le_bytes();
    i32::from_le_bytes([a, b, c, d])
}

pub(crate) const REGISTERS: i32 = at(offset_of!(Native, registers));
pub(crate) const SLOTS: i32 = at(offset_of!(Native, slots));
pub(crate) const SLOT_COUNT: i32 = at(offset_of!(Native, slot_count));
pub(crate) const SMALL_INT_BOUND: i32 = at(offset_of!(Native, small_int_bound));
pub(crate) const STEADY: i32 = at(offset_of!(Native, steady));
pub(crate) const COUNTDOWN: i32 = at(offset_of!(Native, countdown));
pub(crate) const CLAUSE_AT: i32 = at(offset_of!(Native, clause_at));
/// [`Native::clause_at`] with no clause to tell the interpreter of.
pub(crate) const CLAUSE_NONE: u64 = u64::MAX;
pub(crate) const LOOP_READY: usize = offset_of!(LoopRecord, ready);
pub(crate) const LOOP_KIND: usize = offset_of!(LoopRecord, kind);
pub(crate) const LOOP_CONTROLLED: u64 = 0;
pub(crate) const LOOP_COUNT: u64 = 1;
pub(crate) const LOOP_FOREVER: u64 = 2;
pub(crate) const LOOP_SLOT: usize = offset_of!(LoopRecord, slot);
pub(crate) const LOOP_STEP: usize = offset_of!(LoopRecord, step);
pub(crate) const LOOP_TO: usize = offset_of!(LoopRecord, to);
pub(crate) const LOOP_VALUE: usize = offset_of!(LoopRecord, value);
pub(crate) const LOOP_DIRTY: usize = offset_of!(LoopRecord, dirty);
pub(crate) const LOOP_FOR_REMAINING: usize = offset_of!(LoopRecord, for_remaining);
pub(crate) const QUICK_VALUES: i32 = at(offset_of!(Native, quick_values));
pub(crate) const PARSE_TRACED: i32 = at(offset_of!(Native, parse_traced));
pub(crate) const INTERMEDIATES: i32 = at(offset_of!(Native, intermediates));
pub(crate) const PASSES: i32 = at(offset_of!(Native, passes));
pub(crate) const JUMP_TO: i32 = at(offset_of!(Native, jump_to));
pub(crate) const OPENED: i32 = at(offset_of!(Native, opened));

/// A variable slot's size, and where its two words are.
pub(crate) const SLOT_SIZE: i64 =
    interrexx_cast::u64_from_usize(size_of::<VarSlot>()).cast_signed();
pub(crate) const SLOT_SET_AT: i32 = at(VarSlot::SET_AT);
pub(crate) const SLOT_VALUE_AT: i32 = at(VarSlot::VALUE_AT);

/// The small-integer encoding, read off `ObjRef::small_int` itself: the tag
/// is what zero encodes to, and the shift is how far one moves from it.
const fn small(value: i64) -> u64 {
    match ObjRef::small_int(value) {
        Some(handle) => handle.bits(),
        None => panic!("0 and 1 are small integers"),
    }
}
pub(crate) const TAG_INT: i64 = small(0).cast_signed();
pub(crate) const TAG_BITS: i64 =
    interrexx_cast::i64_from_u32((small(1) - small(0)).trailing_zeros());
pub(crate) const TAG_MASK: i64 = (1 << TAG_BITS) - 1;
/// What a comparison answers, and the small integers a condition also
/// accepts as logical (`crate::eval::truth_without_conversion`).
pub(crate) const TRUE_BITS: i64 = crate::eval::LOGICAL_TRUE.bits().cast_signed();
pub(crate) const FALSE_BITS: i64 = crate::eval::LOGICAL_FALSE.bits().cast_signed();
pub(crate) const ONE_BITS: i64 = small(1).cast_signed();
pub(crate) const ZERO_BITS: i64 = small(0).cast_signed();
/// What a cleared register holds.
pub(crate) const NIL_BITS: u64 = ObjRef::NIL.bits();

#[cfg(test)]
mod tests {
    use super::*;

    /// The encoding generated code decodes and builds is `ObjRef::small_int`'s
    /// at the edges of the range it is used on.
    #[test]
    fn the_small_integer_encoding_is_the_handles_own() {
        for value in [
            0,
            1,
            -1,
            7,
            -8,
            123_456_789,
            interrexx_core::SMALL_INT_MAX - 1,
            interrexx_core::SMALL_INT_MIN + 1,
        ] {
            let bits = ObjRef::small_int(value).expect("in range").bits();
            assert_eq!(
                bits & TAG_MASK.cast_unsigned(),
                TAG_INT.cast_unsigned(),
                "{value}"
            );
            assert_eq!(bits.cast_signed() >> TAG_BITS, value, "{value}");
            assert_eq!(
                ((value << TAG_BITS) | TAG_INT).cast_unsigned(),
                bits,
                "{value}"
            );
        }
        for other in [ObjRef::NIL, ObjRef::inline_byte(b'a')] {
            assert_ne!(
                other.bits() & TAG_MASK.cast_unsigned(),
                TAG_INT.cast_unsigned()
            );
        }
    }
}
