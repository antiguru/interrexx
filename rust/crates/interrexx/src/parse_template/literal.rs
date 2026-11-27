use super::*;
use crate::plan::Plan;
use interrexx_parse::SymbolTable;

/// The slot writes for a literal source and a template whose positions and
/// target values are known independently of the running activation.
pub(crate) fn literal_writes(
    parse: &Parse,
    plan: &Plan,
    symbols: &SymbolTable,
) -> Option<Vec<(u32, ObjRef)>> {
    let ParseSource::Value(Some(expression)) = &parse.source else {
        return None;
    };
    let ExprKind::Literal(bytes) = &expression.kind else {
        return None;
    };
    if parse.upper || parse.lower || crate::value::canonical_small_int(bytes).is_some() {
        return None;
    }
    let mut cursor = Cursor::new(bytes.len());
    let mut writes = Vec::new();
    for trigger in &parse.template {
        let trigger = trigger.as_ref()?;
        if trigger.kind == TriggerKind::End {
            cursor.move_to_end();
        } else {
            let operand = match &trigger.value.as_ref()?.kind {
                ExprKind::Literal(operand) => operand,
                ExprKind::Constant(id) => symbols.name(*id).as_bytes(),
                _ => return None,
            };
            match trigger.kind {
                TriggerKind::String => cursor.search(bytes, operand),
                TriggerKind::Mixed => cursor.caseless_search(bytes, operand),
                kind => {
                    // Every one-digit position fits even NUMERIC DIGITS 1
                    // (r[parse.patterns.number]). Wider operands are checked
                    // against the settings in force when the clause runs.
                    let &[digit] = operand else { return None };
                    if !digit.is_ascii_digit() {
                        return None;
                    }
                    let offset = usize::from(digit - b'0');
                    match kind {
                        TriggerKind::Plus => cursor.forward(offset),
                        TriggerKind::Minus => cursor.backward(offset),
                        TriggerKind::Absolute => cursor.absolute(offset),
                        TriggerKind::PlusLength => cursor.forward_length(offset),
                        TriggerKind::MinusLength => cursor.backward_length(offset),
                        _ => return None,
                    }
                }
            }
        }
        for (index, target) in trigger.targets.iter().enumerate() {
            let piece = if index + 1 == trigger.targets.len() {
                cursor.remainder()
            } else {
                cursor.next_word(bytes)
            };
            let Some(target) = target else { continue };
            let ExprKind::Variable(id) = target.kind else {
                return None;
            };
            let slot = u32::try_from(plan.slot_for_symbol(id)?).ok()?;
            // Inline pieces can live in the plan without adding GC roots.
            writes.push((slot, ObjRef::inline_text(&bytes[piece])?));
        }
    }
    (!writes.is_empty()).then_some(writes)
}
