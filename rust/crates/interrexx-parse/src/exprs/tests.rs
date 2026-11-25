//! The arena's builder and validator. Spans are byte offsets into the text in
//! each test's comment.

use crate::ast::PrefixOp;
use crate::selector::SelectorTable;
use crate::token::{Operator, SymbolId};

use super::{CallTarget, ExprId, Exprs, Inconsistent, NodeKind};

fn literal(exprs: &mut Exprs, text: &[u8], at: usize) -> ExprId {
    let bytes = exprs.push_bytes(text);
    exprs.push(NodeKind::Literal(bytes), at..at + text.len())
}

fn symbol(index: usize) -> SymbolId {
    SymbolId::from_index(index).expect("a small index is a symbol id")
}

/// `1 + 2`, answering the three ids in push order.
fn sum(exprs: &mut Exprs) -> (ExprId, ExprId, ExprId) {
    let one = literal(exprs, b"1", 0);
    let two = literal(exprs, b"2", 4);
    let sum = exprs.push(
        NodeKind::Binary {
            op: Operator::Plus,
            left: one,
            right: two,
        },
        2..3,
    );
    (one, two, sum)
}

#[test]
fn a_binary_node_follows_its_operands_and_links_them() {
    // `1 + 2`
    let mut exprs = Exprs::default();
    let (one, two, sum) = sum(&mut exprs);
    assert_eq!(exprs.range(sum).len(), 3);
    assert_eq!(exprs.node(sum).first(), one);
    assert_eq!(exprs.node(sum).span(), 0..5);
    assert_eq!(
        (exprs.node(one).parent(), exprs.node(one).position()),
        (Some(sum), 0)
    );
    assert_eq!(
        (exprs.node(two).parent(), exprs.node(two).position()),
        (Some(sum), 1)
    );
    assert_eq!(exprs.check([sum]), Ok(()));
}

#[test]
fn a_prefix_node_is_widened_to_its_operand() {
    // `- 12`
    let mut exprs = Exprs::default();
    let twelve = literal(&mut exprs, b"12", 2);
    let negated = exprs.push(
        NodeKind::Prefix {
            op: PrefixOp::Minus,
            operand: twelve,
        },
        0..1,
    );
    assert_eq!(exprs.node(negated).span(), 0..4);
    assert_eq!(exprs.node(negated).first(), twelve);
    assert_eq!(exprs.node(twelve).parent(), Some(negated));
    assert_eq!(exprs.check([negated]), Ok(()));
}

#[test]
fn an_omitted_argument_is_a_node_that_widens_nothing() {
    // `f(,2)`
    let mut exprs = Exprs::default();
    let omitted = exprs.push(NodeKind::Omitted, 2..2);
    let two = literal(&mut exprs, b"2", 3);
    let args = exprs.push_edges(&[omitted, two]);
    let call = exprs.push(
        NodeKind::Call {
            target: CallTarget::Symbol(symbol(0)),
            args,
        },
        0..1,
    );
    assert_eq!(exprs.node(call).span(), 0..4);
    assert_eq!(exprs.edges(args), &[omitted, two]);
    assert_eq!(exprs.node(call).first(), omitted);
    assert_eq!(
        (exprs.node(omitted).parent(), exprs.node(omitted).position()),
        (Some(call), 0)
    );
    assert_eq!(exprs.check([call]), Ok(()));

    // An omitted node far outside the extent leaves it alone: only a present
    // child widens its parent.
    let mut exprs = Exprs::default();
    let omitted = exprs.push(NodeKind::Omitted, 40..40);
    let args = exprs.push_edges(&[omitted]);
    let call = exprs.push(
        NodeKind::Call {
            target: CallTarget::Symbol(symbol(0)),
            args,
        },
        0..1,
    );
    assert_eq!(exprs.node(call).span(), 0..1);
}

/// The first `children` of `a~m:s(1)`'s receiver, scope and argument, under
/// a message with `scoped` as given, answering the message's id.
fn scoped_message(exprs: &mut Exprs, scoped: bool, children: usize) -> ExprId {
    let mut ids = Vec::new();
    if children > 0 {
        ids.push(exprs.push(NodeKind::Variable(symbol(0)), 0..1));
    }
    if children > 1 {
        ids.push(exprs.push(NodeKind::Variable(symbol(1)), 4..5));
    }
    if children > 2 {
        ids.push(literal(exprs, b"1", 6));
    }
    let edges = exprs.push_edges(&ids);
    exprs.push(
        NodeKind::Message {
            name: SelectorTable::new().intern(b"M"),
            cascade: false,
            scoped,
            children: edges,
        },
        1..3,
    )
}

#[test]
fn a_message_lists_its_receiver_then_its_scope_then_its_arguments() {
    // `a~m:s(1)`
    let mut exprs = Exprs::default();
    let message = scoped_message(&mut exprs, true, 3);
    let NodeKind::Message { children, .. } = exprs.node(message).kind().clone() else {
        panic!("a message");
    };
    let children = exprs.edges(children).to_vec();
    let positions: Vec<_> = children
        .iter()
        .map(|&child| (exprs.node(child).parent(), exprs.node(child).position()))
        .collect();
    assert_eq!(
        positions,
        [(Some(message), 0), (Some(message), 1), (Some(message), 2)]
    );
    assert_eq!(exprs.node(message).span(), 0..7);
    assert_eq!(exprs.check([message]), Ok(()));

    // Scoped, but with the receiver alone.
    let mut exprs = Exprs::default();
    let message = scoped_message(&mut exprs, true, 1);
    assert_eq!(
        exprs.check([message]),
        Err(Inconsistent::Arity {
            node: message.index()
        })
    );

    // Scoped with a receiver and a scope and no arguments is whole.
    let mut exprs = Exprs::default();
    let message = scoped_message(&mut exprs, true, 2);
    assert_eq!(exprs.check([message]), Ok(()));

    // Unscoped, the receiver alone is a whole message.
    let mut exprs = Exprs::default();
    let message = scoped_message(&mut exprs, false, 1);
    assert_eq!(exprs.check([message]), Ok(()));

    // Any message needs its receiver.
    let mut exprs = Exprs::default();
    let message = scoped_message(&mut exprs, false, 0);
    assert_eq!(
        exprs.check([message]),
        Err(Inconsistent::Arity {
            node: message.index()
        })
    );
}

#[test]
fn rewinding_drops_what_was_pushed_after_the_mark() {
    // `1 + 2`, then `f('x')` discarded
    let mut exprs = Exprs::default();
    let (_, _, sum) = sum(&mut exprs);
    let (nodes, edges, bytes) = (exprs.len(), exprs.edges.len(), exprs.bytes.len());
    let mark = exprs.mark();
    let x = literal(&mut exprs, b"x", 8);
    let args = exprs.push_edges(&[x]);
    let target = exprs.push_bytes(b"f");
    exprs.push(
        NodeKind::Call {
            target: CallTarget::Literal(target),
            args,
        },
        6..7,
    );
    exprs.rewind(mark);
    assert_eq!(
        (exprs.len(), exprs.edges.len(), exprs.bytes.len()),
        (nodes, edges, bytes)
    );
    assert_eq!(exprs.check([sum]), Ok(()));
}

#[test]
fn a_copy_is_the_subtree_pushed_again_with_its_ids_rebased() {
    // `f(1, 'x')`
    let mut exprs = Exprs::default();
    let one = literal(&mut exprs, b"1", 2);
    let x = literal(&mut exprs, b"x", 5);
    let args = exprs.push_edges(&[one, x]);
    let target = exprs.push_bytes(b"F");
    let original = exprs.push(
        NodeKind::Call {
            target: CallTarget::Literal(target),
            args,
        },
        0..1,
    );
    let length = exprs.len();
    let (edges, bytes) = (exprs.edges.len(), exprs.bytes.len());

    let copy = exprs.copy(original);

    assert_eq!(exprs.len(), 2 * length);
    assert_eq!(copy.index(), original.index() + length);
    assert_eq!(exprs.node(copy).parent(), None);
    assert_eq!(exprs.node(copy).span(), exprs.node(original).span());
    let copied_one = ExprId::from_index(one.index() + length);
    let copied_x = ExprId::from_index(x.index() + length);
    let NodeKind::Call {
        target: copied_target,
        args: copied_args,
    } = exprs.node(copy).kind().clone()
    else {
        panic!("a call");
    };
    assert_eq!(copied_target, CallTarget::Literal(target));
    assert_ne!(copied_args, args);
    assert_eq!(exprs.edges(copied_args), &[copied_one, copied_x]);
    assert_eq!(exprs.edges(args), &[one, x]);
    assert_eq!(exprs.edges.len(), edges + 2);
    assert_eq!(exprs.bytes.len(), bytes);
    let NodeKind::Literal(read) = *exprs.node(copied_x).kind() else {
        panic!("a literal");
    };
    assert_eq!(exprs.bytes(read), b"x");
    assert_eq!(exprs.node(copy).first(), copied_one);
    assert_eq!(
        (
            exprs.node(copied_x).parent(),
            exprs.node(copied_x).position()
        ),
        (Some(copy), 1)
    );
    assert_eq!(exprs.check([original, copy]), Ok(()));

    // A binary copy rebases its operands, which live in the node itself.
    let mut exprs = Exprs::default();
    let (_, _, sum) = sum(&mut exprs);
    let copy = exprs.copy(sum);
    assert_eq!(
        *exprs.node(copy).kind(),
        NodeKind::Binary {
            op: Operator::Plus,
            left: ExprId::from_index(3),
            right: ExprId::from_index(4),
        }
    );
    assert_eq!(exprs.check([sum, copy]), Ok(()));
}

#[test]
fn a_child_past_the_end_is_out_of_range() {
    let mut exprs = Exprs::default();
    let (one, _, sum) = sum(&mut exprs);
    exprs.node_mut(sum).kind = NodeKind::Binary {
        op: Operator::Plus,
        left: one,
        right: ExprId::from_index(7),
    };
    assert_eq!(
        exprs.check([sum]),
        Err(Inconsistent::OutOfRange { node: sum.index() })
    );
}

#[test]
fn an_edge_or_byte_range_past_the_end_is_out_of_range() {
    let mut exprs = Exprs::default();
    let one = literal(&mut exprs, b"1", 2);
    let args = exprs.push_edges(&[one]);
    let call = exprs.push(
        NodeKind::Call {
            target: CallTarget::Symbol(symbol(0)),
            args,
        },
        0..1,
    );
    let valid = exprs.clone();

    exprs.edges.clear();
    assert_eq!(
        exprs.check([call]),
        Err(Inconsistent::OutOfRange { node: call.index() })
    );

    let mut exprs = valid.clone();
    exprs.bytes.clear();
    assert_eq!(
        exprs.check([call]),
        Err(Inconsistent::OutOfRange { node: one.index() })
    );

    let mut exprs = valid;
    exprs.node_mut(call).kind = NodeKind::Call {
        target: CallTarget::Literal(super::BytesRange { start: 0, end: 9 }),
        args,
    };
    assert_eq!(
        exprs.check([call]),
        Err(Inconsistent::OutOfRange { node: call.index() })
    );
}

#[test]
fn a_root_past_the_end_is_out_of_range() {
    let mut exprs = Exprs::default();
    let (_, _, sum) = sum(&mut exprs);
    assert_eq!(
        exprs.check([sum, ExprId::from_index(5)]),
        Err(Inconsistent::OutOfRange { node: 5 })
    );
}

#[test]
fn a_child_at_its_parent_is_after_its_parent() {
    let mut exprs = Exprs::default();
    let (_, two, sum) = sum(&mut exprs);
    exprs.node_mut(sum).kind = NodeKind::Binary {
        op: Operator::Plus,
        left: sum,
        right: two,
    };
    assert_eq!(
        exprs.check([sum]),
        Err(Inconsistent::ChildAfterParent { node: sum.index() })
    );
}

#[test]
fn a_moved_first_is_not_contiguous() {
    let mut exprs = Exprs::default();
    let (_, two, sum) = sum(&mut exprs);
    exprs.node_mut(sum).first = two;
    assert_eq!(
        exprs.check([sum]),
        Err(Inconsistent::NotContiguous { node: sum.index() })
    );
}

#[test]
fn a_leaf_whose_first_is_not_itself_is_not_contiguous() {
    let mut exprs = Exprs::default();
    let (one, two, sum) = sum(&mut exprs);
    exprs.node_mut(two).first = one;
    assert_eq!(
        exprs.check([sum]),
        Err(Inconsistent::NotContiguous { node: two.index() })
    );
}

#[test]
fn a_gap_between_children_is_not_contiguous() {
    // `1 + 2` with a stray `3` between the operands.
    let mut exprs = Exprs::default();
    let one = literal(&mut exprs, b"1", 0);
    let three = literal(&mut exprs, b"3", 2);
    let two = literal(&mut exprs, b"2", 4);
    let sum = exprs.push(
        NodeKind::Binary {
            op: Operator::Plus,
            left: three,
            right: two,
        },
        2..3,
    );
    exprs.node_mut(sum).kind = NodeKind::Binary {
        op: Operator::Plus,
        left: one,
        right: two,
    };
    exprs.node_mut(sum).first = one;
    exprs.node_mut(one).parent = Some(sum);
    exprs.node_mut(three).parent = None;
    assert_eq!(
        exprs.check([sum, three]),
        Err(Inconsistent::NotContiguous { node: sum.index() })
    );
}

#[test]
fn a_last_child_short_of_its_parent_is_not_contiguous() {
    // `1 + 2` with a stray `3` after the right operand.
    let mut exprs = Exprs::default();
    let one = literal(&mut exprs, b"1", 0);
    let two = literal(&mut exprs, b"2", 4);
    let three = literal(&mut exprs, b"3", 6);
    let pair = exprs.push_edges(&[two, three]);
    let list = exprs.push(NodeKind::List(pair), 3..7);
    exprs.node_mut(list).kind = NodeKind::Binary {
        op: Operator::Plus,
        left: one,
        right: two,
    };
    exprs.node_mut(list).first = one;
    exprs.node_mut(one).parent = Some(list);
    exprs.node_mut(two).position = 1;
    exprs.node_mut(three).parent = None;
    assert_eq!(
        exprs.check([list, three]),
        Err(Inconsistent::NotContiguous { node: list.index() })
    );
}

#[test]
fn a_root_left_out_is_reached_no_times() {
    let mut exprs = Exprs::default();
    let (_, _, sum) = sum(&mut exprs);
    assert_eq!(
        exprs.check([]),
        Err(Inconsistent::Reached {
            node: sum.index(),
            times: 0
        })
    );
}

#[test]
fn a_root_passed_twice_is_reached_twice() {
    let mut exprs = Exprs::default();
    let (_, _, sum) = sum(&mut exprs);
    assert_eq!(
        exprs.check([sum, sum]),
        Err(Inconsistent::Reached {
            node: sum.index(),
            times: 2
        })
    );
}

#[test]
fn a_child_whose_parent_points_elsewhere_breaks_the_link() {
    let mut exprs = Exprs::default();
    let (one, two, sum) = sum(&mut exprs);
    exprs.node_mut(one).parent = Some(two);
    assert_eq!(
        exprs.check([sum]),
        Err(Inconsistent::Link { node: one.index() })
    );
}

#[test]
fn a_child_whose_position_is_wrong_breaks_the_link() {
    let mut exprs = Exprs::default();
    let (_, two, sum) = sum(&mut exprs);
    exprs.node_mut(two).position = 0;
    assert_eq!(
        exprs.check([sum]),
        Err(Inconsistent::Link { node: two.index() })
    );
}

#[test]
fn a_root_with_a_parent_breaks_the_link() {
    let mut exprs = Exprs::default();
    let (one, _, sum) = sum(&mut exprs);
    exprs.node_mut(sum).parent = Some(one);
    assert_eq!(
        exprs.check([sum]),
        Err(Inconsistent::Link { node: sum.index() })
    );
}

#[test]
fn leaves_of_every_kind_pass() {
    // `a b. c.d .e >f g:h` as separate roots.
    let mut exprs = Exprs::default();
    let roots = [
        exprs.push(NodeKind::Constant(symbol(0)), 0..1),
        exprs.push(NodeKind::Variable(symbol(1)), 2..3),
        exprs.push(NodeKind::Stem(symbol(2)), 4..6),
        exprs.push(NodeKind::Compound(symbol(3)), 7..10),
        exprs.push(NodeKind::DotVariable(symbol(4)), 11..13),
        exprs.push(
            NodeKind::VariableReference {
                symbol: symbol(5),
                stem: false,
            },
            14..16,
        ),
        exprs.push(
            NodeKind::ClassResolver {
                namespace: symbol(6),
                name: symbol(7),
            },
            17..20,
        ),
    ];
    let items = exprs.push_edges(&roots);
    let logical = exprs.push(NodeKind::Logical(items), 0..20);
    let qualified_args = exprs.push_edges(&[]);
    let qualified = exprs.push(
        NodeKind::QualifiedCall {
            namespace: symbol(6),
            name: symbol(7),
            args: qualified_args,
        },
        21..26,
    );
    assert_eq!(exprs.node(logical).first(), roots[0]);
    assert_eq!(exprs.node(qualified).first(), qualified);
    assert_eq!(exprs.check([logical, qualified]), Ok(()));
}

#[test]
fn a_long_chain_clones_compares_formats_and_drops_on_a_small_thread() {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(|| {
            // `1 + 1 + ...`
            let mut exprs = Exprs::default();
            let mut chain = literal(&mut exprs, b"1", 0);
            for term in 1..1_000_000 {
                let at = 4 * term;
                let one = literal(&mut exprs, b"1", at);
                chain = exprs.push(
                    NodeKind::Binary {
                        op: Operator::Plus,
                        left: chain,
                        right: one,
                    },
                    at - 2..at - 1,
                );
            }
            assert_eq!(exprs.check([chain]), Ok(()));
            let copy = exprs.clone();
            assert!(copy == exprs);
            assert!(!format!("{copy:?}").is_empty());
            drop(copy);
            drop(exprs);
        })
        .expect("a thread spawns")
        .join()
        .expect("the chain thread finishes");
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "a node's children tile the nodes before it")]
fn pushing_over_children_with_a_gap_between_them_is_a_bug() {
    // `1 + 2` with a stray `3` between the operands.
    let mut exprs = Exprs::default();
    let one = literal(&mut exprs, b"1", 0);
    literal(&mut exprs, b"3", 2);
    let two = literal(&mut exprs, b"2", 4);
    exprs.push(
        NodeKind::Binary {
            op: Operator::Plus,
            left: one,
            right: two,
        },
        2..3,
    );
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "a node's last child is the node pushed before it")]
fn pushing_over_children_short_of_the_last_node_is_a_bug() {
    // `- 1` with a stray `2` after the operand.
    let mut exprs = Exprs::default();
    let one = literal(&mut exprs, b"1", 2);
    literal(&mut exprs, b"2", 4);
    exprs.push(
        NodeKind::Prefix {
            op: PrefixOp::Minus,
            operand: one,
        },
        0..1,
    );
}

#[test]
fn a_copy_of_an_operand_is_a_root() {
    // `1 + - 2`, copying `- 2`
    let mut exprs = Exprs::default();
    let one = literal(&mut exprs, b"1", 0);
    let two = literal(&mut exprs, b"2", 6);
    let negated = exprs.push(
        NodeKind::Prefix {
            op: PrefixOp::Minus,
            operand: two,
        },
        4..5,
    );
    let sum = exprs.push(
        NodeKind::Binary {
            op: Operator::Plus,
            left: one,
            right: negated,
        },
        2..3,
    );
    let copy = exprs.copy(negated);
    assert_eq!(copy, ExprId::from_index(5));
    assert_eq!(
        (exprs.node(copy).parent(), exprs.node(copy).position()),
        (None, 0)
    );
    assert_eq!(
        *exprs.node(copy).kind(),
        NodeKind::Prefix {
            op: PrefixOp::Minus,
            operand: ExprId::from_index(4),
        }
    );
    assert_eq!(exprs.node(copy).span(), 4..7);
    assert_eq!(exprs.check([sum, copy]), Ok(()));
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "adopted by one parent only")]
fn a_node_is_adopted_by_one_parent_only() {
    let mut exprs = Exprs::default();
    let one = literal(&mut exprs, b"1", 0);
    exprs.push(
        NodeKind::Prefix {
            op: PrefixOp::Minus,
            operand: one,
        },
        0..1,
    );
    exprs.push(
        NodeKind::Prefix {
            op: PrefixOp::Minus,
            operand: one,
        },
        0..1,
    );
}

/// Copies `expr` into `into` and answers the copy's root, an omitted argument
/// or item becoming an `Omitted` node at the parent's start. Recursive, so its
/// callers run it on a large stack.
pub(crate) fn lower(expr: &crate::ast::Expr, into: &mut Exprs) -> ExprId {
    use crate::ast::{CallTarget as Target, ExprKind};

    let at = expr.span.start;
    let list = |items: &[Option<crate::ast::Expr>], into: &mut Exprs, ids: &mut Vec<ExprId>| {
        for item in items {
            ids.push(match item {
                Some(item) => lower(item, into),
                None => into.push(NodeKind::Omitted, at..at),
            });
        }
    };
    let mut ids = Vec::new();
    let kind = match &expr.kind {
        ExprKind::Literal(bytes) => NodeKind::Literal(into.push_bytes(bytes)),
        ExprKind::Constant(id) => NodeKind::Constant(*id),
        ExprKind::Variable(id) => NodeKind::Variable(*id),
        ExprKind::Stem(id) => NodeKind::Stem(*id),
        ExprKind::Compound(id) => NodeKind::Compound(*id),
        ExprKind::DotVariable(id) => NodeKind::DotVariable(*id),
        ExprKind::ClassResolver { namespace, name } => NodeKind::ClassResolver {
            namespace: *namespace,
            name: *name,
        },
        ExprKind::VariableReference(inner) => match inner.kind {
            ExprKind::Variable(symbol) => NodeKind::VariableReference {
                symbol,
                stem: false,
            },
            ExprKind::Stem(symbol) => NodeKind::VariableReference { symbol, stem: true },
            _ => panic!("a reference names a variable or a stem"),
        },
        ExprKind::Prefix { op, operand } => NodeKind::Prefix {
            op: *op,
            operand: lower(operand, into),
        },
        ExprKind::Binary { op, left, right } => {
            let left = lower(left, into);
            NodeKind::Binary {
                op: *op,
                left,
                right: lower(right, into),
            }
        }
        ExprKind::Call { target, args } => {
            let target = match target {
                Target::Symbol(id) => CallTarget::Symbol(*id),
                Target::Literal(bytes) => CallTarget::Literal(into.push_bytes(bytes)),
            };
            list(args, into, &mut ids);
            NodeKind::Call {
                target,
                args: into.push_edges(&ids),
            }
        }
        ExprKind::QualifiedCall {
            namespace,
            name,
            args,
        } => {
            list(args, into, &mut ids);
            NodeKind::QualifiedCall {
                namespace: *namespace,
                name: *name,
                args: into.push_edges(&ids),
            }
        }
        ExprKind::Message {
            target,
            name,
            super_class,
            args,
            cascade,
        } => {
            ids.push(lower(target, into));
            if let Some(scope) = super_class {
                ids.push(lower(scope, into));
            }
            list(args, into, &mut ids);
            NodeKind::Message {
                name: name.clone(),
                cascade: *cascade,
                scoped: super_class.is_some(),
                children: into.push_edges(&ids),
            }
        }
        ExprKind::List(items) => {
            list(items, into, &mut ids);
            NodeKind::List(into.push_edges(&ids))
        }
        ExprKind::Logical(items) => {
            for item in items {
                ids.push(lower(item, into));
            }
            NodeKind::Logical(into.push_edges(&ids))
        }
    };
    into.push(kind, expr.span.clone())
}

/// Asserts that the arena renders `expr` as the tree does, and that the copy
/// is a valid arena.
fn assert_renders_alike(expr: &crate::ast::Expr, symbols: &crate::token::SymbolTable) {
    let mut exprs = Exprs::default();
    let root = lower(expr, &mut exprs);
    assert_eq!(exprs.check([root]), Ok(()));
    assert_eq!(exprs.shape(root, symbols), expr.shape(symbols));
}

#[test]
fn the_arena_renders_every_shape_the_tree_renders() {
    use crate::ast::InstructionKind;

    crate::with_parser_stack(|| {
        let texts = [
            "'a''b'",
            "abc",
            "3",
            "s.",
            "s.1.k",
            ".nil",
            "- \\ + a",
            "a b",
            "a||b",
            "a'q'",
            "f(1,,3)",
            "'F'(1)",
            "ns:f(,2)",
            "ns:cls",
            "a~m:super(1, , 2)",
            "a~~m",
            "a[1, 2]",
            "(1, , )",
            ">a",
            "<s.",
        ];
        for text in texts {
            let program = crate::parse_program(format!("x = {text}").into_bytes())
                .unwrap_or_else(|error| panic!("`x = {text}` parses: {error:?}"));
            let value = program
                .main
                .instructions
                .iter()
                .find_map(|instruction| match &instruction.kind {
                    InstructionKind::Assignment { value, .. } => Some(value),
                    _ => None,
                })
                .expect("the program assigns");
            assert_renders_alike(value, &program.symbols);
        }
        let program = crate::parse_program(b"if a, b then nop".to_vec()).expect("the IF parses");
        let condition = program
            .main
            .instructions
            .iter()
            .find_map(|instruction| match &instruction.kind {
                InstructionKind::If { condition, .. } => Some(condition),
                _ => None,
            })
            .expect("the program has an IF");
        assert_renders_alike(condition, &program.symbols);
    })
    .expect("the parser thread spawns");
}

#[test]
fn a_long_chain_and_a_deep_nesting_render_on_a_small_thread() {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(|| {
            let n = 200_000;
            let symbols = crate::token::SymbolTable::default();
            let plus = |left, right| NodeKind::Binary {
                op: Operator::Plus,
                left,
                right,
            };

            // `1 + 1 + ...`
            let mut exprs = Exprs::default();
            let mut chain = literal(&mut exprs, b"1", 0);
            for term in 1..n {
                let at = 4 * term;
                let one = literal(&mut exprs, b"1", at);
                chain = exprs.push(plus(chain, one), at - 2..at - 1);
            }
            assert_eq!(
                exprs.shape(chain, &symbols),
                "(+ ".repeat(n - 1) + "\"1\"" + &" \"1\")".repeat(n - 1)
            );

            // `1 + (1 + (...))`: the leaves, then the operators innermost
            // first, which is the post-order of a right nesting.
            let mut exprs = Exprs::default();
            let leaves: Vec<ExprId> = (0..n)
                .map(|term| literal(&mut exprs, b"1", 6 * term))
                .collect();
            let mut nest = leaves[n - 1];
            for term in (0..n - 1).rev() {
                let at = 6 * term + 2;
                nest = exprs.push(plus(leaves[term], nest), at..at + 1);
            }
            assert_eq!(
                exprs.shape(nest, &symbols),
                "(+ \"1\" ".repeat(n - 1) + "\"1\"" + &")".repeat(n - 1)
            );
        })
        .expect("a thread spawns")
        .join()
        .expect("the render thread finishes");
}
