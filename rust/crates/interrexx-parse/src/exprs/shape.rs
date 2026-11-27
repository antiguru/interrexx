//! An arena expression's canonical S-expression rendering, the text golden
//! tests compare, written by loops over the subtree's range rather than by
//! recursion over its children.
//!
//! A node's text is its children's texts with the node's own pieces around
//! them: a head before its first child, a separator before each later child,
//! and a tail after its last. A leaf is a head alone. Three loops render a
//! subtree: a forward loop gives each node its length, which needs its
//! children's lengths first; a reverse loop gives each node its offset, which
//! needs its parent's offset first; a third loop writes each node's own
//! pieces at their offsets into a buffer allocated once, so every output byte
//! is written once.

use std::fmt::Write as _;

use crate::ast::{PrefixOp, Tail, compound_parts};
use crate::token::{Operator, SymbolTable};

use super::{CallTarget, ExprId, Exprs, NodeKind, children};

/// One of a node's own pieces of text.
#[derive(Copy, Clone)]
enum Piece {
    /// Before the first child, or the whole text of a leaf.
    Head,
    /// Before the child at this position, which is not the first.
    Before(usize),
    /// After the last child.
    Tail,
}

/// `bytes` as a quoted, escaped string, so that no two byte strings render
/// alike.
fn quoted(out: &mut String, bytes: &[u8]) {
    write!(out, "{:?}", String::from_utf8_lossy(bytes)).expect("a String accepts every write");
}

impl Exprs {
    /// The canonical rendering of the expression rooted at `root`: an
    /// S-expression in pre-order, with `<omitted>` for an omitted argument or
    /// item and `(vref A)` or `(vref stem:A.)` for a variable reference.
    pub fn shape(&self, root: ExprId, symbols: &SymbolTable) -> String {
        let nodes = self.range(root);
        let first = self.node(root).first.index();
        let mut scratch = String::new();
        let mut inline = [root; 2];

        // Forward: a node's length is its pieces' lengths plus its children's,
        // and every child precedes its parent.
        let mut lengths = vec![0usize; nodes.len()];
        for (at, node) in nodes.iter().enumerate() {
            let kids = self.kids(&node.kind, &mut inline);
            let count = kids.len();
            let mut length = self.piece(&node.kind, Piece::Head, count, symbols, &mut scratch);
            for (position, child) in kids.iter().enumerate() {
                if position > 0 {
                    length += self.piece(
                        &node.kind,
                        Piece::Before(position),
                        count,
                        symbols,
                        &mut scratch,
                    );
                }
                length += lengths[child.index() - first];
            }
            length += self.piece(&node.kind, Piece::Tail, count, symbols, &mut scratch);
            lengths[at] = length;
        }

        // Reverse: a parent, whose offset is known, hands each child its
        // offset in one sweep over its child list, which is in position order.
        let mut offsets = vec![0usize; nodes.len()];
        for (at, node) in nodes.iter().enumerate().rev() {
            let kids = self.kids(&node.kind, &mut inline);
            let count = kids.len();
            let mut cursor =
                offsets[at] + self.piece(&node.kind, Piece::Head, count, symbols, &mut scratch);
            for (position, child) in kids.iter().enumerate() {
                if position > 0 {
                    cursor += self.piece(
                        &node.kind,
                        Piece::Before(position),
                        count,
                        symbols,
                        &mut scratch,
                    );
                }
                offsets[child.index() - first] = cursor;
                cursor += lengths[child.index() - first];
            }
        }

        // Write: each node's head at its offset, each separator just before
        // its child's offset and its tail just before its end, into a buffer
        // of the root's length.
        let mut out = vec![0u8; lengths[nodes.len() - 1]];
        for (at, node) in nodes.iter().enumerate() {
            let kids = self.kids(&node.kind, &mut inline);
            let count = kids.len();
            let length = self.piece(&node.kind, Piece::Head, count, symbols, &mut scratch);
            let start = offsets[at];
            out[start..start + length].copy_from_slice(scratch.as_bytes());
            for (position, child) in kids.iter().enumerate().skip(1) {
                let length = self.piece(
                    &node.kind,
                    Piece::Before(position),
                    count,
                    symbols,
                    &mut scratch,
                );
                let end = offsets[child.index() - first];
                out[end - length..end].copy_from_slice(scratch.as_bytes());
            }
            let length = self.piece(&node.kind, Piece::Tail, count, symbols, &mut scratch);
            let end = offsets[at] + lengths[at];
            out[end - length..end].copy_from_slice(scratch.as_bytes());
        }
        String::from_utf8(out).expect("every piece is a str")
    }

    /// The child ids of `kind`, in position order.
    fn kids<'a>(&'a self, kind: &NodeKind, inline: &'a mut [ExprId; 2]) -> &'a [ExprId] {
        children(&self.edges, kind, inline).expect("a node's child list is in the arena")
    }

    /// Writes `piece` of a node of `kind` with `count` children into
    /// `scratch`, replacing what it held, and answers its length.
    fn piece(
        &self,
        kind: &NodeKind,
        piece: Piece,
        count: usize,
        symbols: &SymbolTable,
        scratch: &mut String,
    ) -> usize {
        scratch.clear();
        match piece {
            Piece::Head => self.head(kind, count, symbols, scratch),
            Piece::Before(position) => match kind {
                // The receiver comes first, then the name, then ` :` and the
                // scope where there is one, then the arguments.
                NodeKind::Message { name, scoped, .. } if position == 1 => {
                    scratch.push(' ');
                    quoted(scratch, name.bytes());
                    scratch.push_str(if *scoped { " :" } else { " " });
                }
                _ => scratch.push(' '),
            },
            Piece::Tail => match kind {
                NodeKind::Message { name, .. } if count == 1 => {
                    scratch.push(' ');
                    quoted(scratch, name.bytes());
                    scratch.push(')');
                }
                NodeKind::Prefix { .. }
                | NodeKind::Binary { .. }
                | NodeKind::Call { .. }
                | NodeKind::QualifiedCall { .. }
                | NodeKind::List(_)
                | NodeKind::Logical(_)
                | NodeKind::Message { .. } => scratch.push(')'),
                NodeKind::Literal(_)
                | NodeKind::Constant(_)
                | NodeKind::Variable(_)
                | NodeKind::Stem(_)
                | NodeKind::Compound(_)
                | NodeKind::DotVariable(_)
                | NodeKind::VariableReference { .. }
                | NodeKind::ClassResolver { .. }
                | NodeKind::Omitted => {}
            },
        }
        scratch.len()
    }

    /// Writes the head of a node of `kind` with `count` children to `out`:
    /// the text before its first child, or the whole text of a leaf.
    fn head(&self, kind: &NodeKind, count: usize, symbols: &SymbolTable, out: &mut String) {
        // The space a list-shaped node puts before its first child.
        let lead = if count > 0 { " " } else { "" };
        match kind {
            NodeKind::Literal(bytes) => quoted(out, self.bytes(*bytes)),
            NodeKind::Constant(id) | NodeKind::Variable(id) => out.push_str(symbols.name(*id)),
            NodeKind::Stem(id) => {
                out.push_str("stem:");
                out.push_str(symbols.name(*id));
            }
            NodeKind::Compound(id) => {
                let (stem, tails) = compound_parts(symbols.name(*id));
                out.push_str("compound:");
                out.push_str(stem);
                out.push('[');
                for (at, tail) in tails.iter().enumerate() {
                    if at > 0 {
                        out.push(',');
                    }
                    match tail {
                        Tail::Constant(text) => {
                            out.push_str("const:");
                            out.push_str(text);
                        }
                        Tail::Variable(text) => {
                            out.push_str("var:");
                            out.push_str(text);
                        }
                    }
                }
                out.push(']');
            }
            NodeKind::DotVariable(id) => {
                out.push_str("env:");
                out.push_str(symbols.name(*id));
            }
            NodeKind::VariableReference { symbol, stem } => {
                out.push_str(if *stem { "(vref stem:" } else { "(vref " });
                out.push_str(symbols.name(*symbol));
                out.push(')');
            }
            NodeKind::ClassResolver { namespace, name } => {
                out.push_str("(class ");
                out.push_str(symbols.name(*namespace));
                out.push(':');
                out.push_str(symbols.name(*name));
                out.push(')');
            }
            NodeKind::Omitted => out.push_str("<omitted>"),
            NodeKind::Prefix { op, .. } => out.push_str(match op {
                PrefixOp::Plus => "(u+ ",
                PrefixOp::Minus => "(u- ",
                PrefixOp::Not => "(u\\ ",
            }),
            NodeKind::Binary { op, .. } => {
                out.push('(');
                out.push_str(match op {
                    Operator::Abuttal => "abut",
                    Operator::Blank => "blank",
                    other => other.spelling(),
                });
                out.push(' ');
            }
            NodeKind::Call { target, .. } => {
                out.push_str("(call ");
                match target {
                    CallTarget::Symbol(id) => out.push_str(symbols.name(*id)),
                    CallTarget::Literal(bytes) => quoted(out, self.bytes(*bytes)),
                }
                out.push_str(lead);
            }
            NodeKind::QualifiedCall {
                namespace, name, ..
            } => {
                out.push_str("(qcall ");
                out.push_str(symbols.name(*namespace));
                out.push(':');
                out.push_str(symbols.name(*name));
                out.push_str(lead);
            }
            NodeKind::List(_) => {
                out.push_str("(list");
                out.push_str(lead);
            }
            NodeKind::Logical(_) => {
                out.push_str("(logical");
                out.push_str(lead);
            }
            NodeKind::Message { cascade, .. } => {
                out.push_str(if *cascade { "(msg~~ " } else { "(msg~ " });
            }
        }
    }
}
