//! This file stays `mod.rs`, not `gate_walk.rs`, because `tests/` is Cargo's
//! integration-test root: a bare `tests/gate_walk.rs` would be auto-discovered
//! as its own test binary in addition to being pulled in as a module by
//! `tiling.rs`, `variants.rs` and `roots.rs`, compiling this walk standalone
//! for no reason. Renaming it to `gate_walk.rs` would add that extra binary back.

// Each test binary compiles its own copy of this module, and neither binary
// uses every item: `tiling.rs` never lists the samples, `variants.rs` never
// walks children directly. The allowance is per-binary surplus in a shared
// test module, not a not-yet-called library item, so the gate criterion that
// every `allow(dead_code)` under `src/` names the task that deletes it does
// not apply here and the grep that enforces it does not look here.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use interrexx_parse::{
    Args, CodeBody, Directive, DirectiveKind, ExprRef, ExprView, Instruction, Program,
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

/// The expression a directive holds outside any body: only `::CONSTANT (expr)`
/// has one.
pub fn expr_of_directive(kind: &DirectiveKind) -> Option<ExprRef<'_>> {
    match kind {
        DirectiveKind::Constant(c) => c.value.expression(),
        DirectiveKind::Annotate(_)
        | DirectiveKind::Attribute(_)
        | DirectiveKind::Class(_)
        | DirectiveKind::Method(_)
        | DirectiveKind::Options(_)
        | DirectiveKind::Requires(_)
        | DirectiveKind::Resource(_)
        | DirectiveKind::Routine(_) => None,
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

/// Calls `visit` on every expression in the program, parents before children.
pub fn each_expr<'a>(p: &'a Program, visit: &mut impl FnMut(ExprRef<'a>)) {
    // An explicit stack rather than recursion through `children_of`, because
    // a left-leaning expression tree the width of one clause overflows the
    // stack here otherwise: `deep_nested_expr.rex`'s 3000-term chain aborted
    // this walk on a default 2 MiB thread.
    // Pushing a node's children in reverse and popping is what keeps this a
    // parents-before-children, left-to-right walk exactly like the recursive
    // version: the leftmost child is pushed last, so it is popped, and so
    // fully visited depth-first, before its next sibling.
    fn walk<'a>(root: ExprRef<'a>, visit: &mut impl FnMut(ExprRef<'a>)) {
        let mut stack = vec![root];
        while let Some(e) = stack.pop() {
            visit(e);
            let mut children: Vec<ExprRef<'a>> = Vec::new();
            children_of(e, &mut |child| children.push(child));
            stack.extend(children.into_iter().rev());
        }
    }
    let bodies = std::iter::once(&p.main).chain(
        p.directives
            .iter()
            .filter_map(|d| body_of_directive(&d.kind)),
    );
    for body in bodies {
        for i in &body.instructions {
            i.kind.for_each_root(&mut |e| walk(body.expr(e), visit));
        }
    }
    for d in &p.directives {
        if let Some(e) = expr_of_directive(&d.kind) {
            walk(e, visit);
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

/// Every `*.rex` file under `dir`, recursively, in a stable order.
pub fn rex_files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d)
            .unwrap_or_else(|e| panic!("cannot read directory {}: {e}", d.display()))
        {
            let path = entry.expect("readable directory entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rex") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// The corpus directory, `rust/corpus/lang/`.
pub fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/lang")
}

/// The samples directory at the repository root.
pub fn samples_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../third_party/ooRexx/samples")
}
