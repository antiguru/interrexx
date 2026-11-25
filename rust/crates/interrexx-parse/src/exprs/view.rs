//! A borrowed view of one expression node, backed either by the boxed tree
//! or by an arena, so that a reader written against the view reads both.
//!
//! Every method reads one node: a child is answered as another view, never
//! visited, so the view itself never walks.

use std::ops::Range;

use crate::ast::{CallTarget as TreeTarget, Expr, ExprKind, PrefixOp};
use crate::selector::Selector;
use crate::token::{Operator, SymbolId};

use super::{CallTarget, EdgeRange, ExprId, Exprs, NodeKind};

/// One expression node, borrowed.
#[derive(Copy, Clone)]
pub struct ExprRef<'a> {
    backing: Backing<'a>,
}

#[derive(Copy, Clone)]
enum Backing<'a> {
    Tree(&'a Expr),
    Arena { exprs: &'a Exprs, id: ExprId },
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
    Tree(&'a [Option<Expr>]),
    TreeRequired(&'a [Expr]),
    /// A node's children, where an `Omitted` node reads as omitted.
    Edges {
        exprs: &'a Exprs,
        ids: &'a [ExprId],
    },
    /// An instruction's own list, where `None` is omitted.
    Slots {
        exprs: &'a Exprs,
        ids: &'a [Option<ExprId>],
    },
}

impl<'a> ExprRef<'a> {
    /// The view of a tree node.
    pub fn tree(expr: &'a Expr) -> ExprRef<'a> {
        ExprRef {
            backing: Backing::Tree(expr),
        }
    }

    /// The node's kind, with its children as views.
    pub fn kind(self) -> ExprView<'a> {
        match self.backing {
            Backing::Tree(expr) => tree_view(expr),
            Backing::Arena { exprs, id } => arena_view(exprs, id),
        }
    }

    /// The byte range in the retained source, which contains every present
    /// child's span.
    pub fn span(self) -> Range<usize> {
        match self.backing {
            Backing::Tree(expr) => expr.span.clone(),
            Backing::Arena { exprs, id } => exprs.node(id).span(),
        }
    }
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
        ExprRef {
            backing: Backing::Arena { exprs: self, id },
        }
    }
}

fn tree_view(expr: &Expr) -> ExprView<'_> {
    match &expr.kind {
        ExprKind::Literal(bytes) => ExprView::Literal(bytes),
        ExprKind::Constant(id) => ExprView::Constant(*id),
        ExprKind::Variable(id) => ExprView::Variable(*id),
        ExprKind::Stem(id) => ExprView::Stem(*id),
        ExprKind::Compound(id) => ExprView::Compound(*id),
        ExprKind::DotVariable(id) => ExprView::DotVariable(*id),
        ExprKind::Prefix { op, operand } => ExprView::Prefix {
            op: *op,
            operand: ExprRef::tree(operand),
        },
        ExprKind::Binary { op, left, right } => ExprView::Binary {
            op: *op,
            left: ExprRef::tree(left),
            right: ExprRef::tree(right),
        },
        ExprKind::Call { target, args } => ExprView::Call {
            target: match target {
                TreeTarget::Symbol(id) => Target::Symbol(*id),
                TreeTarget::Literal(bytes) => Target::Literal(bytes),
            },
            args: Args::tree(args),
        },
        ExprKind::QualifiedCall {
            namespace,
            name,
            args,
        } => ExprView::QualifiedCall {
            namespace: *namespace,
            name: *name,
            args: Args::tree(args),
        },
        ExprKind::ClassResolver { namespace, name } => ExprView::ClassResolver {
            namespace: *namespace,
            name: *name,
        },
        ExprKind::Message {
            target,
            name,
            super_class,
            args,
            cascade,
        } => ExprView::Message {
            target: ExprRef::tree(target),
            name,
            super_class: super_class.as_deref().map(ExprRef::tree),
            args: Args::tree(args),
            cascade: *cascade,
        },
        ExprKind::List(items) => ExprView::List(Args::tree(items)),
        ExprKind::Logical(items) => ExprView::Logical(Args::tree_required(items)),
        ExprKind::VariableReference(inner) => match inner.kind {
            ExprKind::Variable(symbol) => ExprView::VariableReference {
                symbol,
                stem: false,
            },
            ExprKind::Stem(symbol) => ExprView::VariableReference { symbol, stem: true },
            _ => panic!("a variable reference names a variable or a stem"),
        },
    }
}

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
    pub(crate) fn tree(list: &'a [Option<Expr>]) -> Args<'a> {
        Args {
            backing: ArgsBacking::Tree(list),
        }
    }

    pub(crate) fn tree_required(list: &'a [Expr]) -> Args<'a> {
        Args {
            backing: ArgsBacking::TreeRequired(list),
        }
    }

    /// An instruction's own list over `exprs`, where `None` is omitted.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "no code body holds an arena yet")
    )]
    pub(crate) fn slots(exprs: &'a Exprs, ids: &'a [Option<ExprId>]) -> Args<'a> {
        Args {
            backing: ArgsBacking::Slots { exprs, ids },
        }
    }

    /// How many positions the list has, omitted ones included.
    pub fn len(self) -> usize {
        match self.backing {
            ArgsBacking::Tree(list) => list.len(),
            ArgsBacking::TreeRequired(list) => list.len(),
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
            ArgsBacking::Tree(list) => list.get(index).map(|item| item.as_ref().map(ExprRef::tree)),
            ArgsBacking::TreeRequired(list) => {
                list.get(index).map(|item| Some(ExprRef::tree(item)))
            }
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
