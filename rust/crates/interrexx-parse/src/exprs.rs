//! The expression arena of one code body: its nodes in post-order, with the
//! child lists and literal bytes in side vectors.
//!
//! An [`ExprId`] names a node of the arena. The arena satisfies four
//! invariants, which [`Exprs::check`] verifies:
//!
//! 1. **Children before parents.** Every child's id is less than its
//!    parent's.
//! 2. **A subtree is a contiguous range, in evaluation order.** The nodes of
//!    the subtree rooted at `n` are exactly `first(n)..=n`, and their order is
//!    the order in which evaluation completes them: post-order, children left
//!    to right.
//! 3. **A tree, not a DAG.** Every node is the child of exactly one parent, or
//!    is the root of exactly one expression slot of one instruction. No node
//!    is unreachable.
//! 4. **`parent` and `position` agree with the parents' child lists.**
//!
//! Every walk over an expression is a loop over [`Exprs::range`], never a
//! recursion over the children:
//!
//! * a scan asks whether the subtree holds a node of some kind:
//!   `exprs.range(n).iter().any(|node| matches!(node.kind(), NodeKind::Call { .. }))`;
//! * a forward attribute, computed from the children, is a loop over the
//!   range in index order, which sees every child before its parent;
//! * a backward attribute, handed down from the parent, is a loop over the
//!   range in reverse index order, which sees every parent before its
//!   children;
//! * compilation is a loop over the range in index order with a stack of the
//!   values not yet consumed, where work a parent does between two children
//!   happens at the child's root, which knows its `parent` and `position`.
//!
//! The derived `Clone`, `PartialEq` and `Debug` are over flat vectors, so
//! they do not recurse either.

use std::num::NonZeroU32;
use std::ops::Range;

use crate::ast::PrefixOp;
use crate::selector::Selector;
use crate::token::{Operator, SymbolId};

/// A node of an arena. `ExprId(n)` is the arena's `n`th node, counting from
/// one, so that `Option<ExprId>` is as small as an `ExprId`.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ExprId(pub(crate) NonZeroU32);

impl ExprId {
    /// The node's index in the arena's node vector.
    pub fn index(self) -> usize {
        self.0.get() as usize - 1
    }

    /// The id of the node at `index`.
    #[cfg(test)]
    pub(crate) fn from_index(index: usize) -> ExprId {
        ExprId::after(index)
    }

    /// The id of the node pushed after `len` nodes.
    fn after(len: usize) -> ExprId {
        u32::try_from(len)
            .ok()
            .and_then(|len| NonZeroU32::MIN.checked_add(len))
            .map(ExprId)
            .expect("an expression arena's node ids are u32")
    }

    /// This id moved `by` nodes on.
    fn shifted(self, by: u32) -> ExprId {
        ExprId(
            self.0
                .checked_add(by)
                .expect("an expression arena's node ids are u32"),
        )
    }
}

/// The children of a node with a list of them: a range of the arena's edges.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct EdgeRange {
    pub(crate) start: u32,
    pub(crate) end: u32,
}

/// A literal's or call target's bytes: a range of the arena's bytes.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct BytesRange {
    pub(crate) start: u32,
    pub(crate) end: u32,
}

/// What a `Call` node calls.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CallTarget {
    Symbol(SymbolId),
    Literal(BytesRange),
}

/// A node's kind, with its payload and its child ids.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NodeKind {
    Literal(BytesRange),
    Constant(SymbolId),
    Variable(SymbolId),
    Stem(SymbolId),
    Compound(SymbolId),
    DotVariable(SymbolId),
    /// `>name` or `<name`: the variable is named, never evaluated, so it is a
    /// leaf.
    VariableReference {
        symbol: SymbolId,
        stem: bool,
    },
    ClassResolver {
        namespace: SymbolId,
        name: SymbolId,
    },
    /// An omitted argument or list item, a node so that its position has a
    /// place in the evaluation order.
    Omitted,
    Prefix {
        op: PrefixOp,
        operand: ExprId,
    },
    Binary {
        op: Operator,
        left: ExprId,
        right: ExprId,
    },
    Call {
        target: CallTarget,
        args: EdgeRange,
    },
    QualifiedCall {
        namespace: SymbolId,
        name: SymbolId,
        args: EdgeRange,
    },
    /// The children are the receiver, then the scope where `scoped`, then
    /// the arguments, the order they are evaluated in.
    Message {
        name: Selector,
        cascade: bool,
        scoped: bool,
        children: EdgeRange,
    },
    List(EdgeRange),
    Logical(EdgeRange),
}

/// One node of an arena.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Node {
    pub(crate) kind: NodeKind,
    pub(crate) start: u32,
    pub(crate) end: u32,
    pub(crate) first: ExprId,
    pub(crate) parent: Option<ExprId>,
    pub(crate) position: u32,
}

// A `Selector` is a pointer and a length, so the size follows the pointer
// width.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Node>() == 56);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(size_of::<Node>() == 44);

impl Node {
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    /// The byte range in the retained source, which contains every present
    /// child's span.
    pub fn span(&self) -> Range<usize> {
        self.start as usize..self.end as usize
    }

    /// The first node of this node's subtree, which is this node for a leaf.
    pub fn first(&self) -> ExprId {
        self.first
    }

    /// The node this one is a child of, or `None` for the root of an
    /// expression slot.
    pub fn parent(&self) -> Option<ExprId> {
        self.parent
    }

    /// This node's index among its parent's children.
    pub fn position(&self) -> u32 {
        self.position
    }
}

/// The expressions of one code body.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Exprs {
    pub(crate) nodes: Vec<Node>,
    pub(crate) edges: Vec<ExprId>,
    pub(crate) bytes: Vec<u8>,
}

/// How far an arena reached, for [`Exprs::rewind`].
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "the parser does not build an arena yet")
)]
pub(crate) struct Mark {
    nodes: u32,
    edges: u32,
    bytes: u32,
}

/// The invariant an arena breaks, at the index of the first node found to
/// break it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Inconsistent {
    /// A child id, edge range, byte range or root lies outside the arena.
    OutOfRange { node: usize },
    /// A child's id is not below its parent's.
    ChildAfterParent { node: usize },
    /// A message lacks its receiver, or its scope where it is scoped.
    Arity { node: usize },
    /// A subtree is not the contiguous range ending at its root.
    NotContiguous { node: usize },
    /// A node is reached by parents and roots other than once.
    Reached { node: usize, times: u32 },
    /// A child's `parent` or `position` disagrees with its parent's child
    /// list, or a root has a parent.
    Link { node: usize },
}

/// The child ids of `kind`, written to `inline` for the kinds that hold them
/// in the node, or `None` when its edge range lies outside `edges`.
fn children<'a>(
    edges: &'a [ExprId],
    kind: &NodeKind,
    inline: &'a mut [ExprId; 2],
) -> Option<&'a [ExprId]> {
    match *kind {
        NodeKind::Prefix { operand, .. } => {
            inline[0] = operand;
            Some(&inline[..1])
        }
        NodeKind::Binary { left, right, .. } => {
            *inline = [left, right];
            Some(&inline[..])
        }
        NodeKind::Call { args, .. }
        | NodeKind::QualifiedCall { args, .. }
        | NodeKind::Message { children: args, .. }
        | NodeKind::List(args)
        | NodeKind::Logical(args) => edges.get(args.start as usize..args.end as usize),
        NodeKind::Literal(_)
        | NodeKind::Constant(_)
        | NodeKind::Variable(_)
        | NodeKind::Stem(_)
        | NodeKind::Compound(_)
        | NodeKind::DotVariable(_)
        | NodeKind::VariableReference { .. }
        | NodeKind::ClassResolver { .. }
        | NodeKind::Omitted => Some(&[]),
    }
}

/// The byte range `kind` reads, if any.
fn bytes_of(kind: &NodeKind) -> Option<BytesRange> {
    match *kind {
        NodeKind::Literal(bytes)
        | NodeKind::Call {
            target: CallTarget::Literal(bytes),
            ..
        } => Some(bytes),
        _ => None,
    }
}

/// `len` as an arena offset.
fn offset(len: usize, what: &str) -> u32 {
    u32::try_from(len).unwrap_or_else(|_| panic!("an expression arena's {what} are u32"))
}

impl Exprs {
    /// How many nodes the arena holds.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn node(&self, id: ExprId) -> &Node {
        &self.nodes[id.index()]
    }

    /// The subtree rooted at `id`, in evaluation order: `first(id)..=id`.
    pub fn range(&self, id: ExprId) -> &[Node] {
        &self.nodes[self.node(id).first.index()..=id.index()]
    }

    pub fn edges(&self, range: EdgeRange) -> &[ExprId] {
        &self.edges[range.start as usize..range.end as usize]
    }

    pub fn bytes(&self, range: BytesRange) -> &[u8] {
        &self.bytes[range.start as usize..range.end as usize]
    }

    /// Checks the four invariants, with `roots` the roots of the body's
    /// expression slots. Reads every node, edge and byte range through
    /// bounds-checked lookups, so any arena answers rather than panicking,
    /// and takes time linear in the nodes plus the edges.
    ///
    /// # Errors
    ///
    /// The first invariant found broken, at the node that breaks it.
    pub fn check(&self, roots: impl IntoIterator<Item = ExprId>) -> Result<(), Inconsistent> {
        let mut reached = vec![0u32; self.nodes.len()];
        let mut inline = [ExprId(NonZeroU32::MIN); 2];
        let at = |id: ExprId| self.nodes.get(id.index());
        for (node, parent) in self.nodes.iter().enumerate() {
            if let Some(bytes) = bytes_of(&parent.kind)
                && self
                    .bytes
                    .get(bytes.start as usize..bytes.end as usize)
                    .is_none()
            {
                return Err(Inconsistent::OutOfRange { node });
            }
            let Some(children) = children(&self.edges, &parent.kind, &mut inline) else {
                return Err(Inconsistent::OutOfRange { node });
            };
            for &child in children {
                if child.index() >= self.nodes.len() {
                    return Err(Inconsistent::OutOfRange { node });
                }
                if child.index() >= node {
                    return Err(Inconsistent::ChildAfterParent { node });
                }
            }
            if let NodeKind::Message { scoped, .. } = parent.kind
                && children.len() < 1 + usize::from(scoped)
            {
                return Err(Inconsistent::Arity { node });
            }
            let contiguous = match (children.first(), children.last()) {
                (Some(&head), Some(last)) => {
                    at(head).map(Node::first) == Some(parent.first)
                        && children.windows(2).all(|pair| match *pair {
                            [previous, child] => at(child)
                                .is_some_and(|child| child.first.index() == previous.index() + 1),
                            _ => false,
                        })
                        && last.index() + 1 == node
                }
                _ => parent.first.index() == node,
            };
            if !contiguous {
                return Err(Inconsistent::NotContiguous { node });
            }
            for (position, &child) in children.iter().enumerate() {
                let linked = at(child).is_some_and(|linked| {
                    linked.parent.map(ExprId::index) == Some(node)
                        && linked.position as usize == position
                });
                if !linked {
                    return Err(Inconsistent::Link {
                        node: child.index(),
                    });
                }
                if let Some(times) = reached.get_mut(child.index()) {
                    *times = times.saturating_add(1);
                }
            }
        }
        for root in roots {
            let Some(node) = at(root) else {
                return Err(Inconsistent::OutOfRange { node: root.index() });
            };
            if node.parent.is_some() {
                return Err(Inconsistent::Link { node: root.index() });
            }
            if let Some(times) = reached.get_mut(root.index()) {
                *times = times.saturating_add(1);
            }
        }
        match reached.iter().enumerate().find(|&(_, &times)| times != 1) {
            Some((node, &times)) => Err(Inconsistent::Reached { node, times }),
            None => Ok(()),
        }
    }

    /// The node at `id`, for a test to break an invariant through.
    #[cfg(test)]
    pub(crate) fn node_mut(&mut self, id: ExprId) -> &mut Node {
        &mut self.nodes[id.index()]
    }
}

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "the parser does not build an arena yet")
)]
impl Exprs {
    /// Appends `bytes` and answers their range.
    pub(crate) fn push_bytes(&mut self, bytes: &[u8]) -> BytesRange {
        let start = offset(self.bytes.len(), "byte offsets");
        self.bytes.extend_from_slice(bytes);
        BytesRange {
            start,
            end: offset(self.bytes.len(), "byte offsets"),
        }
    }

    /// Appends `children` as one child list and answers its range.
    pub(crate) fn push_edges(&mut self, children: &[ExprId]) -> EdgeRange {
        let start = offset(self.edges.len(), "edge offsets");
        self.edges.extend_from_slice(children);
        EdgeRange {
            start,
            end: offset(self.edges.len(), "edge offsets"),
        }
    }

    /// Pushes a node of `kind` over children already pushed, which must be
    /// the nodes just before it. Sets each child's `parent` and `position`,
    /// and widens `extent` by every child's span but an omitted one's.
    pub(crate) fn push(&mut self, kind: NodeKind, extent: Range<usize>) -> ExprId {
        let id = ExprId::after(self.nodes.len());
        let (mut start, mut end) = (extent.start, extent.end);
        let mut first = id;
        let mut inline = [id; 2];
        let children = children(&self.edges, &kind, &mut inline)
            .expect("a pushed node's child list is in the arena");
        for (position, &child) in children.iter().enumerate() {
            let node = &mut self.nodes[child.index()];
            if position == 0 {
                first = node.first;
            } else {
                debug_assert_eq!(
                    node.first.index(),
                    children[position - 1].index() + 1,
                    "a node's children tile the nodes before it"
                );
            }
            node.parent = Some(id);
            node.position = offset(position, "child positions");
            if !matches!(node.kind, NodeKind::Omitted) {
                start = start.min(node.start as usize);
                end = end.max(node.end as usize);
            }
        }
        debug_assert!(
            children
                .last()
                .is_none_or(|last| last.index() + 1 == id.index()),
            "a node's last child is the node pushed before it"
        );
        self.nodes.push(Node {
            kind,
            start: offset(start, "source offsets"),
            end: offset(end, "source offsets"),
            first,
            parent: None,
            position: 0,
        });
        id
    }

    /// Pushes the subtree of `root` again, with its ids and edge ranges
    /// rebased onto the copy and its byte ranges shared, and answers the
    /// copy's root, which has no parent.
    pub(crate) fn copy(&mut self, root: ExprId) -> ExprId {
        let first = self.node(root).first.index();
        let by = offset(self.nodes.len() - first, "node ids");
        self.nodes.reserve(root.index() + 1 - first);
        for index in first..=root.index() {
            let mut node = self.nodes[index].clone();
            match &mut node.kind {
                NodeKind::Prefix { operand, .. } => *operand = operand.shifted(by),
                NodeKind::Binary { left, right, .. } => {
                    *left = left.shifted(by);
                    *right = right.shifted(by);
                }
                NodeKind::Call { args, .. }
                | NodeKind::QualifiedCall { args, .. }
                | NodeKind::Message { children: args, .. }
                | NodeKind::List(args)
                | NodeKind::Logical(args) => {
                    let start = offset(self.edges.len(), "edge offsets");
                    for edge in args.start as usize..args.end as usize {
                        let child = self.edges[edge].shifted(by);
                        self.edges.push(child);
                    }
                    *args = EdgeRange {
                        start,
                        end: offset(self.edges.len(), "edge offsets"),
                    };
                }
                NodeKind::Literal(_)
                | NodeKind::Constant(_)
                | NodeKind::Variable(_)
                | NodeKind::Stem(_)
                | NodeKind::Compound(_)
                | NodeKind::DotVariable(_)
                | NodeKind::VariableReference { .. }
                | NodeKind::ClassResolver { .. }
                | NodeKind::Omitted => {}
            }
            node.first = node.first.shifted(by);
            if index == root.index() {
                node.parent = None;
                node.position = 0;
            } else {
                node.parent = node.parent.map(|parent| parent.shifted(by));
            }
            self.nodes.push(node);
        }
        root.shifted(by)
    }

    /// How far the arena reaches now.
    pub(crate) fn mark(&self) -> Mark {
        Mark {
            nodes: offset(self.nodes.len(), "node ids"),
            edges: offset(self.edges.len(), "edge offsets"),
            bytes: offset(self.bytes.len(), "byte offsets"),
        }
    }

    /// Drops everything pushed since `mark` was taken.
    pub(crate) fn rewind(&mut self, mark: Mark) {
        self.nodes.truncate(mark.nodes as usize);
        self.edges.truncate(mark.edges as usize);
        self.bytes.truncate(mark.bytes as usize);
    }
}

#[cfg(test)]
mod tests;
