# Expressions as a term graph

This note designs the replacement of the boxed `Expr` tree (`interrexx-parse/src/ast.rs`) with a
flat arena of nodes in post-order, for #609 and #69. It is a design for review; nothing in it is
built yet.

## Problem

An expression is a tree of boxed nodes, and every pass over it recurses once per level. The parser
bounds real nesting (parentheses, argument lists) by `MAX_EXPR_DEPTH`, and its operator loop
(`binary_rest`) is iterative, so it builds a left-deep chain of any length. Every pass after it then
walks that chain one native frame per term:

* `push_native`, `native_shape` and `value_only` in `interrexx/src/ir/compile.rs`, through
  `push_arguments` and `push_scope` as well;
* `note` and `note_args` in `interrexx/src/plan.rs`;
* `guard_exposes` in `interrexx-parse/src/instruction.rs`, at parse time;
* the tree-walking evaluator: `eval`, `eval_node`, `eval_prefix`, `eval_arithmetic`, `eval_list`,
  `eval_call`, and `message_term` with `evaluate_message_arguments` in `dispatch.rs`;
* the image writer and reader (`interrexx-parse/src/image.rs`), whose `Box<T>` impl recurses into
  `Expr`;
* the derived `Debug`, `PartialEq` and `Clone` on `Expr` (#69);
* in tests, `Expr::shape`, `check_spans` and the differential test's own evaluator.

`Drop` on `Expr` and `calls_nothing` are already iterative, each with a worklist of its own.

The result is a defect against the language, not only a crash. `r[expr.index.nesting]` says that
many terms side by side at one level are not nesting and have no limit. Today a long flat chain
aborts the process in `push_native`, and a shorter one whose leftmost term is a function call
compiles to `Op::EvalExpr` (the call is past `NodePath`'s width), where the evaluator's
`MAX_EVAL_DEPTH` raises 11.1 for a program the rule says runs.

The owner has ruled out the patch that suggests itself: rewriting each recursive walker with an
explicit stack or worklist. Each such rewrite fixes one walker, the next walker written is
recursive again because recursion is the natural way to walk a tree, and the worklists encode the
control flow of the walk by hand in a different way each time. The aim here is a representation on
which the natural way to write a walk is a loop, so that a deep walk is not something a new pass
has to remember to avoid.

## Representation

Each code body owns one arena of expression nodes, `Exprs`, beside its instructions. An instruction
names its expressions by `ExprId`, a `u32` index into its body's arena, where it holds an `Expr`
today. A `Fragment` for `INTERPRET` is a body and has its own arena. The `::CONSTANT` value of a
directive, the one expression outside any code body, gets an arena of its own in the directive.

```rust
pub struct ExprId(NonZeroU32);       // Option<ExprId> stays four bytes

pub struct Exprs {
    nodes: Vec<Node>,                // post-order; ExprId(n) is nodes[n - 1]
    edges: Vec<ExprId>,              // the children of the nodes with a list of them
    bytes: Vec<u8>,                  // literal and call-target bytes, by range
}

pub struct Node {
    kind: NodeKind,
    span: Span,                      // byte range in the retained source, as u32 offsets
    first: ExprId,                   // the first node of this node's subtree
    parent: Option<ExprId>,          // None for the root of an expression slot
    position: u32,                   // this node's index among its parent's children
}

pub enum NodeKind {
    Literal(BytesRange),
    Constant(SymbolId), Variable(SymbolId), Stem(SymbolId), Compound(SymbolId),
    DotVariable(SymbolId),
    VariableReference { symbol: SymbolId, stem: bool },
    ClassResolver { namespace: SymbolId, name: SymbolId },
    Omitted,
    Prefix { op: PrefixOp, operand: ExprId },
    Binary { op: Operator, left: ExprId, right: ExprId },
    Call { target: CallTarget, args: EdgeRange },
    QualifiedCall { namespace: SymbolId, name: SymbolId, args: EdgeRange },
    Message { name: Selector, cascade: bool, scoped: bool, children: EdgeRange },
    List(EdgeRange),
    Logical(EdgeRange),
}
```

The field names and widths are a sketch; the implementation fixes them with a size assertion, as
`Op` has one. The points that matter:

* **Children are indices, and every child is a node.** A call's arguments, a list's items, a
  logical list's parts and a message's receiver, scope and arguments are a range of `edges`, in
  source order. An omitted argument or list item is an `Omitted` node, not a `None`: an omitted
  position has its own place in the evaluation order (it owes an empty `>A>` line in a call), and
  a node is what gives it one. The parser drops trailing omitted arguments as it does today; a
  list keeps them (`(1,)~size` is 2).
* **A message's children are one range**: the receiver first, then the `:scope` override where
  `scoped`, then the arguments. That is the order they are evaluated in.
* **`VariableReference` is a leaf.** Its operand is never evaluated, only named, so it is not a
  child; the symbol and whether it is a stem are the node's payload. `ClassResolver` is a leaf
  already.
* **Every node is addressable by its id.** An op that names a node today does it by instruction
  index, expression slot and `NodePath`, a 31-step route from the slot's root; past that width a
  node has no address and its whole slot falls back to `Op::EvalExpr`. An `ExprId` addresses any
  node of the body, so `NodePath`, the slot-and-path lookup in `run.rs` and the width fallback go.
* **The AST stays plain owned Rust data** (D13). The arena is a few vectors.

`Debug`, `PartialEq` and `Clone` are derived on `Exprs`, and over flat vectors they are loops. An
instruction's `Debug` prints ids; a test or a panic message that wants an expression's text uses a
renderer that is itself a loop (see below).

### Invariants

The arena of a body satisfies these after the body is parsed, and a validator checks all of them
(in debug builds after each body, and in a test over the whole corpus):

1. **Children before parents.** Every child's id is less than its parent's.
2. **A subtree is a contiguous range, in evaluation order.** The nodes of the subtree rooted at `n`
   are exactly `first(n)..=n`, and their order is the order in which evaluation completes them:
   post-order, children left to right.
3. **A tree, not a DAG.** Every node is the child of exactly one parent, or is the root of exactly
   one expression slot of one instruction. No node is unreachable.
4. **`parent` and `position` agree with the parents' child lists.**

Invariant 2 is what every walk below leans on. It makes "the subtree of `n`" a slice, so a
question about a subtree is a scan of that slice, and running a subtree is a loop over it.

## Construction

The parser emits nodes into the arena directly; no tree exists at any point. Each parse function
that returns an `Expr` today returns the `ExprId` of the root it pushed. The builder sets `first`,
`parent` and `position` and widens the span as it pushes a node, from children that already exist,
so it does constant work per child.

Post-order falls out of the recursive descent as written, because every node is built only after
the parse calls that return its children, and those calls follow the source left to right:

* `binary_rest` has its left operand complete before it reads an operator, parses the right
  operand (with `message_subterm` and a nested `binary_rest` of higher precedence), and only then
  builds the operator node: left range, right range, operator. An abuttal consumes no token but
  takes the same path. The nested `binary_rest` recurses once per rise in precedence, so its depth
  is bounded by the number of precedence levels, not by the expression.
* A message cascade (`cascade`, `message`, `collection_message`) has its target complete, then
  parses the `:scope` term and the arguments in that order, then builds the message node. So does
  the cascade arm of `binary_rest` for a target built by `variable_reference_term`.
* A call, a qualified call and a list parse their elements in order and then build the node.
* Parentheses return their inner expression unchanged and add no node, as today.

These places build out of that order today, and change:

* **`name op= expr`** (`instruction.rs`). Today the value is parsed first and the target's copy is
  built after it, as the left operand. The builder instead pushes the left operand's read before
  parsing the value, so that the read precedes the value in the range, which is the order it is
  evaluated in. The assignment target is a separate leaf, the root of its own slot.
* **`term op= expr`** for a message term. The left operand is a copy of the whole term, and the
  term is evaluated twice (once as the operand, once as the assignment's receiver and arguments).
  The copy is the term's range pushed again with its ids rebased, before the value is parsed: a
  loop over a slice.
* **Prefix operators.** `message_subterm` recurses once per prefix operator. With an arena it
  reads the run of prefix operators into a small vector, parses the operand once, and pushes one
  `Prefix` node per operator, innermost first. Whether a prefix chain still counts against the
  nesting limit is then a language question, not a stack one (see the open questions).

**Backtracking rewinds the arena.** A trial parse that is discarded leaves nodes behind:
`parse_variable_or_message_term`'s trial, `message_term` answering `None` for a plain call, the
message instruction falling back to keyword processing, and the other `trial_from` users in
`instruction.rs` and `instruction/parse.rs`. The builder has `mark()` and `rewind(mark)`, and each
discard rewinds to the mark taken before the trial. A missed rewind leaves an unreachable node,
which invariant 3 catches.

The parser's remaining recursion is real nesting: a parenthesis and an argument list re-enter
`full_subexpression` or `subexpression`. It stays recursive and stays bounded by `MAX_EXPR_DEPTH`
on the sized parser stack.

## Walking the arena

Every pass over an expression takes one of these forms, and none of them recurses:

* **A scan.** A question of the form "does the subtree of `n` contain a node of this kind" is
  `nodes[first(n)..=n].iter().any(...)`. `value_only`, `calls_nothing`, `guard_exposes` and the
  plan's `note` are scans.
* **A forward attribute.** A property computed from the children (whether a subtree compiles
  natively, whether a slot calls anything) is one loop over the range in index order, which sees
  every child before its parent.
* **A backward attribute.** A property handed down from the parent (whether an ancestor will
  evaluate this node itself, so its own ops are not emitted) is one loop over the range in reverse
  index order, which sees every parent before its children.
* **Evaluation with a value stack.** Running a subtree is a loop over its range in index order.
  Each node pops its operands' values from a stack and pushes its own. Work a parent does between
  two of its children (a call's `>A>` line after an argument, a message's scope checks after the
  scope term) runs when the loop reaches that child's root, which knows its `parent` and
  `position`. No node has work to do before its first child; a position that would need some is a
  node of its own, which is why an omitted argument is `Omitted`.

### A value stack is not a recursion stack

The last form keeps a stack, and the owner's ruling is against walkers that replace recursion
with a hand-kept stack, so the difference has to be real. It is: the two stacks hold different
things.

A worklist walker of a tree keeps **control state**: which children of which node are still to be
visited, and what to do with each result when it returns. That state is the shape of the
recursion, rebuilt by hand, and each walker rebuilds it differently: `calls_nothing` pushes right
before left so the left is popped first; `Drop` swaps children out for leaves.

The loop here keeps **no control state**. Where it is in the walk is the index `i` and nothing
else, because the order of the walk is the order of the slice; invariant 2 has already done the
work a worklist does. What the stack holds is **data**: the values computed and not yet consumed,
exactly the operands of the operators still to come. That is the stack an expression evaluator
needs whatever the representation, and the interpreter has it already: the evaluator roots its
intermediate values in the activity's temp frames (`push_temp`), and the compiler hands out
registers from a compile-time stack (`Registers`). For a left-deep chain the value stack holds two
entries however long the chain is; it grows only with right-nested operands, which are
parentheses and so bounded by the parser's limit, and it lives on the heap.

### Keeping recursion unnatural

An `ExprId` can still be passed to a function that calls itself on the children; nothing in Rust
forbids it. What the design changes is that nothing invites it:

* There is no owned tree to pattern-match into. A node's children are ids, and the API offers
  `range(n) -> &[Node]` and per-node operand ids, not a `children()` that hands out sub-nodes.
* Every derived trait and the image codec are over flat vectors, so no derive generates
  recursion.
* Every question the existing passes ask is answered by one of the loop forms above, and the
  module documentation says so, with an example of each.
* An end-to-end test runs a long flat chain through every pass on a small thread (see Testing). A
  new pass that recurses over expressions and is reached by ordinary programs fails it.

## Consumers

### IR compilation

`push_native` becomes a forward loop over the slot's range with a compile-time register stack. A
node's result lands in the register of its first operand, or in a fresh register for a node with
none; its other operands' registers are released after its op. This is the discipline
`push_native` follows today, stated for a stack: the left operand of a binary node lands in `dst`
and only the right takes a register, so a left-deep chain costs two registers however long it
runs. A prefix node writes into its operand's register, and a message into its receiver's. The
slot's first value takes the `dst` the caller passes, and every node writes into the register of
its first operand, so the root's value ends in `dst`.

One allocation changes: a call's result takes its first argument's register instead of one
reserved before the arguments. The arguments have been pushed onto the driver's argument stack by
the time the call op writes, so the register is free. Golden register numbers change for calls;
register counts do not grow.

The ops interleaved between children come from the child-root events: `PushArg` and
`TraceArgument` after each argument's root (an `Omitted` node pushes `ARG_OMITTED`), and
`SendScope` after a message's scope term. The trace echoes (`TraceLiteral`, `TraceRead`,
`TraceOperator`, `TracePrefix`, `TraceFunction`) already follow their node's op, which is the
post-order the loop visits in.

`native_shape` is a forward attribute and `value_only` a scan. The first change keeps every
compile decision as it is, so that the golden diff shows only addresses and call registers: a
call whose arguments are not all native still compiles to `Op::CallExpr`, and a backward
attribute marks its argument subtrees as not emitted. With every node addressable, the width
fallback to `Op::EvalExpr` disappears, so a slot that falls back today only because a call or a
send sits too deep compiles natively.

Ops that address a node carry its `ExprId` in place of `(index, slot, path)`: `CallExpr`,
`CallArgs`, `TraceFunction`, `Reference` and `ClassAt`. `Op` stays 16 bytes, with room to spare.

### The plan, and the other scans

`note` scans the slot's range in index order. A post-order and a pre-order walk meet the leaves in
the same order, the source order, so the plan's first-appearance slot numbering does not change;
the only non-leaf work in `note` (a call that names `TRACE`) sets a flag and does not depend on
order. `guard_exposes`, `value_only` and `calls_nothing` are scans of the same kind, and
`compares_as_text` and `share_tail_key` look only at a node and its direct children.

### The evaluator

The tree-walking evaluator stays: it is the engine for the slots that compile to `Op::EvalExpr`
and for the instructions whose step functions in `run.rs` evaluate their expressions themselves.
It becomes a loop over the slot's range, with its values rooted in the activity's temp frame as
they are today. Its trace output does not change, because it already emits each value event
post-order with the value in hand (`eval`'s `trace_intermediate` hook), and the loop visits nodes
in that post-order.

The evaluator's call and send paths take values instead of expressions. `invoke_call`,
`invoke_builtin_call`, `resolved_after_arguments` and `MessageTerm` in `dispatch.rs` evaluate
their argument expressions themselves today; in the loop the arguments are on the value stack when
the call node is reached. The work that today runs between children moves to the child-root
events: a call's `>A>` line after each argument, an omitted argument's empty one, a list item's
line, a message's scope checks after its scope term and before its first argument.

A `Logical` list never nests: the parser builds one only at the top of a condition, and a comma
inside parentheses builds a `List`. Its short circuit is a loop over its parts' ranges, each run
in turn and tested, which stops at the first false part.

`MAX_EVAL_DEPTH` and the per-node `enter_eval_node` go. One depth check stays at each entry to the
evaluator, because a call reached through the evaluator runs its callee on the same native stack,
and nested calls are nesting that `r[expr.index.nesting]` does allow a limit for.

### Golden rendering and the parser's test renderer

A golden renders a node address as the node's id instead of a path (`root.L.R`), and every golden
that holds one is re-recorded with the rewrite switch. `Expr::shape`, which renders a tree as an
S-expression for parser tests, becomes a loop with a stack of rendered strings: each node pops its
children's strings and pushes its own. The expected shapes in the parser's tests do not change.

### The differential test

`expr/differential.rs` evaluates generated expressions over the parsed tree and compares them with
values recorded from ooRexx. Its evaluator becomes a value-stack loop over the range. The recorded
values do not change, which shows that the arena keeps the shape and the evaluation order the tree
had.

### The image

The image writes a body's arena as its vectors, each a loop, and `FORMAT` moves on.

## What it enables

* **Every node has an address**, so ops can name any node, and the `NodePath` machinery and its
  width fallback go.
* **The evaluator can be retired later.** Once every node is addressable, only `QualifiedCall` and
  `Logical` lack native ops. Giving them ops and compiling the expressions the step functions in
  `run.rs` evaluate would leave `Op::EvalExpr` with nothing to do. That is not part of this change.
* **Shared subterms are representable later.** `t.k = t.k + 1` builds the tail key of `t.k` twice
  in the tree, and #598 and #604 recover the sharing at run time with a held key. In a graph, the
  read and the write could name one tail-key node. That breaks invariant 3, and with it the rule
  that a subtree is a range evaluated once in index order, so it needs its own design: a node kind
  for a value computed once and read twice, and the conditions under which the sharing is not
  observable, which are the ones #598 states. It is also not always allowed: `term op= expr`
  evaluates the term twice, and sharing it would change how many times its arguments run. Sharing
  is not part of this change.

## Library embedders on small stacks (#69)

The term graph closes the first half of #69: `Debug`, `PartialEq`, `Clone`, `Drop` and the image
codec are loops over flat vectors and use no stack per level, on any thread.

The second half stays. The parser recurses once per parenthesis or argument list, and
`MAX_EXPR_DEPTH` is sized for `PARSER_STACK_BYTES`. A caller that parses on a small thread can still
exhaust its stack on deep parentheses before the counter raises 11.1. `with_parser_stack` already
gives an embedder a sized thread. What is left is a choice for the owner: document that every parse
entry point needs `PARSER_STACK_BYTES` (or `with_parser_stack`), or let the caller pass a nesting
limit that fits its stack. Making the parser's nesting iterative as well would be a rewrite of the
recursive-descent grammar (D10), which this note does not propose.

## Blast radius

The set of sites was found by the compiler: `Expr`, its fields and `ExprKind` were marked
`#[deprecated]` for one `cargo check --workspace --all-targets` with default features, and the
warnings were counted per file. That finds every use that names the type, a field or a variant,
and misses a value whose type is only inferred and never touched through a field. The `jit`
feature was not built; `ir/drive/jit_host.rs` matches on `ExprKind` and is in the list from a
search.

* **`interrexx-parse`**: `ast.rs`, `expr.rs`, `instruction.rs`, `instruction/parse.rs`,
  `instruction/loops.rs`, `directive.rs`, `image.rs`, `lib.rs`; tests `ast/tests.rs`,
  `expr/tests.rs`, `expr/differential.rs`, `instruction/tests.rs`, `block/tests.rs`,
  `tests/gate_walk/mod.rs`, `tests/tiling.rs`, `tests/variants.rs`, `tests/deep.rs`,
  `tests/program.rs`; `examples/depth_probe.rs`.
* **`interrexx`, compiler and driver**: `ir/compile.rs`, `ir/drive.rs`, `ir/drive/jit_host.rs`,
  `ir.rs` (`NodePath`), `ir/golden.rs`; tests `ir/corpus_shape_tests.rs`, `ir/compile/tests.rs`,
  `ir/golden_tests.rs`, and the golden files.
* **`interrexx`, evaluator and steps**: `eval.rs`, `dispatch.rs`, `run.rs`, `run/call.rs`,
  `run/assignment.rs`, `run/loops.rs`, `run/use_args.rs`, `run/select.rs`, `run/settings.rs`,
  `run/condition/raise.rs`, `parse_template.rs`, `parse_template/trigger.rs`, `stem.rs`,
  `redirect.rs`, `command.rs`, `install.rs`, `install/constant_expression.rs`, `lib.rs`.
* **`interrexx`, plan**: `plan.rs`; tests `plan/tests.rs`.
* **`interrexx` tests**: `tests.rs`, `run/tests/message.rs`, `tests/coverage.rs`,
  `tests/variant_tags.rs`.

`run/loops.rs` builds an `Expr` at run time, a `Stem` or `Compound` leaf for a loop's control
variable, to hand to `assign_expr_target`. That function takes the assignment target's kind and
symbol instead, which is all it reads.

`Expr` and `ExprKind` are public in `interrexx-parse`, so the change breaks that crate's API.

## Landing it

Each step is a pull request that builds and passes on its own; they touch the same files and form
one stack.

1. **The arena in `interrexx-parse`.** `Exprs`, the builder with `mark` and `rewind`, the
   validator, the parser emitting nodes (the prefix loop and the `op=` reorderings included), the
   image format, the loop renderers for `Debug` and `shape`, the differential test as a loop, and
   a sentence in `architecture.md`'s Parsing section.
   To keep this step reviewable, it adds a transitional borrowed view, `ExprRef { exprs, id }`,
   whose `kind()` answers today's `ExprKind` shape with views for children. The interpreter's
   consumers change their types from `&Expr` to `ExprRef` and keep their recursion for now. This
   step closes the first half of #69.
2. **IR compilation as a loop.** `push_native`, `native_shape`, `value_only`, `push_arguments` and
   `push_scope` as loops; ops address nodes by `ExprId`; `NodePath` and the slot-and-path lookup
   go; goldens are re-recorded. The flat chain that aborts in `push_native` today runs.
3. **The scans.** `note`, `guard_exposes`, `calls_nothing`, and the remaining shallow matches.
4. **The evaluator as a loop.** The value-stack evaluator, the call and send paths taking values,
   `MAX_EVAL_DEPTH` replaced by the entry check. The chain whose leftmost term is a call runs.
5. **Removing the view.** `ExprRef::kind()` and its child views go, so that the only way into the
   arena is the range and per-node operand ids. The end-to-end deep-chain test lands here, if it
   has not landed earlier as an expected failure.

## Testing

Each test below fails on main today, which is the check that it covers the change.

* **The flat chain runs.** A program assigning a chain long enough to abort in `push_native` on
  the interpreter's stack prints its value, with and without `TRACE I`, and under `GUARD WHEN`.
* **A chain whose leftmost term is a call runs.** It raises 11.1 from `MAX_EVAL_DEPTH` today.
* **Every pass on a small thread.** A chain run through `run_program_on_stack` with a small stack
  (the one `tests/resources.rs` uses), long enough that any pass recursing once per term overflows
  it, parses, compiles, runs, traces and round-trips through the image. This is the test that
  catches a future recursive walker.
* **#69.** `Debug`, `PartialEq`, `Clone` and drop of a deep expression's arena on a default 2 MiB
  thread.
* **The invariants.** The validator over every program in the corpus and the library: ranges
  contiguous, every node reachable once, `parent` and `position` consistent. A mutation that skips
  a rewind or reorders the `op=` operand fails it.
* **Nothing else moves.** The differential test's recorded values, the parser's expected shapes,
  the trace oracle comparisons and the corpus outputs are unchanged; the goldens change only in
  node addresses and call registers.

Performance is checked by instruction counts against the pinned binary (`performance.md`): parsing
allocates one vector per body instead of a box per node, and compiling and evaluating loop instead
of recursing, so neither should regress, and the budget says whether it did.

## Risks and open questions

* **Prefix chains and the nesting limit.** Once prefix operators are parsed by a loop, a chain of
  them needs no stack, so whether it raises 11.1 is a rule to state, not a limit forced on the
  parser. `r[expr.index.nesting]` names only parentheses and calls. ooRexx raises 11.1 for a long
  enough prefix chain.
* **The order of a call's resolution.** The evaluator resolves a call's fixed target before its
  arguments and reports a failed resolution after them; the loop resolves at the call node, after
  the arguments. The compiled path (`Op::CallArgs`) already resolves after its arguments. The
  trace oracle comparisons and the corpus have to show the earlier lookup was not observable.
* **One arena per body, not per clause.** A body-wide arena lets an op name a node by one `u32`
  and keeps instructions small; it makes an instruction meaningless without its body, so a test
  that compares instructions compares bodies. Nothing found moves an instruction between bodies.
* **The transitional view.** Step 1 keeps the recursive consumers compiling through `ExprRef`,
  which makes recursion easy for the length of the stack. Step 5 removes it; until then the
  structural guarantee is not in place.
* **Golden and image churn.** Every golden with a node address and every recorded image change
  once.
* **#69's second half** needs the owner's choice between a documented minimum stack and a
  caller-supplied nesting limit.
