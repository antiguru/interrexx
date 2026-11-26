# Expressions as a term graph

This note designs the replacement of the boxed `Expr` tree (`interrexx-parse/src/ast.rs`) with a
flat arena of nodes in post-order, built by an operator-precedence parser and run only by the
compiled engine, for #609 and #69. It is a design for review; nothing in it is built yet.

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

### Rules this design serves

The owner has ruled on these points, and the design is built around them:

* **An expression has no size limit.** A long input takes long to process, and that is all: no
  length of chain, no run of prefix operators and no depth of parentheses or argument lists raises
  11.1. The parser reads parentheses and argument lists nested to any depth. The only nesting that
  keeps a limit is routine calls nested at run time (see [The nesting rule](#the-nesting-rule)).
  ooRexx raises 11.1 for a long enough prefix chain and for deep parentheses and argument lists;
  interrexx does not, under an approved Deviation.
* **The fixed widths of the compiled form are resource limits, not language rules.** Register
  indices stay `u16` and the arena's ids and offsets `u32`. An expression that needs more of
  either than the width holds raises Error 5, system resources exhausted, and so does a body that
  fills any other width of the compiled form (see IR compilation, which also proposes where each
  is raised). The bound is set by the implementation, as the depth of routine calls is set by the
  interpreter's stack, and no rule of the language puts it there, so it raises 5 and not 11.1.
* **There is one evaluator.** The tree-walking evaluator is retired, and every expression runs on
  the compiled ops, traced or not, in a program or in an `INTERPRET` string.
* **A malformed expression raises an error that describes a fault present in its text.** Which of
  several faults is reported, and so which code and subcode, need not be ooRexx's. This is the
  rule `decisions.md` already states for the checks of one operation, applied to the parser.

## Representation

Each code body owns one arena of expression nodes, `Exprs`, beside its instructions (one arena per
body, not per clause, by the owner's ruling). An instruction names its expressions by `ExprId`, a
`u32` index into its body's arena, where it holds an `Expr` today. A `Fragment` for `INTERPRET` is
a body and has its own arena. The `::CONSTANT` value of a directive, the one expression outside any
code body, gets an arena of its own in the directive.

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
  child; the symbol and whether it is a stem are the node's payload. The scans that collect the
  symbols an expression names read that payload, as they visit the child today: the plan's
  `note` gives the symbol its variable slot, and `guard_exposes` counts it among the variables a
  `GUARD ... WHEN` watches, so `GUARD ON WHEN check(>x)`, whose only variable is the reference,
  is accepted as it is today and not refused with 99.913. Neither reads the variable.
  `ClassResolver` is a leaf already.
* **Every node is addressable by its id.** An op that names a node today does it by instruction
  index, expression slot and `NodePath`, a 31-step route from the slot's root; past that width a
  node has no address and its whole slot falls back to `Op::EvalExpr`. An `ExprId` addresses any
  node of the body, so `NodePath`, the slot-and-path lookup in `run.rs` and the width fallback go.
* **The AST stays plain owned Rust data** (D13). The arena is a few vectors.

`Debug`, `PartialEq` and `Clone` are derived on `Exprs`, and over flat vectors they are loops. An
instruction's `Debug` prints ids; a test or a panic message that wants an expression's text uses a
renderer made of loops that writes each byte once (see
[Golden rendering](#golden-rendering-and-the-parsers-test-renderer)).

A body-wide arena lets an op name a node by one `u32` and keeps instructions small; it makes an
instruction meaningless without its body, so a test that compares instructions compares bodies.
Nothing found moves an instruction between bodies.

The `u32` ids and offsets are the arena's only fixed widths. A body whose nodes or source offsets
do not fit them raises Error 5 when it is parsed, with no subcode: the error table has none under
5 for this, and 5.900 is the user-defined one. That falls under the decision that behaviour when
memory runs out is best effort: an arena that large does not fit beside the interpreter in the
memory it runs under.

### Invariants

The arena of a body satisfies these after the body is parsed, and a validator checks all of them:
in debug builds after each body is parsed, in every build when a body is read from an image (see
[The image](#the-image)), and in a test over the whole corpus.

1. **Children before parents.** Every child's id is less than its parent's.
2. **A subtree is a contiguous range, in evaluation order.** The nodes of the subtree rooted at `n`
   are exactly `first(n)..=n`, and their order is the order in which evaluation completes them:
   post-order, children left to right.
3. **A tree, not a DAG.** Every node is the child of exactly one parent, or is the root of exactly
   one expression slot of one instruction. No node is unreachable.
4. **`parent` and `position` agree with the parents' child lists.**
5. **An `Omitted` node sits only at an argument or list-item position.** Its parent is a `Call`, a
   `QualifiedCall`, a `List`, or a `Message` at a child position after the receiver and the scope,
   and it is never the root of an expression slot.
6. **What a node indexes outside the arena is in range.** Its span runs forwards and ends within
   the source text, and each symbol id it holds is below the symbol table's length, since a
   message insert reads the span as a slice of the text and a name lookup reads the id as an
   index into the table.

Invariant 2 is what every walk below leans on. It makes "the subtree of `n`" a slice, so a
question about a subtree is a scan of that slice, and running a subtree is a loop over it. It also
gives the parser its operands without a stack of them: the right operand of a node just completed
is the last node pushed, and the left operand's root is the node just before the right operand's
`first`.

## Construction

The expression parser stops being recursive descent. It becomes an operator-precedence loop that
reads tokens left to right and pushes each node into the arena as soon as its children are there,
which is post-order. No tree exists at any point, and the parser recurses neither per operator nor
per nesting level. The clause and instruction grammar around it stays recursive descent (D10); its
recursion follows the block structure, which the block assembler already keeps on a control stack
of its own (`block.rs`), not the nesting of an expression.

### The loop

The parser alternates between two states: it expects an operand, or it has one and expects what
follows it. It keeps one stack, the pending stack, of two kinds of entry:

* **An operator** waiting for its right operand: a dyadic operator with its precedence, or a prefix
  operator, which binds tighter than any dyadic one.
* **A marker** for an open bracket: a parenthesis, an argument list (of a call, a qualified call or
  a message), or a `[` collection message. A marker records the terminator set in force inside it,
  the token that opened it (for error 36's inserts), the arena mark where its contents begin, how
  many items it holds so far and how many of them are not omitted, and, for a message, the
  receiver's id. The bottom of the stack is a marker for the whole expression, which holds the
  caller's terminator set and what the caller asked for (one expression, a comma list that builds
  a `List`, a logical list, a `WHEN` list, or one message term).

The terminator set in force is always the top marker's. A parenthesis resets it to `RIGHT`, an
argument list to `RIGHT`, a `[` to `SQRIGHT`, and a `CALL` instruction's argument list, which has
no closer, to `EOC`; closing the bracket pops the marker and the enclosing set is in force again.
This is today's rule, where each recursive call takes the set its caller passes.

**Expecting an operand.** A prefix `+`, `-` or `\` is pushed as an operator. A prefix `>` or `<`
reads the variable it names and pushes a `VariableReference` leaf. A literal or a symbol is a leaf,
unless a `(` abuts it, which makes it a call and pushes an argument-list marker; a symbol followed
by `:` reads the name after it and is a `ClassResolver` leaf or, with an abutted `(`, a qualified
call with an argument-list marker. A `(` pushes a parenthesis marker. A `,` or the marker's closer
where an argument or list item would start is an omitted position and pushes an `Omitted` node,
except a `)` right after a parenthesis's `(`, which is 35.1 as today. A terminator anywhere else
ends the expression, which is an error if an operator is waiting for its operand.

**Having an operand.** The message operators bind tighter than any operator, prefix ones included,
so they act as postfix operators on the operand just completed, whose root is the last node pushed:
`~` and `~~` read the message name and an optional `:scope` leaf, then either build the `Message`
node at once or, with an abutted `(`, push an argument-list marker that records the receiver; `[`
pushes a collection marker the same way. A message name is read under `EOC`, as today, except after
a variable reference (below). A dyadic operator pops and builds every waiting operator of greater
or equal precedence (so every level associates to the left, `**` included), then is pushed itself.
A blank is the blank operator, unless the next token that is not a blank terminates, in which case
it is skipped and ends the operand. A symbol, a literal or a `(` where an operator was expected is
the abuttal operator, which consumes no token. A `,` pops the operators down to the top marker and
counts an item. The marker's closer does the same, then pops the marker and builds its node: a
`Call`, `QualifiedCall` or `Message` whose children are the items since the marker's mark, a `List`
for a parenthesis that held a comma, or nothing for a parenthesis around one expression, which
returns its inner expression unchanged as today. An argument list drops its trailing `Omitted`
nodes before it builds the call: they are the last nodes pushed, so dropping them is a rewind. A
closer that is not the top marker's, a `\` as a dyadic operator and an assignment operator are
errors.

**After a variable reference** the postfix rule has an exception, which keeps today's parse. When
the operand just completed is a `VariableReference` leaf read in operand position (not one inside
a parenthesis that has closed) and a `~`, `~~` or `[` follows it, the loop first pops and builds
every prefix operator waiting above the nearest dyadic operator or marker, so the message goes to
the prefixed value. `- >a~value` is `(- >a)~value`: with `a = 3` it raises 97.1, because `VALUE`
is sent to `-3`, where `- (>a~value)` prints `-3`; and with `a = 'abc'`, `- >a[1]` raises 41.1
from the prefix before any `[]` is sent. The message names of that cascade, up to the first token
that is not a message operator, are read under the top marker's set rather than `EOC`, so
`do i = 1 to >a~for` and `do i = 1 to - >a~for` are 19.909 at translation, where
`do i = 1 to b~for` sends `FOR`. Today's parser does both because `>a` returns from
`message_subterm` without its cascade, through any prefix operators before it, and the cascade
is then read by `binary_rest` under the caller's terminators. No rule of the specification states
either; the loop keeps them so that no parse moves.

At the end of the clause, or at a terminator of the bottom marker's set, every operator is popped
and built, and a marker still open other than the bottom one is error 36 for its opener.

**Message term mode** starts at an atom, as today's trial starts in `subterm`. A prefix `+`, `-`
or `\` where the term would begin answers `None` before anything is pushed, so the clause falls
through to the command parser: a clause `-2~abs` is a command whose expression is `-(2~abs)`,
where `(-2)~abs` is a message instruction. A `>` or `<` there reads a reference, as `subterm`
does. The bottom marker takes no dyadic operator: anything but a message operator ends the term,
and a term that took none answers `None` after rewinding what it pushed, which is how a bare
`f(1)` clause stays a command.

Building a node does constant work per child: its operands are found from the arena by invariant
2, and the builder sets `first`, `parent` and `position` and widens the span from children that
already exist.

### Where the order changes

These places build out of order today and change, in the parser's callers in `instruction.rs`:

* **`name op= expr`**. Today the value is parsed first and the target's copy is built after it, as
  the left operand. The builder instead pushes the left operand's read before parsing the value, so
  that the read precedes the value in the range, which is the order it is evaluated in. The
  assignment target is a separate leaf, the root of its own slot.
* **`term op= expr`** for a message term. The left operand is a copy of the whole term, and the
  term is evaluated twice (once as the operand, once as the assignment's receiver and arguments).
  The copy is the term's range pushed again with its ids rebased, before the value is parsed: a
  loop over a slice.
* **Prefix operators.** Today `message_subterm` recurses once per prefix operator. In the loop
  each is a pending operator, built when its operand is complete, innermost first, or, before the
  cascade after a variable reference, when that cascade starts (above).

### Trial parses rewind the arena

A trial parse that is discarded leaves nodes behind: `parse_variable_or_message_term`'s trial,
`message_term` answering `None` for a plain call, the message instruction falling back to keyword
processing, and the other `trial_from` users in `instruction.rs` and `instruction/parse.rs`. The
builder has `mark()` and `rewind(mark)`, and each discard rewinds to the mark taken before the
trial, as it rewinds the token cursor today. The pending stack belongs to one parse call and is
dropped with it. A missed rewind leaves an unreachable node, which invariant 3 catches.

### An operator stack is not a walker

The owner has rejected explicit-stack walkers, and the parser's pending stack is a stack, kept by
hand, that grows with nesting. It is a different thing for three reasons.

* **It is the algorithm.** A walker's stack replays the recursion of a tree that already exists,
  and the recursion is the natural way to write it; the stack is a workaround. Operator-precedence
  parsing with an operator stack is the textbook way to read infix text with precedence, and the
  stack holds exactly what the grammar leaves open at a point of the text: the operators waiting
  for an operand and the brackets waiting for a closer. Nothing in it reconstructs a call stack.
* **It lives in one place.** There is one parser. A new pass never meets the pending stack and has
  no reason to write one, because what it walks is the arena the parser left, which every pass
  reads as a loop (below). The objection to walkers was that each new walker reintroduces the
  problem; a parser is not written again for each new question about an expression.
* **Its control state is the token position.** The loop's next step is decided by the next token
  and the top of the stack, not by a record of which children of which node remain to be visited.

What the design asks of the stack is that it lives on the heap, so that its depth is bounded by
memory, and that it holds operators and markers, never nodes, which go to the arena at once.

## Walking the arena

Every pass over an expression after the parser takes one of these forms, and none of them recurses:

* **A scan.** A question of the form "does the subtree of `n` contain a node of this kind" is
  `nodes[first(n)..=n].iter().any(...)`. `value_only`, `calls_nothing`, `guard_exposes` and the
  plan's `note` are scans.
* **A forward attribute.** A property computed from the children (whether a slot calls anything) is
  one loop over the range in index order, which sees every child before its parent.
* **A backward attribute.** A property handed down from the parent is one loop over the range in
  reverse index order, which sees every parent before its children.
* **Compilation with a register stack.** Compiling a subtree is a loop over its range in index
  order. Each node takes its operands' registers and leaves its value in one of them. Work a parent
  does between two of its children (a call's `PushArg` and `>A>` line after an argument, a
  message's scope checks after the scope term) is emitted when the loop reaches that child's root,
  which knows its `parent` and `position`. No node has work to do before its first child; a
  position that would need some is a node of its own, which is why an omitted argument is
  `Omitted`.

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
needs whatever the representation, and the compiler has it already: it hands out registers from a
compile-time stack (`Registers`). For a left-deep chain the stack holds two entries however long
the chain is; it grows only with values that wait while a nested operand runs, and its height is
bounded by the register width (see [IR compilation](#ir-compilation)).

### Keeping recursion unnatural

An `ExprId` can still be passed to a function that calls itself on the children; nothing in Rust
forbids it. What the design changes is that nothing invites it:

* There is no owned tree to pattern-match into. A node's children are ids, and the API offers
  `range(n) -> &[Node]` and per-node operand ids, not a `children()` that hands out sub-nodes.
* Every derived trait and the image codec are over flat vectors, so no derive generates
  recursion.
* Every question the existing passes ask is answered by one of the loop forms above, and the
  module documentation says so, with an example of each.
* An end-to-end test runs a long flat chain and a deep nesting through every pass on a small
  thread (see Testing). A new pass that recurses over expressions and is reached by ordinary
  programs fails it.

## Consumers

### IR compilation

`push_native` becomes a forward loop over the slot's range with the compile-time register stack. A
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

`native_shape` goes once every node kind has ops (see below); until then it is a forward
attribute, and `value_only` is a scan. The first change keeps every compile decision as it is, so
that the golden diff shows only addresses and call registers. With every node addressable, the
width fallback to `Op::EvalExpr` disappears, so a slot that falls back today only because a call
or a send sits too deep compiles natively.

Ops that address a node carry its `ExprId` in place of `(index, slot, path)`: `CallArgs`,
`TraceFunction`, `Reference` and `ClassAt`. `Op` stays 16 bytes, with room to spare.

**Deep nesting and the register width.** Register indices stay `u16`, so `Op` stays 16 bytes.
The register allocator is stack-disciplined (`Registers`' mark and release), so what the width
bounds is the number of values pending at once in one body, not the number a body computes. A
value pending for a nested operand holds its register for as long as that operand runs: the left
operand of `1 + (1 + (1 + ...))`, the receiver of `a~m(a~m(...))`, the first argument of
`f(1, f(1, ...))`, whose register the call's result takes. Only deep right nesting of that kind
reaches the width. A wide call does not, because each argument goes to the driver's argument
stack after its root and frees its register, and a flat chain does not, because it holds two
registers however long it runs.

A slot that needs more registers than the width holds raises Error 5, with no subcode, where today
it is `ChunkTooLarge` and a loud refusal. This is a resource limit set by the register width, as
the depth of routine calls is set by the interpreter's stack, and not a rule of the language, so it
raises 5 and not 11.1. If generated code ever meets it, values past the width could be parked on
the driver's argument stack and taken back before the op that consumes them, without widening
`Op`.

The same holds for every other width the compiler can fill. Each `ChunkTooLarge` refusal a
program can reach raises Error 5, with no subcode. The one refusal that stays a loud error is
`ValidOps`'s check for a register operand outside the chunk's register count: no program fills
that, and it firing means the compiler is wrong.

**Where Error 5 is raised.** A body
is compiled whole when it is first entered under a trace setting: `run_activation` takes the
body's chunk before it runs the first clause. A refusal turned into a condition at that point
would be raised before the body's own `SIGNAL ON SYNTAX` is set, and would refuse a body for a
clause it never reaches. So the widths split by what they bound, read from the refusals in
`ir.rs` and `ir/compile.rs`:

* **A width of one clause raises when that clause runs.** The clause compiles, in place of its
  ops, to one op that raises `SYNTAX 5` at the clause: it is traced as the clause is, the body's
  own `SIGNAL ON SYNTAX` traps it, and a clause that never runs never raises. Emitting a clause
  is a transaction over everything the compiler accumulates for the body, not only its ops: when
  the clause opens, the compiler marks the op stream, the jump patches, the register top, the
  length of the constant table, of the arithmetic hints and of the send and call sites, and the
  control-region stack. A refused clause takes each of them back to its mark, including changes
  to existing control regions and patches. The constant table also drops the interning entries
  of the constants it removes, so a later clause does not find an index past the table's end.
  Check body-wide widths against the committed state after determining whether the clause fits
  its own limits. If tentative entries would overflow a body-wide table, record the overflow
  and finish checking the clause's local limits before deciding which refusal to emit. The
  replacement raise op counts against the committed op stream. The compiler then goes on with
  the next clause, and a dead clause that was replaced cannot exhaust a body-wide table. These widths are the registers, the
  arguments of one call, the items of one `RAISE ARRAY` and the entries of one `PARSE` template.
  Registers count what is pending in the clause and what an enclosing `DO`, `LOOP` or
  `SELECT CASE` holds across its body, so a clause nested in enough loops can pass the width
  with little of its own. A `DO`, `LOOP` or `SELECT CASE` whose own header registers do not fit
  raises at its header clause, and the `END` of such a loop, which reads them, compiles to the
  same op. The width of a clause's expression slots, which today's ops carry, goes with
  `NodePath` in step 2, since an op names its node by `ExprId`.
* **A width of the whole body refuses the body before it runs.** These are the op stream, the
  constant table, and the arithmetic, send and call sites, each counted over the body. They raise
  `SYNTAX 5` when the body is entered, in the activation that enters it, before any clause of the
  body runs, as a syntax error in an external routine is raised at the `CALL` that reaches it:
  the caller's `SIGNAL ON SYNTAX` traps it. For an `INTERPRET` string it is raised at the
  `INTERPRET` instruction, where the running body's trap sees it, as it sees a syntax error in the
  string. A main program that passes one ends with Error 5 before its first clause, as one that
  fails to parse ends with its error. The arena's `u32` ids and offsets are of this kind, and
  raise when the body is parsed (see [Representation](#representation)). A body is compiled once
  per trace setting, and a traced chunk carries trace ops, so a body near a width can fit under
  one setting and not another; the bound is the implementation's, as above.

### The plan, and the other scans

`note` scans the slot's range in index order. A post-order and a pre-order walk meet the leaves in
the same order, the source order, so the plan's first-appearance slot numbering does not change;
the only non-leaf work in `note` (a call that names `TRACE`) sets a flag and does not depend on
order. `guard_exposes`, `value_only` and `calls_nothing` are scans of the same kind, and
`compares_as_text` and `share_tail_key` look only at a node and its direct children.

### Retiring the evaluator

The tree-walking evaluator (`eval.rs`'s `eval` and `eval_node`, the expression paths of
`dispatch.rs`) goes, and with it `Op::EvalExpr`, `Op::CallExpr` and `MAX_EVAL_DEPTH`. Every
expression runs on compiled ops. Traced code already does: a chunk is compiled under the trace
setting in force, and the trace echoes are ops. `INTERPRET` already does: it compiles its fragment
to a chunk of its own. What reaches the evaluator today, read from the compiler and the callers of
`Interp::eval`, is:

* **Slots that compile to `Op::EvalExpr`.** `native_shape` declines a `QualifiedCall` and a
  `Logical` list, and any call, send, variable reference or class lookup past `NodePath`'s width.
  The width goes with the arena. A `QualifiedCall` gets an op of its own, a call through a site
  that resolves in the namespace, its arguments pushed as `CallArgs` pushes them. A `Logical`
  list compiles each part followed by a test that jumps to the false exit, with the keyword the
  test reports, which is the short circuit `ConditionJump` already gives a single condition.
* **`Op::CallExpr`**, a call whose arguments are not all native: the driver goes back to the node
  and evaluates the arguments through `eval`. Once every node kind compiles, every call is
  `CallArgs`.
* **`Op::Message`**, a message instruction whose term does not compile: a `~~` cascade that is not
  an assignment, or one with a part that falls back. `exec_message` evaluates the term. A cascade
  instruction compiles as a send whose answer is dropped, and the op takes the receiver and
  arguments from registers.
* **The step functions of `Op::Exec`** that evaluate their own expressions: the `CALL` forms other
  than a named call, commands, `INTERPRET`, `GUARD`, `REPLY`, `FORWARD`, `RAISE`, `USE ARG`
  defaults, `NUMERIC`, `ADDRESS` with its redirections (`redirect.rs`), `TRACE` and `OPTIONS`,
  and the forms of `SELECT` and `GUARD` that are not compiled already. Each takes its values from
  registers the ops before it filled, as `Op::ExecValue`, `Op::ExecWith` and `Op::RaisePart` do
  for the instructions already moved, and evaluates nothing itself.
* **Message-term assignment** (`run/assignment.rs`): the receiver, scope and arguments of the
  target come from registers.
* **PARSE templates.** The template engine evaluates a parenthesised pattern itself
  (`eval_in_parse`) when it takes a template whole. The engine stops at each such pattern, as the
  compiled walk already stops at the first operand the stream computes, and the pattern's ops run
  before the engine resumes, so the pattern is evaluated when the template reaches it.
* **`::CONSTANT` values** (`install/constant_expression.rs`): the directive's arena compiles as a
  one-slot chunk, and both paths that run the expression today run that chunk. One runs it at
  install, below the level installing the package, for the constant's value. The other is
  `install.rs::run_constant_expression`: the expression's `Method`, captured and defined on a
  class with `.Class~define`, runs again when it is sent, with that send's receiver, scope and
  arguments, answers no result (91.999 when a result is wanted), and has no install-only
  `::CLASS` level below it. Each path keeps its own level, trace and failure site; only how the
  value is computed changes. The corpus programs `constant_expression_method_runs`, `_scopes`,
  `_trace` and `_fails` cover the second path and `class_constant_expression_*` and
  `directive_constant_expression_*` the first; the step that moves `::CONSTANT` keeps every one
  of them passing unchanged.

The compiler finds the list: the step that removes `Interp::eval` and `Interp::eval_node` fails to
build at every caller that is left. The work that today runs between children in the evaluator
moves to the child-root events above: a call's `>A>` line after each argument, an omitted
argument's empty one, a list item's line, a message's scope checks after its scope term and before
its first argument. Trace output does not change, because the evaluator already emits each value
event post-order with the value in hand (`eval`'s `trace_intermediate` hook), which is the order of
the trace ops.

Once nothing is left for it, `native_shape` answers true for every slot and goes, and so does
`push_value`'s fallback arm.

The differential test keeps its own evaluator as a model of the language that checks the parser,
not as an engine that runs programs.

### The JIT

The `jit` feature's code generator (`ir/drive/jit_host.rs`) reads the op stream, not expressions:
it reads a node only for an assignment target's kind, which is a leaf its op names. It follows the
compiler in three ways:

* It reads expression nodes only through the ids its ops carry, never by walking a subtree.
* Every op this design adds (the qualified call, the logical tests, the cascade send) gets an arm
  of its own in `jit_host.rs`'s step match: a native step, a `Step::Op` the driver runs from
  inside the compiled loop, or a refusal with the reason. The match's catch-all
  arm refuses the whole loop without saying so, so an op left to it costs speed silently; the pull
  request that adds an op names it there. The arms for `Op::EvalExpr` and `Op::CallExpr` go with
  those ops.
* The `jit` CI job builds the feature, so a type change the host misses fails there.

`INTERPRET`ed code is still never compiled by the JIT, as `decisions.md` records.

### Golden rendering and the parser's test renderer

A golden renders a node address as the node's id instead of a path (`root.L.R`), and every golden
that holds one is re-recorded with the rewrite switch.

`Expr::shape`, which renders a tree as an S-expression for parser tests, writes each byte of its
output once, into one buffer. Building each node's string from its children's strings would copy
a child's whole text again at every ancestor, which for a left-deep chain is quadratic. The
S-expression is in pre-order and the arena in post-order, so the renderer places the text by
offsets, in three loops of the forms above:

* a forward attribute gives each node the length of its rendering: its own opening, name and
  closing, a separator between children, and its children's lengths;
* a backward attribute gives each node its offset in the output: a parent, which the reverse loop
  reaches before its children, hands each child its offset in one sweep over its child list,
  using `position` to order them;
* one loop over the range writes each node's own bytes at its offset: the opening and name at
  its start, a separator after each child's end, the closing at its end.

The buffer is allocated once at the root's length, and the work is linear in the output. The text
renderer that tests and panic messages use (see [Representation](#representation)) is the same
three loops. The expected shapes in the parser's tests do not change, and the small-thread test
renders its deep chain and deep nesting with `shape` (see Testing).

### The differential test

`expr/differential.rs` evaluates generated expressions over the parsed tree and compares them with
values recorded from ooRexx. Its evaluator becomes a value-stack loop over the range. The recorded
values do not change, which shows that the arena and the loop parser keep the shape and the
evaluation order the tree had.

### The image

The image writes a body's arena as its vectors, each a loop, and `FORMAT` in
`interrexx-parse/src/image.rs`, the version of the `RXPARSED` layout, moves from 3 to 4. That is
the parser's image, which the interpreter's build writes for the built-in library and which
`program_from_image` reads for a library caller. `rexxc`'s compiled program file (`compiled.rs`)
holds the program's source, not a parsed image, has a format version of its own, and does not
change.

**An image of another format is refused, and its reader parses the text.** Both readers,
`program_from_image` and `program_from_static_image`, take the source text with the image, check
the image against it, and answer `None` for an image of another format, of other text, cut short
or carrying more; the module documents `None` as the caller's cue to parse the text. The
built-in library does that (`enter_library_program`), and its image is written by the same build
that reads it. A library caller holding a FORMAT 3 image and its text gets `None`, with no error
code, and parses the text; no entry point reads an image without its text, so there is no case
of old bytes alone to fall back from. No FORMAT 3 decoder is kept: the image is a cache of a
parse, rebuilt from the text it is checked against. A test pins this with a FORMAT 3 image of a
small program, written by today's writer and checked in beside its text: the new reader answers
`None` for it, not a misread program, and the text parses.

**A decoded arena is checked before anything uses it.** The decoder checks each id and range
against the vectors it has read, then runs the invariant validator on each body's arena and on a
`::CONSTANT` directive's. An arena whose ids are in range but whose roots, subtree ranges, or
parent or position links disagree is refused like any image that does not fit. The validator
reads through checked indices only, so such an arena cannot make it panic.
`program_from_image` decodes every body before it answers, so a library caller gets `None` for
an inconsistent arena and parses its text. The built-in image leaves each body encoded until it
is first read (`Body`), and that decode skips the validator, as it already skips the text
fingerprint for a built-in image: the image is the build's own output, written from a validated
parse, so a run pays nothing for a check whose failure would be a defect of the build. The test
that reads the built-in image through `program_from_image`
(`the_librarys_embedded_image_reads_back_as_it_parses`) decodes, and so validates, every body,
which is where such a defect fails: in CI, not in a program.

## What it enables

* **Every node has an address**, so ops can name any node, and the `NodePath` machinery and its
  width fallback go.
* **Shared subterms are representable later.** `t.k = t.k + 1` builds the tail key of `t.k` twice
  in the tree, and #598 and #604 recover the sharing at run time with a held key. In a graph, the
  read and the write could name one tail-key node. That breaks invariant 3, and with it the rule
  that a subtree is a range evaluated once in index order, so it needs its own design: a node kind
  for a value computed once and read twice, and the conditions under which the sharing is not
  observable, which are the ones #598 states. It is also not always allowed: `term op= expr`
  evaluates the term twice, and sharing it would change how many times its arguments run. Sharing
  is not part of this change.

## The nesting rule

Parentheses and argument lists nest without limit in the parser, as terms side by side and prefix
operators already do. With the loop parser no pass recurses per nesting level, so a limit would
have nothing left to protect. `MAX_EXPR_DEPTH` goes, and with it the error the parser raises for
depth.

What is left is the register width of [IR compilation](#ir-compilation): an expression nested to
the right deeply enough raises Error 5. That is a resource running out, like memory, and the
specification already leaves how much a program can ask for before 5 unspecified.

The proposed text of `r[expr.index.nesting]`:

> An expression has no nesting limit. Parentheses, argument lists, prefix operators and message
> sends nest inside one another to any depth, and any number of terms stand side by side at one
> level; a longer expression takes longer to read and to evaluate, and nothing in its text raises
> `SYNTAX 11.1`. An expression that keeps more values pending at once than the interpreter can
> hold raises `SYNTAX 5`, at a depth this book does not fix. Routine calls nested inside one
> another when the program runs have a limit that this book does not fix: the call that goes past
> it raises `SYNTAX 11.1`. Calls written inside one another's arguments are not nested in this
> sense, because each inner call returns before the outer one is made. Parentheses nested 100,000
> deep around one term are read and evaluated, and calls nested 1,000,000 deep raise the error.

Its example keeps the `deep` routine and changes the `INTERPRET` line to show a value instead of
11.1. The same change touches:

* `spec/src/conditions/errors.md`'s system-fault list, where 11 becomes calls nested beyond the
  limit, and the note on 5, which names an expression that keeps too many values pending among
  what a program can ask for;
* a new Deviation in `spec/src/deviations.md`, approved by the owner: ooRexx raises 11.1 for
  parentheses, argument lists or prefix operators nested past its own limit, and interrexx reads
  them, under the ruling that an expression has no size limit;
* the 11.1 row of `corpus/errors/parse-errors.tsv`, which ooRexx refuses and interrexx accepts, and
  which `tests/errors.rs` takes as an accepted program under that Deviation;
* `architecture.md`: its Parsing section, which says the parser has a nesting limit, and the
  `D10` row, which becomes a hand-written recursive-descent parser for clauses and an
  operator-precedence parser for expressions, with the reason that neither recurses per nesting
  level of an expression.

## Library embedders on small stacks (#69)

#69 asks for two things: `Debug`, `PartialEq` and `Clone` that do not overflow on a deep tree, and
a parser that does not exhaust a small caller's stack before its depth counter raises 11.1.

The arena closes the first: `Debug`, `PartialEq`, `Clone`, `Drop` and the image codec are loops
over flat vectors and use no stack per level, on any thread.

The loop parser closes the second by removing its cause. The parser uses no native stack per
nesting level, so a caller on a small thread parses any expression its memory holds, and there is
no depth counter left to be reached or not. The choice #69 left open, between documenting a
minimum stack for every parse entry point and letting the caller pass a nesting limit that fits its
stack, does not arise: there is neither a stack requirement nor a limit to choose.
`with_parser_stack` and `PARSER_STACK_BYTES` go with `MAX_EXPR_DEPTH`, and the crate's module
documentation stops asking for a sized thread.

What is left is outside the parser crate. Running a program nests native frames for what is
nested at run time, routine calls among them, and that depth has the interpreter's own limit and
stack probe; it is the run-time limit the nesting rule keeps, not an expression question.

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
  `tests/program.rs`, `tests/errors.rs`; `examples/depth_probe.rs`.
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
* **Documentation and specification**: `architecture.md`, `spec/src/expressions/index.md`,
  `spec/src/conditions/errors.md`, `spec/src/deviations.md`.

`run/loops.rs` builds an `Expr` at run time, a `Stem` or `Compound` leaf for a loop's control
variable, to hand to `assign_expr_target`. That function takes the assignment target's kind and
symbol instead, which is all it reads.

`Expr`, `ExprKind`, `MAX_EXPR_DEPTH`, `PARSER_STACK_BYTES` and `with_parser_stack` are public in
`interrexx-parse`, so the change breaks that crate's API.

## Landing it

Each step is a pull request that builds and passes on its own; they touch the same files and form
one stack.

1. **The arena in `interrexx-parse`.** `Exprs`, the builder with `mark` and `rewind`, the
   validator, the recursive-descent parser emitting nodes in post-order (which falls out of
   recursive descent, since every node is built after the calls that return its children), the
   prefix operators read as a run instead of by recursion (built before the cascade after a
   variable reference, as today), the `op=` reorderings, the image format with its FORMAT 3
   fixture and the validator run at decode, the renderers for `Debug` and `shape` as loops that
   write each byte once, the differential test as a loop, and a sentence in `architecture.md`'s
   Parsing section. To keep this step reviewable, it adds a
   transitional borrowed view, `ExprRef { exprs, id }`, whose `kind()` answers today's `ExprKind`
   shape with views for children; the interpreter's consumers change their types from `&Expr` to
   `ExprRef` and keep their recursion for now. The owner has accepted this view for the steps up to
   its removal. This step closes the first half of #69, and changes no error code.
2. **IR compilation as a loop.** `push_native`, `native_shape`, `value_only`, `push_arguments` and
   `push_scope` as loops; ops address nodes by `ExprId`; `NodePath` and the slot-and-path lookup
   go; goldens are re-recorded. The flat chain that aborts in `push_native` today runs, and so does
   the chain whose leftmost term is a call, which no longer falls back for width. Register
   exhaustion raises Error 5 from this step on, because a slot that ran on the evaluator for width
   now compiles and can pass the register width (see Risks), and so does every other width
   refusal of the compiler, so that no step leaves a loud refusal a program can reach. A width of
   one clause raises when that clause runs and a width of the body when the body is entered (see
   [IR compilation](#ir-compilation)).
3. **The scans.** `note`, `guard_exposes`, `calls_nothing`, and the remaining shallow matches;
   `note` and `guard_exposes` read a variable reference's symbol from its payload.
4. **Retiring the evaluator.** Ops for `QualifiedCall`, `Logical` lists and cascade message
   instructions; the step functions, message-term assignment, PARSE template patterns and
   `::CONSTANT` values, on both of their paths, taking their values from registers; the
   call-resolution witnesses of Risks carried to the compiled path; `Op::EvalExpr`, `Op::CallExpr`,
   `eval.rs`'s tree walker and `MAX_EVAL_DEPTH` removed; the JIT host's arms. If it is too large
   to review as one, it splits where it stays green: first the new ops, which leave
   `Op::EvalExpr` with no slot to compile, then the step functions, which leave `eval` with no
   caller.
5. **Removing the view.** `ExprRef::kind()` and its child views go, so that the only way into the
   arena is the range and per-node operand ids. The end-to-end flat-chain test lands here, if it
   has not landed earlier as an expected failure.
6. **The operator-precedence parser.** `expr.rs`'s recursive descent is replaced by the loop. It
   keeps counting nesting against `MAX_EXPR_DEPTH` on the pending stack, so that no program's
   result moves in this step except where a malformed expression now reports another fault that
   is present (see Testing). The loop keeps the cascade after a variable reference and the
   atom-only start of message term mode, which the tests under What the rewrite keeps pin.
   `D10`'s row is amended.
7. **No nesting limit.** `MAX_EXPR_DEPTH`, `PARSER_STACK_BYTES` and `with_parser_stack` go;
   arena overflow in the parser raises Error 5; `r[expr.index.nesting]`, the error list, the
   Deviation and the corpus row change as above; the deep-nesting test and the program-level
   Error 5 test land (see Testing). This step closes #69.

The parser comes after the view is gone rather than with the arena. The arena step then shows
that nothing moved, with the parser whose answers every test was written against, and the parser
step lands on a settled arena with no tree-shaped view for it to lean on; its only visible change
is which fault a malformed expression reports, and nothing else in the stack changes error codes,
so a moved code points at it. The cost is that step 1 adapts the recursive-descent functions to
emit nodes and step 6 replaces them; the leaf-reading functions (symbols, literals, message names,
scopes, references, qualified symbols) carry over. The limit goes last because it needs the loop
parser.

## Testing

Each test below fails on main today, which is the check that it covers the change. The tests
under [What the rewrite keeps](#what-the-rewrite-keeps) pass on main instead, and each fails
under a named mistake of the new code.

* **The flat chain runs.** A program assigning a chain long enough to abort in `push_native` on
  the interpreter's stack prints its value, with and without `TRACE I`, and under `GUARD WHEN`.
* **A chain whose leftmost term is a call runs.** It raises 11.1 from `MAX_EVAL_DEPTH` today.
* **A long prefix chain runs.** A chain long enough to overflow the parser's own stack, which
  recurses once per prefix operator today, parses, compiles and prints its value.
* **Deep nesting runs.** Parentheses around one term, `f(f(f(...)))`, and a mix of parentheses,
  call and message argument lists that keeps fewer values pending than the register width, each
  nested deeper than `MAX_EXPR_DEPTH`, parse, compile, run and trace, and print their value. Today
  each raises 11.1 from the parser.
* **Register exhaustion raises 5.** In step 2, a compiler unit test pins the change itself: a
  clause whose registers today make the body's refusal a loud error compiles to the op that raises
  a `SYNTAX` condition with code 5 and no subcode at that clause. A program test in the same step
  nests `1 + a~m(1 + a~m(...))`, which keeps two values pending per level, deep enough to pass the
  width while staying under `MAX_EXPR_DEPTH`; it traps the error with `SIGNAL ON SYNTAX` and goes
  on. Today it prints its value, because the slot runs on the evaluator past `NodePath`'s width, so
  the test pins the move from a value to Error 5. In step 7, `1 + (1 + (...))`, `a~m(a~m(...))` and
  `f(1, f(1, ...))`, which keep one value pending per level and so pass the width only past
  `MAX_EXPR_DEPTH`, raise Error 5 the same way; today each raises 11.1 from the parser.
* **Every compiler width raises 5.** In step 2, each `ChunkTooLarge` reason a program can reach
  raises a `SYNTAX` condition with code 5, built from an oversized input where that is cheap and
  otherwise with the width lowered under `cfg(test)`; the `ValidOps` operand check stays loud.
* **Where 5 is raised.** In step 2, with the widths lowered under `cfg(test)` where an oversized
  input is not cheap, and each failing on main, where the refusal is a loud error:
  * a clause past a width of one clause, after `SIGNAL ON SYNTAX` in the same body, is trapped
    with code 5 and `SIGL` at that clause's line, and the clauses before it have run;
  * the same clause behind `if 0 then` or after `exit` is never reached, and the body runs to its
    end without a condition;
  * with a width of one clause and a width of the whole body both lowered, a refused clause that
    interned constants and registered arithmetic, send and call sites before it ran out, followed
    by a valid clause that needs one more entry in each of those tables, compiles and runs the
    valid clause; it fails if the rollback leaves any of the refused clause's entries behind, and
    a compiler unit test asserts that each table's length after the refused clause equals its
    length before it;
  * an `INTERPRET`ed string holding such a clause raises at that clause, and one past a width of
    the whole body raises at the `INTERPRET` instruction, each trapped by the running body;
  * a routine past a width of the whole body, called from a caller that has set
    `SIGNAL ON SYNTAX`, is trapped by the caller with code 5, and the routine's first clause, a
    `SAY`, prints nothing.
* **Arena overflow raises 5.** The builder, with its id width lowered under `cfg(test)`, raises
  Error 5 when a body's nodes pass it, instead of panicking or wrapping.
* **Every pass on a small thread.** A flat chain and a deep nesting, run through
  `run_program_on_stack` with a small stack (the one `tests/resources.rs` uses), long and deep
  enough that any pass recursing once per term or per level overflows it, parse, compile, run,
  trace, render with `shape` and round-trip through the image. This is the test that catches a
  future recursive pass, and the quadratic renderer it replaces is the reason the chain is
  rendered too.
* **#69.** Parsing, `Debug`, `PartialEq`, `Clone` and drop of a deep expression's arena on a
  default 2 MiB thread, with no sized thread.
* **The invariants.** The validator over every program in the corpus and the library: ranges
  contiguous, every node reachable once, `parent` and `position` consistent. A mutation that skips
  a rewind or reorders the `op=` operand fails it.
* **The image boundary.** A checked-in FORMAT 3 image and its text: the new reader answers
  `None` and the text parses. An image of a parsed program edited so that every id stays in range
  but the arena is inconsistent (a `parent` link pointed elsewhere, a `first` moved, a root left
  out of its slot) is refused by `program_from_image`, one edit per case.
* **Nothing else moves.** The differential test's recorded values, the parser's expected shapes,
  the trace oracle comparisons and the corpus outputs are unchanged; the goldens change only in
  node addresses and call registers.

### What the rewrite keeps

These pin today's behaviour where a natural reading of the new design would move it. Each passes
on main and fails under the mistake named with it.

* **The cascade after a variable reference.** With `a = 3`, `say - >a~value` raises 97.1 and
  `say - (>a~value)` prints `-3`; with `a = 'abc'`, `say - >a[1]` raises 41.1;
  `do i = 1 to - >a~for` and `do i = 1 to >a~for` are 19.909 at translation, and
  `do i = 1 to b~for` sends `FOR`. A parser shape test pins `(- >a)~value` and `(- >a)[1]`. They
  fail on a loop that applies the postfix rule to a reference as to any operand. The spec states
  this parse as a rule of its own in `spec/src/expressions/terms.md`, with these cases as its
  examples, so the spec run checks it too.
* **Message term mode starts at an atom.** A clause `-2~abs` is a command and `(-2)~abs` a
  message instruction: an instruction-kind test pins each, and a golden pins their ops (a value
  send, the prefix and `ExecWith` for the first; the prefix and a clause send for the second).
  They fail on a message term mode that accepts a prefix operator before its term.
* **A reference names a variable to the scans.** A method that exposes `x` and runs
  `GUARD ON WHEN check(>x)`, where `check` reads the value through the reference, is accepted and
  passes the guard. It fails with 99.913 on scans that read only variable and stem nodes.
* **`::CONSTANT` on both paths.** The `constant_expression_*`, `class_constant_expression_*` and
  `directive_constant_expression_*` corpus programs, unchanged. They fail if the
  `run_constant_expression` path is dropped with `Interp::eval` or runs with the install level.
* **The order of a call's resolution.** The cases listed under Risks, in the shapes that run on
  each engine today. They fail on a compiled call that resolves its target before its arguments.

### Malformed expressions

The differential test covers well-formed expressions: their shape and their value. What a
malformed one raises is pinned today in several places, each against one code and subcode:

* `interrexx-parse/tests/errors.rs` over `corpus/errors/parse-errors.tsv`, whose translation rows
  must reproduce ooRexx's code, subcode and line, with two named exceptions (the label colon and the
  `DO` sub-keyword rules). The test
  `every_message_the_parser_names_is_raised_by_a_corpus_row_or_recorded_as_unreached` counts a
  message as raised only where the row's code matches.
* The `error` and `logical_error` cases in `expr/tests.rs`, and the error cases of
  `instruction/tests.rs` and `clause/tests.rs` whose fault is in an expression.
* Recorded interpreter runs: `tests/ir_recorded_cases/parse-errors-main`, `tests/rexxc/cases` and
  `tests/rexx_command/cases`, which hold interrexx's own answers and re-record with their rewrite
  switch.
* Oracle comparisons that raise a parse error: the specification's examples under
  `REXX_SPEC_ORACLE`, the corpus programs that `INTERPRET` malformed strings, and ooTest cases
  that check a syntax error's code.

Under the rule that a reported fault must be present, not that it must be ooRexx's, these change
as follows, in step 6:

* **An old-against-new comparison.** Before the parser is replaced, a seeded generator mutates the
  differential corpus's expressions into malformed ones (a token dropped, doubled or moved, a
  stray `)`, `]`, `,`, `~`, `:` or operator inserted, the text cut short) and records the code and
  subcode the recursive-descent parser raises for each. The file is data and stays after that
  parser is gone. The test parses each input with the loop parser and passes where the code is the
  recorded one, or where it differs and the new fault is shown present: for a fault that is local
  to a token (a bracket without its partner, a `~` or `:` without a name, a `>` or `<` without a
  variable, a prefix or dyadic operator without its term, a `\` or an assignment operator used as
  a dyadic operator), a token-level witness over the clause checks it. A difference the witnesses
  cannot settle, such as one 35.1 against another code, is listed in a reviewed file with the
  fault the new code names; the test fails on a difference that is neither witnessed nor listed,
  so no code moves without someone reading it.
* **The parse-error corpus** applies the same acceptance to its translation rows whose fault is in
  an expression: ooRexx's code, or another whose fault a witness or the reviewed list shows
  present. The test that every message the parser names is raised counts those rows too. The line
  stays exact: errors are reported against the clause, which does not move.
* **The unit tests** keep their codes where the text has one fault, and those whose text has
  several are rewritten to one fault each, so that a unit test pins a rule of the grammar and not
  a choice between faults.
* **The recorded runs** are re-recorded and the diff read.
* **The oracle comparisons**: a specification example whose text has more than one fault is
  rewritten to have one, since an example shows a rule; a corpus or ooTest case whose code moves
  is listed with the fault the new code names, as the expected failures of the oracle comparisons
  are.

Performance is checked by instruction counts against the pinned binary (`performance.md`): parsing
allocates one vector per body instead of a box per node and loops instead of recursing, and
compiling loops instead of recursing, so neither should regress, and the budget says whether it
did. Retiring the evaluator moves the slots that ran on it onto ops, which the budget measures the
same way.

## Risks

* **The order of a call's resolution.** The evaluator resolves a call's fixed target (a label or
  a builtin) before its arguments and reports a failed resolution after them; the compiled path
  (`Op::CallArgs`) resolves after its arguments, and once the evaluator is retired that is the
  only order. The routine search that is not fixed runs after the arguments on both paths today,
  so an argument that adds the routine to the package is seen. #994 moves a qualified call's
  namespace lookup after its arguments on both paths. The corpus alone does not show that the
  earlier lookup was unobservable, so the tests carry the witnesses explicitly, each in a slot
  that compiles natively today and in one that runs on the evaluator (a call nested past
  `NodePath`'s width, or traced with an argument that falls back), and step 4 keeps every one with
  its expected output once both shapes compile:
  * #994's `qualified_calls_resolve_after_arguments_in_the_driver_and_walker`: an argument that
    rebinds the namespace reaches the new package, an argument that creates the namespace is
    seen, and an argument that raises takes precedence over a namespace or a routine that does
    not resolve (42.3 rather than 98.987 or 43.902), with each argument's side effect seen before
    a failed resolution is reported;
  * for a plain call, an argument that adds the routine through the package is seen, and an
    argument that raises takes precedence over a target that does not resolve (42.3 rather than
    43.1).

  If #994 has not landed when step 4 does, the qualified call's op this design adds takes #994's
  order, and its test lands with that step.
* **Error codes for malformed expressions move.** A loop parser meets the faults of a malformed
  expression in another order than recursive descent, so for text with more than one fault it can
  report another of them. That is allowed; what must hold is that the reported fault is present,
  which the comparison above checks and the reviewed list makes visible. An expression with one
  fault should report the code it reports today; a difference there is a defect of the new parser.
* **A program that runs today raises 5.** A slot that passes the register width while staying
  under `MAX_EXPR_DEPTH`, such as `1 + a~m(1 + a~m(...))` nested deep, runs today because it
  falls back to the evaluator for `NodePath`'s width. From step 2 it compiles and raises Error 5.
  That follows from keeping registers `u16` with no parking, and the test above pins it.
* **Golden and image churn.** Every golden with a node address and every recorded image change
  once.
* **The view lives for several steps.** It keeps the recursive consumers compiling, which makes
  recursion easy until step 5 removes it; until then the structural guarantee is not in place.
