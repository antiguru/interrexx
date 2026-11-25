//! A borrowed view of one expression node of an arena.
//!
//! Every method reads one node: a child is answered as another view, never
//! visited, so the view itself never walks.

use std::ops::Range;

use crate::ast::PrefixOp;
use crate::selector::Selector;
use crate::token::{Operator, SymbolId};

use super::{CallTarget, EdgeRange, ExprId, Exprs, NodeKind};

/// One expression node, borrowed.
#[derive(Copy, Clone)]
pub struct ExprRef<'a> {
    exprs: &'a Exprs,
    id: ExprId,
}

/// What a node is, with its payload and its children as views.
#[derive(Copy, Clone, Debug)]
pub enum ExprView<'a> {
    Literal(&'a [u8]),
    Constant(SymbolId),
    Variable(SymbolId),
    Stem(SymbolId),
    Compound(SymbolId),
    DotVariable(SymbolId),
    Prefix {
        op: PrefixOp,
        operand: ExprRef<'a>,
    },
    Binary {
        op: Operator,
        left: ExprRef<'a>,
        right: ExprRef<'a>,
    },
    Call {
        target: Target<'a>,
        args: Args<'a>,
    },
    QualifiedCall {
        namespace: SymbolId,
        name: SymbolId,
        args: Args<'a>,
    },
    ClassResolver {
        namespace: SymbolId,
        name: SymbolId,
    },
    Message {
        target: ExprRef<'a>,
        name: &'a Selector,
        super_class: Option<ExprRef<'a>>,
        args: Args<'a>,
        cascade: bool,
    },
    List(Args<'a>),
    /// A conditional's comma-separated parts, none of them omitted.
    Logical(Args<'a>),
    /// `>name` or `<name`: `stem` for a stem, a simple variable otherwise.
    VariableReference {
        symbol: SymbolId,
        stem: bool,
    },
}

/// What a call names.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Target<'a> {
    Symbol(SymbolId),
    Literal(&'a [u8]),
}

/// A list of arguments or items, each present or omitted.
#[derive(Copy, Clone)]
pub struct Args<'a> {
    backing: ArgsBacking<'a>,
}

#[derive(Copy, Clone)]
enum ArgsBacking<'a> {
    /// A node's children, where an `Omitted` node reads as omitted.
    Edges { exprs: &'a Exprs, ids: &'a [ExprId] },
    /// An instruction's own list, where `None` is omitted.
    Slots {
        exprs: &'a Exprs,
        ids: &'a [Option<ExprId>],
    },
}

impl<'a> ExprRef<'a> {
    /// The node's id in its arena.
    pub fn id(self) -> ExprId {
        self.id
    }

    /// The node's kind, with its children as views. Always inlined, with
    /// `arena_view`, so a caller's match on one variant compiles to a match
    /// on the node rather than building the whole view.
    #[inline(always)]
    pub fn kind(self) -> ExprView<'a> {
        arena_view(self.exprs, self.id)
    }

    /// The byte range in the retained source, which contains every present
    /// child's span.
    pub fn span(self) -> Range<usize> {
        self.exprs.node(self.id).span()
    }

    // The accessors below each read one variant and answer only what their
    // callers use, so a hot reader does not build the whole `ExprView`.

    /// The symbol of a simple variable, `None` for any other node.
    #[inline]
    pub fn variable(self) -> Option<SymbolId> {
        match self.exprs.node(self.id).kind() {
            NodeKind::Variable(id) => Some(*id),
            _ => None,
        }
    }

    /// What a `Call` node names, `None` for any other node.
    #[inline]
    pub fn call_target(self) -> Option<Target<'a>> {
        match self.exprs.node(self.id).kind() {
            NodeKind::Call { target, .. } => Some(match target {
                CallTarget::Symbol(id) => Target::Symbol(*id),
                CallTarget::Literal(bytes) => Target::Literal(self.exprs.bytes(*bytes)),
            }),
            _ => None,
        }
    }

    /// The children a node path steps through, as [`Branches`] groups them.
    #[inline]
    pub fn branches(self) -> Branches<'a> {
        let exprs = self.exprs;
        let edges = |range: EdgeRange| Args::edges(exprs, exprs.edges(range));
        match exprs.node(self.id).kind() {
            NodeKind::Prefix { operand, .. } => Branches::Prefix(exprs.get(*operand)),
            NodeKind::Binary { left, right, .. } => {
                Branches::Binary(exprs.get(*left), exprs.get(*right))
            }
            NodeKind::Call { args, .. }
            | NodeKind::QualifiedCall { args, .. }
            | NodeKind::List(args) => Branches::Items(edges(*args)),
            NodeKind::Message {
                scoped, children, ..
            } => {
                // The receiver, then the scope where scoped, then the
                // arguments, as `ExprView::Message` splits them.
                let children = exprs.edges(*children);
                Branches::Message {
                    target: exprs.get(children[0]),
                    args: Args::edges(exprs, &children[1 + usize::from(*scoped)..]),
                }
            }
            _ => Branches::Other,
        }
    }
}

/// A node's children as a node path addresses them.
#[derive(Copy, Clone, Debug)]
pub enum Branches<'a> {
    /// A prefix operator's operand.
    Prefix(ExprRef<'a>),
    /// A dyadic operator's left and right operands.
    Binary(ExprRef<'a>, ExprRef<'a>),
    /// A call's arguments, a qualified call's, or a parenthesised list's
    /// items.
    Items(Args<'a>),
    /// A message's receiver and its arguments; a scope is not addressed.
    Message { target: ExprRef<'a>, args: Args<'a> },
    /// Any node a path does not step into: a leaf, or a conditional's list.
    Other,
}

impl std::fmt::Debug for ExprRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExprRef")
            .field("span", &self.span())
            .finish()
    }
}

impl Exprs {
    /// The view of the node `id`.
    pub fn get(&self, id: ExprId) -> ExprRef<'_> {
        ExprRef { exprs: self, id }
    }

    /// The views of the subtree rooted at `root`, children before their
    /// parent, ending with `root`: the arena's `range` as views, without its
    /// `Omitted` nodes, which are read through their list and never viewed.
    pub fn subtree(&self, root: ExprId) -> impl Iterator<Item = ExprRef<'_>> {
        (self.node(root).first.index()..=root.index())
            .map(ExprId::after)
            .filter(|id| !matches!(self.node(*id).kind, NodeKind::Omitted))
            .map(|id| self.get(id))
    }
}

#[inline(always)]
fn arena_view(exprs: &Exprs, id: ExprId) -> ExprView<'_> {
    let edges = |range: EdgeRange| Args {
        backing: ArgsBacking::Edges {
            exprs,
            ids: exprs.edges(range),
        },
    };
    match exprs.node(id).kind() {
        NodeKind::Literal(bytes) => ExprView::Literal(exprs.bytes(*bytes)),
        NodeKind::Constant(id) => ExprView::Constant(*id),
        NodeKind::Variable(id) => ExprView::Variable(*id),
        NodeKind::Stem(id) => ExprView::Stem(*id),
        NodeKind::Compound(id) => ExprView::Compound(*id),
        NodeKind::DotVariable(id) => ExprView::DotVariable(*id),
        NodeKind::VariableReference { symbol, stem } => ExprView::VariableReference {
            symbol: *symbol,
            stem: *stem,
        },
        NodeKind::ClassResolver { namespace, name } => ExprView::ClassResolver {
            namespace: *namespace,
            name: *name,
        },
        NodeKind::Omitted => panic!("an omitted node is read through its list, never viewed"),
        NodeKind::Prefix { op, operand } => ExprView::Prefix {
            op: *op,
            operand: exprs.get(*operand),
        },
        NodeKind::Binary { op, left, right } => ExprView::Binary {
            op: *op,
            left: exprs.get(*left),
            right: exprs.get(*right),
        },
        NodeKind::Call { target, args } => ExprView::Call {
            target: match target {
                CallTarget::Symbol(id) => Target::Symbol(*id),
                CallTarget::Literal(bytes) => Target::Literal(exprs.bytes(*bytes)),
            },
            args: edges(*args),
        },
        NodeKind::QualifiedCall {
            namespace,
            name,
            args,
        } => ExprView::QualifiedCall {
            namespace: *namespace,
            name: *name,
            args: edges(*args),
        },
        NodeKind::Message {
            name,
            cascade,
            scoped,
            children,
        } => {
            // The receiver, then the scope where scoped, then the arguments.
            let children = exprs.edges(*children);
            let arguments = 1 + usize::from(*scoped);
            ExprView::Message {
                target: exprs.get(children[0]),
                name,
                super_class: scoped.then(|| exprs.get(children[1])),
                args: Args {
                    backing: ArgsBacking::Edges {
                        exprs,
                        ids: &children[arguments..],
                    },
                },
                cascade: *cascade,
            }
        }
        NodeKind::List(items) => ExprView::List(edges(*items)),
        NodeKind::Logical(items) => ExprView::Logical(edges(*items)),
    }
}

impl<'a> Args<'a> {
    /// A node's children, or an instruction's list whose items are all
    /// present, over `exprs`; an `Omitted` node reads as omitted.
    pub(crate) fn edges(exprs: &'a Exprs, ids: &'a [ExprId]) -> Args<'a> {
        Args {
            backing: ArgsBacking::Edges { exprs, ids },
        }
    }

    /// An instruction's own list over `exprs`, where `None` is omitted.
    pub(crate) fn slots(exprs: &'a Exprs, ids: &'a [Option<ExprId>]) -> Args<'a> {
        Args {
            backing: ArgsBacking::Slots { exprs, ids },
        }
    }

    /// How many positions the list has, omitted ones included.
    pub fn len(self) -> usize {
        match self.backing {
            ArgsBacking::Edges { ids, .. } => ids.len(),
            ArgsBacking::Slots { ids, .. } => ids.len(),
        }
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// The item at `index`: `None` past the end, `Some(None)` where it is
    /// omitted.
    pub fn get(self, index: usize) -> Option<Option<ExprRef<'a>>> {
        match self.backing {
            ArgsBacking::Edges { exprs, ids } => ids.get(index).map(|&id| {
                (!matches!(exprs.node(id).kind(), NodeKind::Omitted)).then(|| exprs.get(id))
            }),
            ArgsBacking::Slots { exprs, ids } => {
                ids.get(index).map(|slot| slot.map(|id| exprs.get(id)))
            }
        }
    }

    /// The items in order, `None` where one is omitted.
    pub fn iter(self) -> impl Iterator<Item = Option<ExprRef<'a>>> + 'a {
        (0..self.len()).filter_map(move |index| self.get(index))
    }
}

impl std::fmt::Debug for Args<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Args").field("len", &self.len()).finish()
    }
}
