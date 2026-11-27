use super::*;
use crate::plan::Plan;
use interrexx_parse::{CodeBody, ExprView, SymbolTable};

/// The slot writes for a literal source and a template whose positions and
/// target values are known independently of the running activation.
pub(crate) fn literal_writes(
    body: &CodeBody,
    parse: &Parse,
    plan: &Plan,
    symbols: &SymbolTable,
) -> Option<Vec<(u32, ObjRef)>> {
    let ParseSource::Value(Some(expression)) = &parse.source else {
        return None;
    };
    let ExprView::Literal(bytes) = body.expr(*expression).kind() else {
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
            let operand = match body.expr(trigger.value?).kind() {
                ExprView::Literal(operand) => operand,
                ExprView::Constant(id) => symbols.name(id).as_bytes(),
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
            let id = body.expr(*target).variable()?;
            let slot = u32::try_from(plan.slot_for_symbol(id)?).ok()?;
            // Inline pieces can live in the plan without adding GC roots.
            writes.push((slot, ObjRef::inline_text(&bytes[piece])?));
        }
    }
    (!writes.is_empty()).then_some(writes)
}
