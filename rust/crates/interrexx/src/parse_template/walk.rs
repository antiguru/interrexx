//! The template walk as the compiled stream drives it: [`Op::ParseBegin`]
//! takes the first parse string, and one [`Op::ParseStep`] per template entry
//! applies that entry. Between them the walk lives in registers of the
//! clause's own frame, so it is rooted, it is the frame's to move and to drop,
//! and a nested activation that parses has a walk of its own. The steps are
//! the template engine's own, [`Interp::apply_trigger`] and
//! [`Interp::assign_targets`]; what the compiled form changes is who walks
//! the template and evaluates its operands.
//!
//! [`Op::ParseBegin`]: crate::ir::Op::ParseBegin
//! [`Op::ParseStep`]: crate::ir::Op::ParseStep

use super::*;
use interrexx_core::RegFrame;

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

    /// The parse string a walk filed from `base` reads.
    fn walk_source(&self, registers: RegFrame<'_>, base: u16) -> SourceText {
        self.plain_parse_source(registers.get(base + SOURCE))
            .map_or(SourceText::Owned(Vec::new()), |(source, _)| source)
    }

    /// Files `value`, which [`Interp::plain_parse_source`] took, as the walk's
    /// parse string, with a cursor at its origin.
    fn file_parse_string(&self, registers: RegFrame<'_>, base: u16, value: ObjRef) {
        let origin = self
            .plain_parse_source(value)
            .and_then(|(_, origin)| origin);
        registers.set(base + SOURCE, value);
        registers.set(base + ORIGIN, origin.unwrap_or(ObjRef::NIL));
        for offset in [START, END, PATTERN_START, PATTERN_END, SUBCURRENT] {
            registers.set(base + offset, position(0));
        }
    }

    /// The cursor a walk filed from `base` stands at over `source`.
    fn walk_cursor(registers: RegFrame<'_>, base: u16, source: &SourceText) -> Cursor {
        let origin = registers.get(base + ORIGIN);
        Cursor {
            length: source.len(),
            start: at(registers, base, START),
            end: at(registers, base, END),
            pattern_start: at(registers, base, PATTERN_START),
            pattern_end: at(registers, base, PATTERN_END),
            subcurrent: at(registers, base, SUBCURRENT),
            origin: (origin != ObjRef::NIL).then_some(origin),
        }
    }

    fn file_cursor(registers: RegFrame<'_>, base: u16, cursor: &Cursor) {
        registers.set(base + START, position(cursor.start));
        registers.set(base + END, position(cursor.end));
        registers.set(base + PATTERN_START, position(cursor.pattern_start));
        registers.set(base + PATTERN_END, position(cursor.pattern_end));
        registers.set(base + SUBCURRENT, position(cursor.subcurrent));
    }

    /// The argument `PARSE ARG` reads for template `at`, as a parse string:
    /// an omitted or absent one is the null string.
    fn argument_parse_value(&self, at: usize) -> ObjRef {
        match self.activity.call_context.arguments.get(at) {
            Some(Some(argument)) => *argument,
            Some(None) | None => ObjRef::inline_text(b"").unwrap_or(ObjRef::NIL),
        }
    }

    /// [`crate::ir::Op::ParseBegin`]: files the first parse string of `parse`
    /// in the registers from `base`, and answers whether the compiled walk
    /// takes the clause. It does not where a `>K>`, `>>>` or `>=>` line will
    /// be traced, or where a parse string is not a string that is its own
    /// string value -- a number to render or an object to ask -- and nothing
    /// has been done then that [`Interp::exec_parse`] would do again.
    pub(crate) fn parse_begin(
        &mut self,
        code: &Code<'_>,
        parse: &Parse,
        evaluated: Option<ObjRef>,
        registers: RegFrame<'_>,
        base: u16,
    ) -> Result<bool, Failure> {
        let traced = self.traced_mode();
        if traced.results || traced.intermediates || parse.upper || parse.lower {
            return Ok(false);
        }
        let first = match &parse.source {
            ParseSource::Value(None) => ObjRef::inline_text(b"").unwrap_or(ObjRef::NIL),
            ParseSource::Value(Some(_)) => match evaluated {
                Some(value) => value,
                None => return Ok(false),
            },
            // A simple name: its read is the variable's value and `NOVALUE`
            // for an unset one. A stem or a compound is read by the engine's
            // own `Interp::read_parse_var`.
            ParseSource::Var(id) => {
                let name = code.symbols.name(*id).as_bytes();
                if !matches!(crate::run::shape_of(name), crate::run::NameShape::Simple) {
                    return Ok(false);
                }
                let (value, novalue) = self.read(code, *id);
                self.novalue_check(novalue, value)?;
                value
            }
            // Every argument a template of this clause will read, judged
            // before the first is taken, since a later one that needs
            // rendering could no longer hand the clause back.
            ParseSource::Arg => {
                let strings = parse
                    .template
                    .iter()
                    .filter(|entry| entry.is_none())
                    .count()
                    + 1;
                if (0..strings).any(|at| {
                    self.plain_parse_source(self.argument_parse_value(at))
                        .is_none()
                }) {
                    return Ok(false);
                }
                registers.set(base + NEXT_ARGUMENT, position(1));
                self.argument_parse_value(0)
            }
            ParseSource::LineIn
            | ParseSource::Pull
            | ParseSource::Source
            | ParseSource::Version => return Ok(false),
        };
        if self.plain_parse_source(first).is_none() {
            return Ok(false);
        }
        // Read once, here, before any target is assigned, so a target that is
        // the source variable does not change what is parsed
        // (`r[parse.index.order]`).
        self.file_parse_string(registers, base, first);
        Ok(true)
    }

    /// [`crate::ir::Op::ParseStep`]: template entry `entry` of `parse`,
    /// applied to the walk filed from `base`, with the trigger's operand
    /// already evaluated where `operand` holds it.
    pub(crate) fn parse_step(
        &mut self,
        code: &Code<'_>,
        parse: &Parse,
        entry: usize,
        operand: Option<ObjRef>,
        registers: RegFrame<'_>,
        base: u16,
    ) -> Result<(), Failure> {
        let Some(entry) = parse.template.get(entry) else {
            return Err(Loud::parse_trigger_operand().into());
        };
        let Some(trigger) = entry else {
            // The comma fence: the next argument for `PARSE ARG`, and the
            // null string for every other source, parsed from its first
            // character (`r[parse.multiple.comma]`).
            let next = match parse.source {
                ParseSource::Arg => {
                    let at = at(registers, base, NEXT_ARGUMENT);
                    registers.set(base + NEXT_ARGUMENT, position(at + 1));
                    self.argument_parse_value(at)
                }
                _ => ObjRef::inline_text(b"").unwrap_or(ObjRef::NIL),
            };
            self.file_parse_string(registers, base, next);
            return Ok(());
        };
        let indent = self.activity.clause_state.current_value_indent;
        let source = self.walk_source(registers, base);
        let mut cursor = Self::walk_cursor(registers, base, &source);
        match operand {
            Some(value) => {
                self.apply_trigger_value(trigger, value, &source, &mut cursor, indent)?
            }
            None => self.apply_trigger(code, trigger, &source, &mut cursor, indent)?,
        }
        self.assign_targets(code, trigger, &source, &mut cursor, indent)?;
        Self::file_cursor(registers, base, &cursor);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
