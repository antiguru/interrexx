//! The instruction and directive nodes. An instruction names its
//! expressions by [`ExprId`] in the arena of the code body that holds it.

use std::collections::BTreeMap;
use std::ops::Range;

use crate::exprs::{Args, ExprId, ExprRef, Exprs, Inconsistent};
use crate::token::SymbolId;

/// The prefix operators.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PrefixOp {
    Plus,
    Minus,
    Not,
}

impl PrefixOp {
    /// The canonical source spelling, which is what a `>P>` trace line carries
    /// as its tag.
    pub fn spelling(self) -> &'static str {
        match self {
            PrefixOp::Plus => "+",
            PrefixOp::Minus => "-",
            PrefixOp::Not => "\\",
        }
    }
}

/// One tail element of a compound variable, borrowed from the interned name.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Tail<'a> {
    /// A piece that is empty or starts with a digit, so it can never be a
    /// variable name and stands for itself.
    Constant(&'a str),
    /// Anything else: a simple variable whose value supplies this piece.
    Variable(&'a str),
}

/// Splits a compound variable's name into its stem and its tail pieces.
pub fn compound_parts(name: &str) -> (&str, Vec<Tail<'_>>) {
    let dot = name
        .find('.')
        .expect("a compound symbol holds at least one period");
    let (stem, rest) = name.split_at(dot + 1);
    let tails = rest
        .split('.')
        .map(|piece| {
            if piece.is_empty() || piece.starts_with(|c: char| c.is_ascii_digit()) {
                Tail::Constant(piece)
            } else {
                Tail::Variable(piece)
            }
        })
        .collect();
    (stem, tails)
}

/// One code body: an instruction chain and the labels declared in it.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct CodeBody {
    /// In source order, which is also the execution chain: control falls from
    /// each instruction to the next index unless a jump target says
    /// otherwise. Ends where the first `::` directive clause begins or the
    /// source ends -- `translate_block` stops there and leaves the cursor
    /// sitting on it for the caller, so an instruction after that point
    /// belongs to the next code body, never this one.
    pub instructions: Vec<Instruction>,
    /// Keyed by the label token's VALUE, not by `SymbolId`: upcased for a
    /// symbol label, verbatim for a literal one. `Box<[u8]>` rather than
    /// `Box<str>`, because a literal label is not required to be valid UTF-8
    /// any more than any other literal is -- measured, a label spelled with a
    /// raw non-UTF-8 byte is a legal `SIGNAL VALUE` target under
    /// `build/bin/rexx`. Interning the key would be wrong in both directions;
    /// The first occurrence of a duplicated
    /// label wins -- measured, two labels spelled `a:` in one program is
    /// accepted and `signal a` reaches the first -- so this is built with
    /// "insert if absent", never an unconditional overwrite.
    pub labels: BTreeMap<Box<[u8]>, usize>,
    /// The expressions of every instruction above, each subtree a contiguous
    /// range in evaluation order.
    pub exprs: Exprs,
}

impl CodeBody {
    /// The view of `expr`, an expression slot of one of this body's
    /// instructions.
    pub fn expr(&self, expr: ExprId) -> ExprRef<'_> {
        self.exprs.get(expr)
    }

    /// The view of `list`, an argument or item list of one of this body's
    /// instructions, where `None` is omitted.
    pub fn args<'a>(&'a self, list: &'a [Option<ExprId>]) -> Args<'a> {
        Args::slots(&self.exprs, list)
    }

    /// The view of `list`, a list of one of this body's instructions whose
    /// items are all present.
    pub fn items<'a>(&'a self, list: &'a [ExprId]) -> Args<'a> {
        Args::edges(&self.exprs, list)
    }

    /// Checks the arena's invariants against the roots of every
    /// instruction's expression slots.
    ///
    /// # Errors
    ///
    /// The first invariant found broken, as [`Exprs::check`] reports it.
    pub fn check(&self) -> Result<(), Inconsistent> {
        let mut roots = Vec::new();
        for instruction in &self.instructions {
            instruction.kind.for_each_root(&mut |root| roots.push(root));
        }
        self.exprs.check(roots)
    }
}

/// A directive's code body. A program read from an image (see
/// [`crate::program_from_static_image`]) holds each one still encoded until
/// it is first read; reading it then decodes it, once. Every other body is
/// built decoded. Either way it reads as the [`CodeBody`] it is.
pub struct Body {
    decoded: std::sync::OnceLock<CodeBody>,
    /// The body's image, while it may not have been decoded yet.
    encoded: Option<&'static [u8]>,
}

impl Body {
    /// A body that is already decoded.
    pub fn new(code: CodeBody) -> Self {
        Body {
            decoded: std::sync::OnceLock::from(code),
            encoded: None,
        }
    }

    /// A body to decode from `encoded` when it is first read.
    pub(crate) fn encoded(encoded: &'static [u8]) -> Self {
        Body {
            decoded: std::sync::OnceLock::new(),
            encoded: Some(encoded),
        }
    }

    /// Whether the body has been decoded.
    pub fn is_decoded(&self) -> bool {
        self.decoded.get().is_some()
    }
}

impl Default for Body {
    fn default() -> Self {
        Body::new(CodeBody::default())
    }
}

impl std::ops::Deref for Body {
    type Target = CodeBody;

    fn deref(&self) -> &CodeBody {
        self.decoded.get_or_init(|| {
            let encoded = self.encoded.expect("a body is decoded or holds its image");
            crate::image::decode_body(encoded)
                .expect("a body image the program's image check accepted decodes")
        })
    }
}

impl std::ops::DerefMut for Body {
    fn deref_mut(&mut self) -> &mut CodeBody {
        let _ = std::ops::Deref::deref(self);
        self.decoded.get_mut().expect("decoded just now")
    }
}

impl Clone for Body {
    fn clone(&self) -> Self {
        Body::new(CodeBody::clone(self))
    }
}

impl PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl Eq for Body {}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        CodeBody::fmt(self, f)
    }
}

/// Which closure action an `END` performs, as the end style and `SELECT` end
/// matching set it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum EndStyle {
    /// A `DO` with no control expression and no `LABEL`.
    Do,
    /// A `DO` with no control expression but with a `LABEL`.
    LabeledDo,
    /// Every other `DO`/`LOOP` form, labelled or not: the end style is the
    /// plain loop block for all of them, so the label makes no difference here
    /// where it does for the block form.
    Loop,
    /// A `SELECT` with no `OTHERWISE`, labelled or not. Reaching this `END` at
    /// run time is error 7.3, because every `WHEN` was false.
    Select,
    /// A `SELECT` with an `OTHERWISE` and no `LABEL`.
    Otherwise,
    /// A `SELECT` with an `OTHERWISE` and a `LABEL`.
    LabeledOtherwise,
}

/// What an `END` turned out to close, and how.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct EndTarget {
    /// The block instruction this `END` closes: a `Do`, a `Loop` or a
    /// `Select`. Never the `Otherwise`, even when `style` says the `SELECT` had
    /// one: the parser pops the `OTHERWISE` and matches the `END` against the
    /// `SELECT` behind it.
    pub block: usize,
    pub style: EndStyle,
}

/// One instruction, and the clause `TRACE` prints for it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Instruction {
    pub kind: InstructionKind,
    /// Byte range in the retained source: the clause this instruction was
    /// built from, which is what `TRACE` echoes on its `*-*` line.
    pub clause_span: Range<usize>,
}

/// One instruction form: the 35 keyword instructions, plus the four clause
/// shapes that no keyword introduces.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InstructionKind {
    // ---- the four clause shapes with no keyword ----
    /// `name = expr`, and also `name (op)= expr`, whose right-hand side is
    /// already the expanded `name op expr`, its read of `name` a node of its
    /// own before the value's.
    Assignment {
        target: ExprId,
        value: ExprId,
    },
    /// `name:`, and also `"name":`. The clause ends at the colon.
    Label {
        name: Box<[u8]>,
    },
    /// A standalone message send, `q~append(1)`, and the message-assignment
    /// forms `q[1] = 2` and `q[1] += 2`.
    Message {
        term: ExprId,
        /// The right-hand side when this is an assignment form. For the
        /// `(op)=` spelling this is already the expanded `term op expr`, over
        /// a copy of the term pushed before the value.
        value: Option<ExprId>,
    },
    /// Anything else: a clause that is an expression is a command, dispatched
    /// through the current `ADDRESS`.
    Command {
        expression: Option<ExprId>,
    },

    // ---- control flow (12) ----
    Do(Box<Loop>),
    Loop(Box<Loop>),
    If {
        condition: ExprId,
        /// Where control goes when `condition` is false: the `ELSE` when there
        /// is one, otherwise the instruction after the `THEN` branch. `None` is
        /// the end of this body.
        false_target: Option<usize>,
    },
    Then,
    Else {
        /// Where control goes when the `THEN` branch finished, which is the
        /// instruction after the `ELSE` branch. `None` is the end of this body.
        then_exit: Option<usize>,
    },
    Select {
        label: Option<SymbolId>,
        /// `SELECT CASE expr`, a different instruction class from the plain
        /// `SELECT`.
        case: Option<ExprId>,
        /// The `WHEN` instructions this `SELECT` collected, in source order.
        whens: Vec<usize>,
        otherwise: Option<usize>,
        /// The `END` that closes this `SELECT`.
        end: Option<usize>,
    },
    When {
        condition: ExprId,
        /// Where control goes when `condition` is false: the next `WHEN`, the
        /// `OTHERWISE`, or the enclosing `SELECT`'s `END`.
        false_target: Option<usize>,
        /// Where control goes when this `WHEN`'s branch finished, which is the
        /// instruction after the enclosing `SELECT`'s `END`, because one true
        /// `WHEN` ends the whole `SELECT`. `None` is the end of this body.
        exit: Option<usize>,
    },
    /// A `WHEN` inside `SELECT CASE`, a different kind from the plain `WHEN`
    /// because the clause means something else.
    WhenCase {
        /// At least one, and none may be omitted: an empty element is 35.934.
        values: Vec<ExprId>,
        false_target: Option<usize>,
        exit: Option<usize>,
    },
    Otherwise,
    Catch(Box<Catch>),
    Leave {
        name: Option<SymbolId>,
    },
    Iterate {
        name: Option<SymbolId>,
    },
    End {
        name: Option<SymbolId>,
        /// What this `END` closes, and how.
        closes: Option<EndTarget>,
    },

    // ---- data (8) ----
    Drop {
        variables: Vec<VariableRef>,
    },
    Expose {
        variables: Vec<VariableRef>,
    },
    Parse(Box<Parse>),
    Arg(Box<Parse>),
    Pull(Box<Parse>),
    Push {
        expression: Option<ExprId>,
    },
    Queue {
        expression: Option<ExprId>,
    },
    Say {
        expression: Option<ExprId>,
    },

    // ---- procedure (11) ----
    Call(Box<Call>),
    Return {
        expression: Option<ExprId>,
    },
    Procedure {
        variables: Vec<VariableRef>,
    },
    Signal(Box<Signal>),
    Exit {
        expression: Option<ExprId>,
    },
    Interpret {
        expression: ExprId,
    },
    Guard(Box<Guard>),
    Reply {
        expression: Option<ExprId>,
    },
    Forward(Box<Forward>),
    Raise(Box<Raise>),
    Use(Box<Use>),

    // ---- settings (4) ----
    Numeric {
        setting: NumericSetting,
        expression: Option<ExprId>,
    },
    Address(Box<Address>),
    Trace(Trace),
    Options {
        expression: ExprId,
    },

    // ---- and NOP ----
    Nop,
}

impl InstructionKind {
    /// The instruction keyword that introduced this node, or `None` for the
    /// four clause shapes that no keyword introduces.
    pub fn keyword(&self) -> Option<&'static str> {
        Some(match self {
            InstructionKind::Assignment { .. }
            | InstructionKind::Label { .. }
            | InstructionKind::Message { .. }
            | InstructionKind::Command { .. } => return None,
            InstructionKind::Do(_) => "DO",
            InstructionKind::Loop(_) => "LOOP",
            InstructionKind::If { .. } => "IF",
            InstructionKind::Then => "THEN",
            InstructionKind::Else { .. } => "ELSE",
            InstructionKind::Select { .. } => "SELECT",
            // Both spellings are the WHEN keyword. They are separate variants
            // because they are separate instruction classes and the clause
            // means something else in each, not because the keyword differs.
            InstructionKind::When { .. } | InstructionKind::WhenCase { .. } => "WHEN",
            InstructionKind::Otherwise => "OTHERWISE",
            InstructionKind::Catch(_) => "CATCH",
            InstructionKind::Leave { .. } => "LEAVE",
            InstructionKind::Iterate { .. } => "ITERATE",
            InstructionKind::End { .. } => "END",
            InstructionKind::Drop { .. } => "DROP",
            InstructionKind::Expose { .. } => "EXPOSE",
            InstructionKind::Parse(_) => "PARSE",
            InstructionKind::Arg(_) => "ARG",
            InstructionKind::Pull(_) => "PULL",
            InstructionKind::Push { .. } => "PUSH",
            InstructionKind::Queue { .. } => "QUEUE",
            InstructionKind::Say { .. } => "SAY",
            InstructionKind::Call(_) => "CALL",
            InstructionKind::Return { .. } => "RETURN",
            InstructionKind::Procedure { .. } => "PROCEDURE",
            InstructionKind::Signal(_) => "SIGNAL",
            InstructionKind::Exit { .. } => "EXIT",
            InstructionKind::Interpret { .. } => "INTERPRET",
            InstructionKind::Guard(_) => "GUARD",
            InstructionKind::Reply { .. } => "REPLY",
            InstructionKind::Forward(_) => "FORWARD",
            InstructionKind::Raise(_) => "RAISE",
            InstructionKind::Use(_) => "USE",
            InstructionKind::Numeric { .. } => "NUMERIC",
            InstructionKind::Address(_) => "ADDRESS",
            InstructionKind::Trace(_) => "TRACE",
            InstructionKind::Options { .. } => "OPTIONS",
            InstructionKind::Nop => "NOP",
        })
    }
}

impl InstructionKind {
    /// Calls `f` on the root of each expression slot the instruction holds,
    /// skipping the slots left empty and the omitted positions of a list.
    /// The slots come in the order the fields are declared.
    pub fn for_each_root(&self, f: &mut impl FnMut(ExprId)) {
        let opt = |slot: &Option<ExprId>, f: &mut dyn FnMut(ExprId)| {
            if let Some(expr) = *slot {
                f(expr);
            }
        };
        match self {
            InstructionKind::Assignment { target, value } => {
                f(*target);
                f(*value);
            }
            InstructionKind::Message { term, value } => {
                f(*term);
                opt(value, f);
            }
            InstructionKind::Command { expression }
            | InstructionKind::Push { expression }
            | InstructionKind::Queue { expression }
            | InstructionKind::Say { expression }
            | InstructionKind::Return { expression }
            | InstructionKind::Exit { expression }
            | InstructionKind::Reply { expression }
            | InstructionKind::Numeric { expression, .. } => opt(expression, f),
            InstructionKind::Do(header) | InstructionKind::Loop(header) => header.for_each_root(f),
            InstructionKind::If { condition, .. } | InstructionKind::When { condition, .. } => {
                f(*condition)
            }
            InstructionKind::WhenCase { values, .. } => values.iter().copied().for_each(&mut *f),
            InstructionKind::Select { case, .. } => opt(case, f),
            InstructionKind::Catch(catch) => opt(&catch.variable, f),
            InstructionKind::Label { .. }
            | InstructionKind::Then
            | InstructionKind::Else { .. }
            | InstructionKind::Otherwise
            | InstructionKind::Leave { .. }
            | InstructionKind::Iterate { .. }
            | InstructionKind::End { .. }
            | InstructionKind::Drop { .. }
            | InstructionKind::Expose { .. }
            | InstructionKind::Procedure { .. }
            | InstructionKind::Nop => {}
            InstructionKind::Parse(parse)
            | InstructionKind::Arg(parse)
            | InstructionKind::Pull(parse) => parse.for_each_root(f),
            InstructionKind::Call(call) => match &**call {
                Call::Named { args, .. } | Call::Qualified { args, .. } => {
                    args.iter().flatten().copied().for_each(&mut *f)
                }
                Call::Dynamic { target, args } => {
                    f(*target);
                    args.iter().flatten().copied().for_each(&mut *f);
                }
                Call::Trap(_) => {}
            },
            InstructionKind::Signal(signal) => match &**signal {
                Signal::Value(expr) => f(*expr),
                Signal::Label(_) | Signal::Trap(_) => {}
            },
            InstructionKind::Interpret { expression } | InstructionKind::Options { expression } => {
                f(*expression)
            }
            InstructionKind::Guard(guard) => opt(&guard.condition, f),
            InstructionKind::Forward(forward) => {
                let Forward {
                    to,
                    message,
                    class,
                    arguments,
                    array,
                    continue_: _,
                } = &**forward;
                for slot in [to, message, class, arguments] {
                    opt(slot, f);
                }
                array.iter().flatten().flatten().copied().for_each(&mut *f);
            }
            InstructionKind::Raise(raise) => {
                let Raise {
                    condition: _,
                    propagate: _,
                    rc,
                    description,
                    additional,
                    array,
                    result,
                } = &**raise;
                for slot in [rc, description, additional] {
                    opt(slot, f);
                }
                array.iter().flatten().flatten().copied().for_each(&mut *f);
                if let Some(result) = result {
                    opt(&result.value, f);
                }
            }
            InstructionKind::Use(using) => match &**using {
                Use::Arg { targets, .. } => {
                    for target in targets.iter().flatten() {
                        f(target.target);
                        opt(&target.default, f);
                    }
                }
                Use::Local { .. } => {}
            },
            InstructionKind::Address(address) => {
                opt(&address.dynamic, f);
                opt(&address.command, f);
                if let Some(io) = &address.io {
                    for redirection in [&io.input, &io.output, &io.error] {
                        match redirection {
                            Redirection::Stream(expr) | Redirection::Using(expr) => f(*expr),
                            Redirection::Default | Redirection::Normal | Redirection::Stem(_) => {}
                        }
                    }
                }
            }
            InstructionKind::Trace(Trace::Value(expr)) => f(*expr),
            InstructionKind::Trace(Trace::Default | Trace::Setting(_) | Trace::Skip(_)) => {}
        }
    }
}

/// A condition handler at the end of a DO, LOOP or SELECT block.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Catch {
    /// Where to assign the condition object on entry, if specified.
    pub variable: Option<ExprId>,
    /// The listed conditions, in the order they were written.
    pub conditions: Vec<CatchCondition>,
    /// The opening instruction. Set when the parser assembles the block.
    pub block: Option<usize>,
}

/// A CATCH match, optionally restricted to an error or return code.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CatchCondition {
    /// An uppercase condition name, with the name after USER included.
    pub name: Box<[u8]>,
    pub code: Option<CatchCode>,
}

/// A CATCH code is a constant or a simple name read when matching.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CatchCode {
    Literal(Box<[u8]>),
    Variable(SymbolId),
}

/// One name in a `DROP`, `EXPOSE`, `PROCEDURE EXPOSE` or `USE LOCAL` list.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum VariableRef {
    /// A name written out: a simple variable, a stem, or a compound. Which
    /// of the three follows from the spelling, as it does for an expression leaf.
    Direct(SymbolId),
    /// `(name)`, where the *value* of `name` names the variable to act on.
    Indirect(SymbolId),
}

/// A `DO` or `LOOP` header.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Loop {
    /// `DO LABEL name`. For a controlled or `OVER` loop with no `LABEL`, the
    /// control variable's name becomes the label, .
    pub label: Option<SymbolId>,
    /// `DO COUNTER name`.
    pub counter: Option<SymbolId>,
    pub kind: LoopKind,
    /// A trailing `WHILE` or `UNTIL`. Never both: the loop conditional requires
    /// the end of the clause after the one it parsed.
    pub conditional: Option<LoopConditional>,
    /// The `END` that closes this block.
    pub end: Option<usize>,
}

impl Loop {
    /// [`InstructionKind::for_each_root`] for a `DO` or `LOOP` header.
    fn for_each_root(&self, f: &mut impl FnMut(ExprId)) {
        match &self.kind {
            LoopKind::Simple | LoopKind::Forever => {}
            LoopKind::Count(count) => count.iter().copied().for_each(&mut *f),
            LoopKind::Controlled(controlled) => {
                f(controlled.initial);
                [controlled.to, controlled.by, controlled.for_count]
                    .into_iter()
                    .flatten()
                    .for_each(&mut *f);
            }
            LoopKind::Over {
                target, for_count, ..
            }
            | LoopKind::With {
                target, for_count, ..
            } => {
                f(*target);
                for_count.iter().copied().for_each(&mut *f);
            }
        }
        if let Some(conditional) = &self.conditional {
            f(conditional.condition);
        }
    }
}

/// Which loop a `DO` or `LOOP` header is.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LoopKind {
    /// `DO` alone: a block, not a loop. `LOOP` alone is `Forever` instead.
    Simple,
    /// `DO FOREVER`, and `LOOP` with no control at all.
    Forever,
    /// `DO expr`, repeated that many times.
    Count(Option<ExprId>),
    /// `DO i = 1 TO 9 BY 2 FOR 3`.
    Controlled(Box<Controlled>),
    /// `DO name OVER expr`.
    Over {
        control: SymbolId,
        target: ExprId,
        for_count: Option<ExprId>,
    },
    /// `DO WITH INDEX i ITEM v OVER expr`. At least one of the two variables
    /// is present.
    With {
        index: Option<SymbolId>,
        item: Option<SymbolId>,
        target: ExprId,
        for_count: Option<ExprId>,
    },
}

/// The control expressions of `DO i = initial TO t BY b FOR f`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Controlled {
    pub control: SymbolId,
    pub initial: ExprId,
    pub to: Option<ExprId>,
    pub by: Option<ExprId>,
    pub for_count: Option<ExprId>,
    /// The order the three keyword expressions were written in, which is the
    /// order they are evaluated in (`control.expressions[keyslot++]`).
    /// Evaluation order is observable, because an expression can have side
    /// effects, so it is recorded rather than fixed.
    pub order: Vec<ControlExpr>,
}

/// One entry of a controlled loop's evaluation order.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ControlExpr {
    To,
    By,
    For,
}

/// A `WHILE` or `UNTIL` on a loop.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LoopConditional {
    /// True for `UNTIL`, which is tested after the body rather than before.
    pub until: bool,
    pub condition: ExprId,
}

/// A `PARSE`, `ARG` or `PULL` instruction.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Parse {
    pub source: ParseSource,
    /// `PARSE UPPER`, and implied by the `ARG` and `PULL` spellings.
    pub upper: bool,
    pub lower: bool,
    pub caseless: bool,
    /// The templates. `None` is the comma fence between one template and the
    /// next, which is pushed as a null entry.
    pub template: Vec<Option<ParseTrigger>>,
}

impl Parse {
    /// [`InstructionKind::for_each_root`] for a `PARSE`, `ARG` or `PULL`.
    fn for_each_root(&self, f: &mut impl FnMut(ExprId)) {
        if let ParseSource::Value(Some(value)) = self.source {
            f(value);
        }
        for trigger in self.template.iter().flatten() {
            trigger.value.iter().copied().for_each(&mut *f);
            trigger.targets.iter().flatten().copied().for_each(&mut *f);
        }
    }
}

/// Where a `PARSE` gets its string.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ParseSource {
    Arg,
    LineIn,
    Pull,
    Source,
    Version,
    Var(SymbolId),
    /// `PARSE VALUE expr WITH`. The expression is optional and defaults to
    /// the null string.
    Value(Option<ExprId>),
}

/// One template trigger and the variables it assigns.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParseTrigger {
    pub kind: TriggerKind,
    /// The pattern or column: a literal, a numeric symbol, or a
    /// parenthesised expression. Absent for `TriggerKind::End`.
    pub value: Option<ExprId>,
    /// The targets assigned when this trigger fires. `None` is a `.`
    /// placeholder, which consumes a field and assigns nothing.
    pub targets: Vec<Option<ExprId>>,
}

/// What a `PARSE` template trigger matches on.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TriggerKind {
    /// The implicit trigger that assigns whatever is left.
    End,
    /// `+n`, relative forward.
    Plus,
    /// `-n`, relative backward.
    Minus,
    /// `=n` and a bare numeric symbol, both absolute.
    Absolute,
    /// `<n`.
    MinusLength,
    /// `>n`.
    PlusLength,
    /// A literal or `(expr)` pattern.
    String,
    /// The same, under `PARSE CASELESS`.
    Mixed,
}

/// A `CALL` instruction, in all four of its forms.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Call {
    /// `CALL name arg, arg`. `literal` is true for `CALL "name"`, which
    /// bypasses the internal label search.
    Named {
        name: Box<[u8]>,
        literal: bool,
        args: Vec<Option<ExprId>>,
    },
    /// `CALL (expr) arg`, whose target is only known at run time.
    Dynamic {
        target: ExprId,
        args: Vec<Option<ExprId>>,
    },
    /// `CALL ns:name arg`, restricted to public routines of that namespace.
    Qualified {
        namespace: SymbolId,
        name: SymbolId,
        args: Vec<Option<ExprId>>,
    },
    /// `CALL ON cond NAME label` and `CALL OFF cond`.
    Trap(ConditionTrap),
}

/// A `SIGNAL` instruction, in all three of its forms.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Signal {
    /// `SIGNAL label` and `SIGNAL "label"`.
    Label(Box<[u8]>),
    /// `SIGNAL VALUE expr`, and the implicit form where the target is not a
    /// symbol or a literal.
    Value(ExprId),
    /// `SIGNAL ON cond NAME label` and `SIGNAL OFF cond`.
    Trap(ConditionTrap),
}

/// The shared shape of `CALL ON`/`OFF` and `SIGNAL ON`/`OFF`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ConditionTrap {
    pub on: bool,
    /// The condition name, `USER name` spelled out for a user condition.
    pub condition: Box<[u8]>,
    /// The label to trap to. `None` for the `OFF` form, which is how the two
    /// forms are told apart.
    pub label: Option<Box<[u8]>>,
}

/// A `GUARD ON`/`OFF` instruction.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Guard {
    pub on: bool,
    /// `GUARD ON WHEN expr`.
    pub condition: Option<ExprId>,
    /// The exposed variables `condition` names, each once, upcased: what a
    /// wait watches.
    pub watched: Vec<Box<[u8]>>,
}

/// A `FORWARD` instruction's options, all of them optional.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Forward {
    pub to: Option<ExprId>,
    pub message: Option<ExprId>,
    pub class: Option<ExprId>,
    /// `FORWARD ARGUMENTS expr`, mutually exclusive with `array`.
    pub arguments: Option<ExprId>,
    /// `FORWARD ARRAY (a, b)`.
    pub array: Option<Vec<Option<ExprId>>>,
    pub continue_: bool,
}

/// A `RAISE` instruction.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Raise {
    /// The condition name, with `USER ` prefixed for a user condition.
    pub condition: Box<[u8]>,
    pub propagate: bool,
    /// The argument `ERROR`, `FAILURE` and `SYNTAX` take.
    pub rc: Option<ExprId>,
    pub description: Option<ExprId>,
    /// `ADDITIONAL expr`, mutually exclusive with `array`.
    pub additional: Option<ExprId>,
    pub array: Option<Vec<Option<ExprId>>>,
    /// `RETURN expr` or `EXIT expr`, whose value is optional either way.
    pub result: Option<RaiseResult>,
}

/// `RAISE ... RETURN expr` versus `RAISE ... EXIT expr`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RaiseResult {
    pub exit: bool,
    pub value: Option<ExprId>,
}

/// A `USE ARG`, `USE STRICT ARG` or `USE LOCAL` instruction.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Use {
    Arg {
        strict: bool,
        /// True when the list ended with `...`, which stops argument-count
        /// checking at that point.
        allow_optionals: bool,
        /// An omitted position, written as a bare comma, is `None`.
        targets: Vec<Option<UseTarget>>,
    },
    Local {
        variables: Vec<VariableRef>,
    },
}

/// One target of a `USE ARG` list.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UseTarget {
    /// A variable or a message term.
    pub target: ExprId,
    /// `USE ARG a = 1`, a constant expression. Never present with `alias`.
    pub default: Option<ExprId>,
    /// `USE ARG >a`, which aliases the caller's variable rather than copying.
    /// `<` is the same thing.
    pub alias: bool,
}

/// An `ADDRESS` instruction.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Address {
    /// `ADDRESS env`, the constant target. A symbol contributes its upcased
    /// spelling and a literal its bytes, which is what the token value yields
    /// for each.
    pub environment: Option<Box<[u8]>>,
    /// `ADDRESS VALUE expr`, and the implicit form where the target is
    /// neither a symbol nor a literal.
    pub dynamic: Option<ExprId>,
    /// `ADDRESS env command`.
    pub command: Option<ExprId>,
    /// The `WITH` redirections. Absent means the plain `ADDRESS` form rather
    /// than the `WITH` form.
    pub io: Option<Box<AddressIo>>,
}

/// `ADDRESS ... WITH` input and output redirection.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct AddressIo {
    pub input: Redirection,
    pub output: Redirection,
    pub error: Redirection,
    pub output_option: OutputOption,
    pub error_option: OutputOption,
}

/// Where one of the three command streams goes.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub enum Redirection {
    /// Not mentioned at all.
    #[default]
    Default,
    /// `NORMAL`, which resets this stream to the default.
    Normal,
    /// `STEM name.`.
    Stem(SymbolId),
    /// `STREAM expr`, a constant expression naming a file.
    Stream(ExprId),
    /// `USING expr`, an object decided at run time.
    Using(ExprId),
}

/// `APPEND` or `REPLACE` on an output redirection.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum OutputOption {
    #[default]
    Default,
    Replace,
    Append,
}

/// Which setting a `NUMERIC` instruction changes.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum NumericSetting {
    Digits,
    Fuzz,
    /// `NUMERIC FORM` alone, which resets to the package default.
    FormDefault,
    FormScientific,
    FormEngineering,
    /// `NUMERIC FORM VALUE expr`, and the implicit form where what follows
    /// `FORM` is not a symbol.
    FormValue,
}

/// A `TRACE` instruction, in its four forms.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Trace {
    /// `TRACE` alone: the default setting.
    Default,
    /// A validated option string, kept as written. Only its leading `?`s and
    /// first other character mean anything, and the rest is retained because the
    /// setting is echoed back by `TRACE()`.
    Setting(Box<[u8]>),
    /// A whole number, which skips that many debug pauses. Negative for the
    /// `TRACE -n` spelling.
    Skip(i64),
    /// `TRACE VALUE expr`, and the implicit form where what follows is not a
    /// symbol, a literal, or a signed number.
    Value(ExprId),
}

/// One `::` directive, and the clause it was built from.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Directive {
    pub kind: DirectiveKind,
    /// Byte range in the retained source: the directive clause.
    pub clause_span: Range<usize>,
}

/// One directive form, one variant per row of the directive table.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DirectiveKind {
    Annotate(Box<Annotate>),
    Attribute(Box<AttributeDirective>),
    Class(Box<ClassDirective>),
    Constant(Box<ConstantDirective>),
    Method(Box<MethodDirective>),
    /// `::OPTIONS`, whose options may repeat and whose order is observable, so
    /// this is a list and not a struct of fields. Measured with
    /// `build/bin/rexx`: `::options digits 12` then `::options digits 5` makes
    /// `digits()` report 5, and the two directives swapped make it report 12,
    /// so a later option overrides an earlier one.
    Options(Vec<PackageOption>),
    Requires(Box<Requires>),
    Resource(Box<Resource>),
    Routine(Box<RoutineDirective>),
}

impl DirectiveKind {
    /// The directive keyword that introduced this node.
    pub fn keyword(&self) -> &'static str {
        match self {
            DirectiveKind::Annotate(_) => "ANNOTATE",
            DirectiveKind::Attribute(_) => "ATTRIBUTE",
            DirectiveKind::Class(_) => "CLASS",
            DirectiveKind::Constant(_) => "CONSTANT",
            DirectiveKind::Method(_) => "METHOD",
            DirectiveKind::Options(_) => "OPTIONS",
            DirectiveKind::Requires(_) => "REQUIRES",
            DirectiveKind::Resource(_) => "RESOURCE",
            DirectiveKind::Routine(_) => "ROUTINE",
        }
    }
}

/// A method's or routine's access scope.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Access {
    #[default]
    Default,
    Private,
    Public,
    Package,
}

/// Whether a method runs with the object's method protection.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Protection {
    #[default]
    Default,
    Protected,
    Unprotected,
}

/// Whether a method takes the object's guard lock.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum GuardOption {
    #[default]
    Default,
    Guarded,
    Unguarded,
}

/// A decoded `EXTERNAL` specification.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExternalSpec {
    /// True for the `REGISTERED` spelling, which only `::ROUTINE` accepts and
    /// which resolves an old-style external function instead of a library
    /// entry point.
    pub registered: bool,
    /// The library name, as written after the first word.
    pub library: Box<[u8]>,
    /// The entry point, when the specification named a third word.
    pub entry: Option<Box<[u8]>>,
}

/// A `::CLASS` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ClassDirective {
    /// The name as written. The class is also exported under the upcased
    /// spelling, which is what a duplicate is detected on, but that is the
    /// accumulator's business and not this node's.
    pub name: Box<[u8]>,
    /// `PUBLIC` or `PRIVATE`. `Package` never appears: measured,
    /// `::CLASS c PACKAGE` is 25.901.
    pub access: Access,
    pub abstract_: bool,
    /// `SUBCLASS c` and `MIXINCLASS c`, which fill the same slot (setting the
    /// mixin class sets the subclass too), which is why a directive carrying
    /// both is 25.901 whichever order they come in.
    pub subclass: Option<ClassRef>,
    /// True when the subclass came from `MIXINCLASS`.
    pub mixin: bool,
    pub metaclass: Option<ClassRef>,
    /// `INHERIT a b c`, which consumes every remaining token of the clause.
    pub inherit: Vec<ClassRef>,
}

/// A class reference on a `::CLASS` directive: the argument of `SUBCLASS`,
/// `MIXINCLASS`, `METACLASS`, or one entry of `INHERIT`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ClassRef {
    /// `ns:name`, restricted to the symbol spelling. A literal cannot carry a
    /// namespace.
    pub namespace: Option<SymbolId>,
    /// Upcased for both spellings. A symbol arrives upcased from the scanner and
    /// a literal is upcased here, because the upcased token value is what the
    /// literal form uses.
    pub name: Box<[u8]>,
}

/// A `::METHOD` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MethodDirective {
    /// The name as written, which is the method object's own name. The lookup
    /// name is its upcased spelling, and the two differ observably for
    /// `::METHOD "abc"`.
    pub name: Box<[u8]>,
    /// `CLASS`, making this a class method rather than an instance method.
    pub class_method: bool,
    /// `ATTRIBUTE`, which generates a getter and a setter pair.
    pub attribute: bool,
    pub abstract_: bool,
    pub access: Access,
    pub protection: Protection,
    pub guard: GuardOption,
    pub external: Option<ExternalSpec>,
    /// `DELEGATE property`, forwarding every message to that property's value.
    /// A symbol only: measured, `::METHOD m DELEGATE "p"` is 20.926.
    pub delegate: Option<SymbolId>,
    /// This method's code body: the clauses after this directive, assembled.
    pub body: Option<Body>,
}

/// A `::ATTRIBUTE` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AttributeDirective {
    /// The name as written, which also names the instance variable the
    /// generated methods read and write.
    pub name: Box<[u8]>,
    pub style: AttributeStyle,
    pub class_method: bool,
    pub abstract_: bool,
    pub access: Access,
    pub protection: Protection,
    pub guard: GuardOption,
    pub external: Option<ExternalSpec>,
    pub delegate: Option<SymbolId>,
    /// This attribute method's code body: the clauses after this directive,
    /// assembled.
    pub body: Option<Body>,
}

/// Which of the attribute method pair a `::ATTRIBUTE` directive defines.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum AttributeStyle {
    /// Neither `GET` nor `SET`, so both methods are generated and no body may
    /// follow.
    #[default]
    Both,
    Get,
    Set,
}

/// A `::CONSTANT` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ConstantDirective {
    pub name: Box<[u8]>,
    pub value: ConstantValue,
}

/// What a `::CONSTANT` directive's value is.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ConstantValue {
    /// No value at all, whose value is the constant's own name AS WRITTEN and
    /// not upcased.
    Name,
    /// A literal, a symbol, or a signed constant symbol, taken as text. The
    /// signed form is concatenated so that `::CONSTANT c - 5` yields `-5` with
    /// the blank dropped.
    Text(Box<[u8]>),
    /// `(expr)`, which is evaluated when the package is installed rather than
    /// now. The expression is outside any code body, so it has an arena of
    /// its own, whose one root is `root`.
    Expression { exprs: Exprs, root: ExprId },
}

impl ConstantValue {
    /// The view of the `(expr)` value, if the value is one.
    pub fn expression(&self) -> Option<ExprRef<'_>> {
        match self {
            ConstantValue::Expression { exprs, root } => Some(exprs.get(*root)),
            ConstantValue::Name | ConstantValue::Text(_) => None,
        }
    }

    /// Checks the arena of an `(expr)` value's invariants; `Ok` for the
    /// other forms, which hold none.
    ///
    /// # Errors
    ///
    /// The first invariant found broken, as [`Exprs::check`] reports it.
    pub fn check(&self) -> Result<(), Inconsistent> {
        match self {
            ConstantValue::Expression { exprs, root } => exprs.check([*root]),
            ConstantValue::Name | ConstantValue::Text(_) => Ok(()),
        }
    }
}

/// A `::ANNOTATE` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Annotate {
    pub target: AnnotationTarget,
    /// The `symbol value` pairs, in order. An `::ANNOTATE` may carry none.
    pub annotations: Vec<Annotation>,
}

/// What a `::ANNOTATE` directive annotates.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AnnotationTarget {
    Package,
    Class(Box<[u8]>),
    Routine(Box<[u8]>),
    Method(Box<[u8]>),
    /// The getter name. Whichever of the getter/setter pair exists is
    /// annotated, so the setter name is derived rather than stored.
    Attribute(Box<[u8]>),
    Constant(Box<[u8]>),
}

/// One `name value` pair of a `::ANNOTATE` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Annotation {
    /// The name, which must be a symbol: measured, `::ANNOTATE PACKAGE "a" 1`
    /// is 20.919.
    pub name: SymbolId,
    /// The value as text, with the same three forms a `::CONSTANT` value has
    /// minus the parenthesised one, which is not accepted here.
    pub value: Box<[u8]>,
}

/// One option of a `::OPTIONS` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PackageOption {
    /// `DIGITS n`, always at least 1.
    Digits(usize),
    Fuzz(usize),
    Form(OptionsForm),
    /// A validated `TRACE` option string, kept as written for the same reason
    /// `Trace::Setting` keeps one.
    Trace(Box<[u8]>),
    /// One of the seven conditions that can be raised as a SYNTAX error
    /// instead of as a condition. `syntax` is true for the `SYNTAX` spelling
    /// and false for `CONDITION`.
    Condition {
        which: ConditionOption,
        syntax: bool,
    },
    /// `PROLOG` and `NOPROLOG`, true for the first.
    Prolog(bool),
    /// `NUMERIC INHERIT` and `NUMERIC NOINHERIT`, true for the first.
    NumericInherit(bool),
}

/// `::OPTIONS FORM`'s two settings.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum OptionsForm {
    Scientific,
    Engineering,
}

/// Which condition a `::OPTIONS` condition option selects.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ConditionOption {
    All,
    Error,
    Failure,
    LostDigits,
    NoString,
    NotReady,
    NoValue,
}

/// A `::REQUIRES` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Requires {
    /// The name as written, so `::REQUIRES "Mixed.cls"` keeps its case while
    /// the symbol spelling arrives upcased.
    pub name: Box<[u8]>,
    /// `LIBRARY`, which makes this a native library rather than a package
    /// file. Never true together with a namespace: measured,
    /// `::REQUIRES x LIBRARY NAMESPACE ns` is 25.904 whichever order the two
    /// come in.
    pub library: bool,
    /// `NAMESPACE ns`. A symbol only, so always upcased, and never `REXX`:
    /// measured, `::REQUIRES "x" NAMESPACE REXX` is 99.944.
    pub namespace: Option<SymbolId>,
}

/// A `::RESOURCE` directive and its verbatim body.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Resource {
    /// The name as written. The package's resource table is keyed by the
    /// upcased spelling.
    pub name: Box<[u8]>,
    /// The line that ends the body, `::END` unless the directive named
    /// another. Compared against the source verbatim, so a lower-case `::end`
    /// does NOT end a body that expects `::END`.
    pub end_marker: Box<[u8]>,
    /// Byte range of each body line in the retained source, line terminators
    /// and the marker line excluded.
    pub lines: Vec<Range<usize>>,
}

/// A `::ROUTINE` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RoutineDirective {
    /// The name as written and NOT upcased, because a quoted routine name is
    /// looked up case-sensitively.
    pub name: Box<[u8]>,
    /// `PUBLIC` or `PRIVATE`. `Package` never appears: measured,
    /// `::ROUTINE r PACKAGE` is 25.903.
    pub access: Access,
    pub external: Option<ExternalSpec>,
    /// This routine's code body: the clauses after this directive, assembled.
    /// `Some` for every routine that is not external.
    pub body: Option<Body>,
}

#[cfg(test)]
mod tests;
