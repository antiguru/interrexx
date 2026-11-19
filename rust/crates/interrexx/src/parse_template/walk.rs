//! The template walk as the compiled stream drives it. [`Op::ParseBegin`]
//! takes the first parse string and walks the entries up to the first whose
//! operand the stream evaluates, and each [`Op::ParseRun`] walks one such
//! entry, with its operand from a register, and the entries after it up to
//! the next. Inside one op the walk is the template engine's own loop, with
//! the cursor in locals; between two ops it lives in registers of the
//! clause's own frame, so it is rooted, it is the frame's to move and to drop,
//! and a nested activation that parses has a walk of its own.
//!
//! [`Op::ParseBegin`]: crate::ir::Op::ParseBegin
//! [`Op::ParseRun`]: crate::ir::Op::ParseRun

use super::*;
use crate::ir::ParseOpening;
use interrexx_core::RegFrame;
use interrexx_parse::Instruction;
use interrexx_parse::InstructionKind;

/// How many registers one compiled walk keeps, from its base.
pub(crate) const WALK_REGISTERS: u16 = 8;

/// The parse string: a string that is its own string value.
const SOURCE: u16 = 0;
/// The number the parse string renders, which a piece covering the whole
/// string assigns, or `.nil`.
const ORIGIN: u16 = 1;
/// [`Cursor`]'s five positions, as small integers.
const START: u16 = 2;
const END: u16 = 3;
const PATTERN_START: u16 = 4;
const PATTERN_END: u16 = 5;
const SUBCURRENT: u16 = 6;
/// `PARSE ARG`: the argument the next template reads.
const NEXT_ARGUMENT: u16 = 7;

/// A position or count as the small integer a register holds. A string is
/// far shorter than the range a small integer covers.
fn position(at: usize) -> ObjRef {
    i64::try_from(at)
        .ok()
        .and_then(ObjRef::small_int)
        .unwrap_or(ObjRef::NIL)
}

/// The position a register filed by [`position`] holds.
fn at(registers: RegFrame<'_>, base: u16, offset: u16) -> usize {
    match registers.get(base + offset).decode() {
        Decoded::SmallInt(at) => usize::try_from(at).unwrap_or(0),
        _ => 0,
    }
}

/// The `PARSE` a walk op's clause is.
fn parse_of(clause: &Instruction) -> Result<&Parse, Failure> {
    match &clause.kind {
        InstructionKind::Parse(parse)
        | InstructionKind::Arg(parse)
        | InstructionKind::Pull(parse) => Ok(parse),
        kind => Err(Loud::instruction(kind).into()),
    }
}

/// One walk in hand: the parse string, the cursor over it, and for `PARSE
/// ARG` the argument the next template reads.
struct Walk {
    value: ObjRef,
    source: SourceText,
    cursor: Cursor,
    next_argument: usize,
}

impl Interp {
    /// `value` as a parse string read in place, and the number a whole-string
    /// piece assigns: what [`Interp::source_text`] answers for a string that
    /// is its own string value and needs no rendering, or `None` for any
    /// other value. Asking sends nothing and changes nothing, so a `None` can
    /// still hand the clause to [`Interp::exec_parse`].
    fn plain_parse_source(&self, value: ObjRef) -> Option<(SourceText, Option<ObjRef>)> {
        match value.decode() {
            Decoded::Text(inline) => Some((SourceText::Inline(inline), None)),
            Decoded::SmallInt(number) => crate::value::small_int_inline_text(number)
                .map(|inline| (SourceText::Inline(inline), Some(value))),
            Decoded::Heap { .. } => self.heap.body_text(value).map(|bytes| {
                (
                    SourceText::Text {
                        value,
                        length: bytes.len(),
                    },
                    None,
                )
            }),
            _ => None,
        }
    }

    /// A walk over `value`, which [`Interp::plain_parse_source`] took, from
    /// its origin.
    fn walk_over(&self, value: ObjRef, next_argument: usize) -> Walk {
        let (source, origin) = self
            .plain_parse_source(value)
            .unwrap_or((SourceText::Owned(Vec::new()), None));
        let mut cursor = Cursor::new(source.len());
        cursor.origin = origin;
        Walk {
            value,
            source,
            cursor,
            next_argument,
        }
    }

    /// The argument `PARSE ARG` reads for template `at`, as a parse string:
    /// an omitted or absent one is the null string.
    fn argument_parse_value(&self, at: usize) -> ObjRef {
        match self.activity.call_context.arguments.get(at) {
            Some(Some(argument)) => *argument,
            Some(None) | None => ObjRef::inline_text(b"").unwrap_or(ObjRef::NIL),
        }
    }

    /// Files `walk` in the registers from `base`, for the op that goes on
    /// with it.
    fn file_walk(registers: RegFrame<'_>, base: u16, walk: &Walk) {
        let cursor = &walk.cursor;
        registers.set(base + SOURCE, walk.value);
        registers.set(base + ORIGIN, cursor.origin.unwrap_or(ObjRef::NIL));
        registers.set(base + START, position(cursor.start));
        registers.set(base + END, position(cursor.end));
        registers.set(base + PATTERN_START, position(cursor.pattern_start));
        registers.set(base + PATTERN_END, position(cursor.pattern_end));
        registers.set(base + SUBCURRENT, position(cursor.subcurrent));
        registers.set(base + NEXT_ARGUMENT, position(walk.next_argument));
    }

    /// The walk [`Interp::file_walk`] filed from `base`.
    fn filed_walk(&self, registers: RegFrame<'_>, base: u16) -> Walk {
        let value = registers.get(base + SOURCE);
        let mut walk = self.walk_over(value, at(registers, base, NEXT_ARGUMENT));
        let origin = registers.get(base + ORIGIN);
        walk.cursor.origin = (origin != ObjRef::NIL).then_some(origin);
        walk.cursor.start = at(registers, base, START);
        walk.cursor.end = at(registers, base, END);
        walk.cursor.pattern_start = at(registers, base, PATTERN_START);
        walk.cursor.pattern_end = at(registers, base, PATTERN_END);
        walk.cursor.subcurrent = at(registers, base, SUBCURRENT);
        walk
    }

    /// Template entries `from..to` of `parse` applied to `walk`, the first
    /// with `operand` where the stream evaluated it: a trigger and then its
    /// targets, or a comma, which steps to the next parse string
    /// (`r[parse.multiple.comma]`).
    fn walk_entries(
        &mut self,
        code: &Code<'_>,
        parse: &Parse,
        from: usize,
        to: usize,
        mut operand: Option<ObjRef>,
        walk: &mut Walk,
    ) -> Result<(), Failure> {
        let indent = self.activity.clause_state.current_value_indent;
        for entry in parse.template.get(from..to).unwrap_or(&[]) {
            let Some(trigger) = entry else {
                // The next argument for `PARSE ARG`, and the null string for
                // every other source, parsed from its first character.
                let next = match parse.source {
                    ParseSource::Arg => {
                        walk.next_argument += 1;
                        self.argument_parse_value(walk.next_argument - 1)
                    }
                    _ => ObjRef::inline_text(b"").unwrap_or(ObjRef::NIL),
                };
                *walk = self.walk_over(next, walk.next_argument);
                continue;
            };
            match operand.take() {
                Some(value) => self.apply_trigger_value(
                    trigger,
                    value,
                    &walk.source,
                    &mut walk.cursor,
                    indent,
                )?,
                None => {
                    self.apply_trigger(code, trigger, &walk.source, &mut walk.cursor, indent)?;
                }
            }
            self.assign_targets(code, trigger, &walk.source, &mut walk.cursor, indent)?;
        }
        Ok(())
    }

    /// [`crate::ir::Op::ParseBegin`]: takes the first parse string of the
    /// clause and walks its entries up to `to`, and answers whether the
    /// compiled walk took the clause. It does not where a `>K>`, `>>>` or
    /// `>=>` line will be traced, under a fold, for a source read as a line
    /// or built by the interpreter, for `PARSE VAR` into words, or where a
    /// parse string is not a string that is its own string value -- a number
    /// to render, an object to ask, a stem or a compound to read -- and
    /// [`Interp::exec_parse`] then runs the clause whole, with nothing done
    /// that it would do again.
    #[inline(never)]
    #[expect(
        clippy::too_many_arguments,
        reason = "the op's own fields, beside the clause and the frame they index"
    )]
    pub(crate) fn parse_begin(
        &mut self,
        code: &Code<'_>,
        clause: &Instruction,
        src: u16,
        base: u16,
        to: u16,
        opening: ParseOpening,
        strings: u8,
        registers: RegFrame<'_>,
    ) -> Result<bool, Failure> {
        let parse = parse_of(clause)?;
        let evaluated = (src != crate::ir::Op::ARG_OMITTED).then(|| registers.get(src));
        let Some((first, source, origin)) =
            self.walked_source(code, parse, evaluated, opening, strings)?
        else {
            self.exec_parse(code, parse, evaluated)?;
            return Ok(false);
        };
        // Read once, here, before any target is assigned, so a target that is
        // the source variable does not change what is parsed
        // (`r[parse.index.order]`). Rooted for the clause, because that
        // target is then all that held it, and the pieces built after it
        // are allocations.
        self.roots.activity_mut().push_temp(first);
        let mut cursor = Cursor::new(source.len());
        cursor.origin = origin;
        let mut walk = Walk {
            value: first,
            source,
            cursor,
            next_argument: 1,
        };
        let to = usize::from(to);
        self.walk_entries(code, parse, 0, to, None, &mut walk)?;
        if to < parse.template.len() && base != crate::ir::Op::ARG_OMITTED {
            Self::file_walk(registers, base, &walk);
        }
        Ok(true)
    }

    /// The first parse string the compiled walk takes, read in place, and
    /// the number a whole-string piece assigns, or `None` for a clause it
    /// hands to the template engine: one the compiler handed it, one that
    /// will trace a line, and one whose parse string is not its own string
    /// value.
    #[allow(
        clippy::type_complexity,
        reason = "the value, its bytes and its origin, once"
    )]
    fn walked_source(
        &mut self,
        code: &Code<'_>,
        parse: &Parse,
        evaluated: Option<ObjRef>,
        opening: ParseOpening,
        strings: u8,
    ) -> Result<Option<(ObjRef, SourceText, Option<ObjRef>)>, Failure> {
        if opening == ParseOpening::Engine {
            return Ok(None);
        }
        let traced = self.traced_mode();
        if traced.results || traced.intermediates {
            return Ok(None);
        }
        let first = match (opening, &parse.source) {
            (ParseOpening::Value, _) => {
                evaluated.unwrap_or_else(|| ObjRef::inline_text(b"").unwrap_or(ObjRef::NIL))
            }
            // A simple name, which the compiler judged: its read is the
            // variable's value and `NOVALUE` for an unset one.
            (ParseOpening::Var, ParseSource::Var(id)) => {
                let (value, novalue) = self.read(code, *id);
                self.novalue_check(novalue, value)?;
                value
            }
            // Every argument a template of this clause will read, judged
            // before the first is taken, since a later one that needs
            // rendering could no longer hand the clause back.
            (ParseOpening::Arg, _) => {
                if (0..usize::from(strings)).any(|at| {
                    self.plain_parse_source(self.argument_parse_value(at))
                        .is_none()
                }) {
                    return Ok(None);
                }
                self.argument_parse_value(0)
            }
            _ => return Ok(None),
        };
        Ok(self
            .plain_parse_source(first)
            .map(|(source, origin)| (first, source, origin)))
    }

    /// [`crate::ir::Op::ParseRun`]: template entries `from..to` of the
    /// clause applied to the walk filed from `base`, entry `from` with its
    /// operand in register `operand`.
    #[inline(never)]
    #[expect(
        clippy::too_many_arguments,
        reason = "the op's own fields, beside the clause and the frame they index"
    )]
    pub(crate) fn parse_run(
        &mut self,
        code: &Code<'_>,
        clause: &Instruction,
        base: u16,
        from: u16,
        to: u16,
        operand: u16,
        registers: RegFrame<'_>,
    ) -> Result<(), Failure> {
        let parse = parse_of(clause)?;
        let mut walk = self.filed_walk(registers, base);
        let value = registers.get(operand);
        let to = usize::from(to);
        self.walk_entries(code, parse, usize::from(from), to, Some(value), &mut walk)?;
        if to < parse.template.len() {
            Self::file_walk(registers, base, &walk);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
