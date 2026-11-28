//! This file stays `mod.rs`, not `gate_walk.rs`, because `tests/` is Cargo's
//! integration-test root: a bare `tests/gate_walk.rs` would be auto-discovered
//! as its own test binary in addition to being pulled in as a module by
//! `tiling.rs` and `variants.rs`, compiling this walk standalone
//! for no reason. Renaming it to `gate_walk.rs` would add that extra binary back.

// Each test binary compiles its own copy of this module, and neither binary
// uses every item: `tiling.rs` never lists the samples, `variants.rs` never
// walks children directly. The allowance is per-binary surplus in a shared
// test module, not a not-yet-called library item, so the gate criterion that
// every `allow(dead_code)` under `src/` names the task that deletes it does
// not apply here and the grep that enforces it does not look here.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

#[path = "../rex_files/mod.rs"]
mod rex_files;
pub use rex_files::rex_files_under;

use interrexx_parse::{
    Args, CodeBody, ConstantValue, Directive, DirectiveKind, ExprRef, ExprView, Instruction, Node,
    Program,
};

/// Calls `f` on each direct child expression of `expr`, in source order. A
/// variable reference is a leaf: the variable it names is not a child.
pub fn children_of<'a>(expr: ExprRef<'a>, f: &mut impl FnMut(ExprRef<'a>)) {
    let each = |args: Args<'a>, f: &mut dyn FnMut(ExprRef<'a>)| {
        for arg in args.iter().flatten() {
            f(arg);
        }
    };
    match expr.kind() {
        ExprView::Literal(_)
        | ExprView::Constant(_)
        | ExprView::Variable(_)
        | ExprView::Stem(_)
        | ExprView::Compound(_)
        | ExprView::DotVariable(_)
        | ExprView::ClassResolver { .. }
        | ExprView::VariableReference { .. } => {}
        ExprView::Prefix { operand, .. } => f(operand),
        ExprView::Binary { left, right, .. } => {
            f(left);
            f(right);
        }
        ExprView::Call { args, .. } | ExprView::QualifiedCall { args, .. } => each(args, f),
        ExprView::Message {
            target,
            super_class,
            args,
            ..
        } => {
            f(target);
            if let Some(super_class) = super_class {
                f(super_class);
            }
            each(args, f);
        }
        ExprView::List(items) | ExprView::Logical(items) => each(items, f),
    }
}

/// The code body a directive carries, if it carries one.
pub fn body_of_directive(kind: &DirectiveKind) -> Option<&CodeBody> {
    match kind {
        DirectiveKind::Method(m) => m.body.as_deref(),
        DirectiveKind::Attribute(a) => a.body.as_deref(),
        DirectiveKind::Routine(r) => r.body.as_deref(),
        DirectiveKind::Annotate(_)
        | DirectiveKind::Class(_)
        | DirectiveKind::Constant(_)
        | DirectiveKind::Options(_)
        | DirectiveKind::Requires(_)
        | DirectiveKind::Resource(_) => None,
    }
}

/// Calls `visit` on every instruction of the program: the main body's, then
/// each directive body's, in source order.
pub fn each_instruction<'a>(p: &'a Program, visit: &mut impl FnMut(&'a Instruction)) {
    for i in &p.main.instructions {
        visit(i);
    }
    for d in &p.directives {
        if let Some(body) = body_of_directive(&d.kind) {
            for i in &body.instructions {
                visit(i);
            }
        }
    }
}

/// Calls `visit` on every directive of the program, in source order.
pub fn each_directive<'a>(p: &'a Program, visit: &mut impl FnMut(&'a Directive)) {
    for d in &p.directives {
        visit(d);
    }
}

/// Calls `visit` on every expression in the program: each root's subtree in
/// arena order, which puts a node's children before the node. A subtree is a
/// contiguous run of its arena, so this is a loop over the arena's nodes.
pub fn each_expr<'a>(p: &'a Program, visit: &mut impl FnMut(ExprRef<'a>)) {
    let bodies = std::iter::once(&p.main).chain(
        p.directives
            .iter()
            .filter_map(|d| body_of_directive(&d.kind)),
    );
    for body in bodies {
        for i in &body.instructions {
            i.kind
                .for_each_root(&mut |root| body.exprs.subtree(root).for_each(&mut *visit));
        }
    }
    for d in &p.directives {
        if let DirectiveKind::Constant(c) = &d.kind
            && let ConstantValue::Expression { exprs, root } = &c.value
        {
            exprs.subtree(*root).for_each(&mut *visit);
        }
    }
}

/// Calls `visit` on every node of every arena of the program, body by body,
/// each slot's subtree in evaluation order. The subtrees of a body's roots
/// cover its arena once, so this is a loop over each arena's nodes.
pub fn each_node<'a>(p: &'a Program, visit: &mut impl FnMut(&'a Node)) {
    let bodies = std::iter::once(&p.main).chain(
        p.directives
            .iter()
            .filter_map(|d| body_of_directive(&d.kind)),
    );
    for body in bodies {
        for i in &body.instructions {
            i.kind
                .for_each_root(&mut |root| body.exprs.range(root).iter().for_each(&mut *visit));
        }
    }
    for d in &p.directives {
        if let DirectiveKind::Constant(c) = &d.kind
            && let ConstantValue::Expression { exprs, root } = &c.value
        {
            exprs.range(*root).iter().for_each(&mut *visit);
        }
    }
}

/// The translation errors that the directives declared so far decide, rather
/// than the clause being parsed.
pub const DECLARATION_ERRORS: &[(u16, u16)] = &[
    (33, 1),
    (99, 901),
    (99, 902),
    (99, 903),
    (99, 906),
    (99, 931),
    (99, 932),
    (99, 942),
    (99, 945),
];

/// The corpus directory, `rust/corpus/lang/`.
pub fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/lang")
}

/// The samples directory at the repository root.
pub fn samples_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../third_party/ooRexx/samples")
}
