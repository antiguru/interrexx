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
    /// The clause generated code opened last: its instruction, line and
    /// indent, which [`Native::clause_dirty`] says the interpreter has not
    /// been given yet.
    pub(crate) clause_index: u64,
    pub(crate) clause_line: u64,
    pub(crate) clause_indent: u64,
    pub(crate) clause_dirty: u64,
    /// What a clause's own indent is offset by in the running activation.
    pub(crate) indent_base: u64,
    /// Non-zero when generated code may take the loop's next pass itself,
    /// which is `run::QuickPass`'s: the control variable in slot `loop_slot`
    /// stepped by `loop_step` and compared with `loop_to`, in a header clause
    /// at `loop_line` that leaves the value indent at `loop_indent`.
    pub(crate) loop_ready: u64,
    pub(crate) loop_slot: u64,
    pub(crate) loop_step: i64,
    pub(crate) loop_to: i64,
    pub(crate) loop_line: u64,
    pub(crate) loop_indent: u64,
    /// The control value generated code last wrote, which
    /// [`Native::loop_dirty`] says the loop's own state has not been given.
    pub(crate) loop_value: i64,
    pub(crate) loop_dirty: u64,
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
            clause_index: 0,
            clause_line: 0,
            clause_indent: 0,
            clause_dirty: 0,
            indent_base: 0,
            loop_ready: 0,
            loop_slot: 0,
            loop_step: 0,
            loop_to: 0,
            loop_line: 0,
            loop_indent: 0,
            loop_value: 0,
            loop_dirty: 0,
        }
    }
}

/// An offset into [`Native`], as the code generator takes it.
const fn at(offset: usize) -> i32 {
    assert!(offset < i32::MAX as usize);
    offset as i32
}

pub(crate) const REGISTERS: i32 = at(offset_of!(Native, registers));
pub(crate) const SLOTS: i32 = at(offset_of!(Native, slots));
pub(crate) const SLOT_COUNT: i32 = at(offset_of!(Native, slot_count));
pub(crate) const SMALL_INT_BOUND: i32 = at(offset_of!(Native, small_int_bound));
pub(crate) const STEADY: i32 = at(offset_of!(Native, steady));
pub(crate) const COUNTDOWN: i32 = at(offset_of!(Native, countdown));
pub(crate) const CLAUSE_INDEX: i32 = at(offset_of!(Native, clause_index));
pub(crate) const CLAUSE_LINE: i32 = at(offset_of!(Native, clause_line));
pub(crate) const CLAUSE_INDENT: i32 = at(offset_of!(Native, clause_indent));
pub(crate) const CLAUSE_DIRTY: i32 = at(offset_of!(Native, clause_dirty));
pub(crate) const INDENT_BASE: i32 = at(offset_of!(Native, indent_base));
pub(crate) const LOOP_READY: i32 = at(offset_of!(Native, loop_ready));
pub(crate) const LOOP_SLOT: i32 = at(offset_of!(Native, loop_slot));
pub(crate) const LOOP_STEP: i32 = at(offset_of!(Native, loop_step));
pub(crate) const LOOP_TO: i32 = at(offset_of!(Native, loop_to));
pub(crate) const LOOP_LINE: i32 = at(offset_of!(Native, loop_line));
pub(crate) const LOOP_INDENT: i32 = at(offset_of!(Native, loop_indent));
pub(crate) const LOOP_VALUE: i32 = at(offset_of!(Native, loop_value));
pub(crate) const LOOP_DIRTY: i32 = at(offset_of!(Native, loop_dirty));

/// A variable slot's size, and where its two words are.
pub(crate) const SLOT_SIZE: i64 = size_of::<VarSlot>() as i64;
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
pub(crate) const TAG_INT: i64 = small(0) as i64;
pub(crate) const TAG_BITS: i64 = (small(1) - small(0)).trailing_zeros() as i64;
pub(crate) const TAG_MASK: i64 = (1 << TAG_BITS) - 1;
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
            assert_eq!(bits & TAG_MASK as u64, TAG_INT as u64, "{value}");
            assert_eq!((bits as i64) >> TAG_BITS, value, "{value}");
            assert_eq!(((value << TAG_BITS) | TAG_INT) as u64, bits, "{value}");
        }
        for other in [ObjRef::NIL, ObjRef::inline_byte(b'a')] {
            assert_ne!(other.bits() & TAG_MASK as u64, TAG_INT as u64);
        }
    }

}
