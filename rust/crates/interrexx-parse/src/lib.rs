//! The Rexx parser: source retention, scanning, and the grammar.
//!
//! The parsing functions run on the caller's thread. When nesting is unknown,
//! use [`with_parser_stack`] or give the parsing thread [`PARSER_STACK_BYTES`]
//! of stack. On a smaller stack, native stack exhaustion can abort the process
//! before parsing reaches [`MAX_EXPR_DEPTH`].

// A layer's entry
// point can have no non-test caller. `cargo clippy
// --all-targets` compiles the library once with `cfg(test)` off, and there each
// such item was dead.

mod ast;
mod block;
mod clause;
mod convert;
mod directive;
mod error;
mod expr;
mod exprs;
mod image;
mod instruction;
mod scanner;
mod selector;
mod source;
mod token;

pub use ast::{
    Access, Address, AddressIo, Annotate, Annotation, AnnotationTarget, AttributeDirective,
    AttributeStyle, Body, Call, Catch, CatchCode, CatchCondition, ClassDirective, ClassRef,
    CodeBody, ConditionOption, ConditionTrap, ConstantDirective, ConstantValue, ControlExpr,
    Controlled, Directive, DirectiveKind, EndStyle, EndTarget, ExternalSpec, Forward, Guard,
    GuardOption, Instruction, InstructionKind, Loop, LoopConditional, LoopKind, MethodDirective,
    NumericSetting, OptionsForm, OutputOption, PackageOption, Parse, ParseSource, ParseTrigger,
    PrefixOp, Protection, Raise, RaiseResult, Redirection, Requires, Resource, RoutineDirective,
    Signal, Tail, Trace, TriggerKind, Use, UseTarget, VariableRef, compound_parts,
};
pub use convert::syntax_code;
/// The parser's own nesting limit, exported because a caller has to be able
/// to reason about it: it decides which inputs come back as `11.1` rather than
/// as an AST, and it is the number a test or an embedder checks its own depths
/// against rather than hardcoding 50,000 in two places.
pub use expr::MAX_EXPR_DEPTH;
pub use exprs::{
    Args, Bounds, Branches, BytesRange, CallTarget, EdgeRange, ExprId, ExprRef, ExprView, Exprs,
    Inconsistent, Node, NodeKind, Target,
};
pub use image::{program_from_image, program_from_static_image, program_image};
pub use scanner::{ResourceBody, Scanned, scan};
pub use selector::Selector;
pub use source::{ProgramSource, SourceKind};
pub use token::{
    KeywordSet, Keywords, Operator, ParseError, SymbolClass, SymbolId, SymbolTable, Tag, Token,
    TokenKind,
};

use interrexx_inventory::errors::E99_914;

use crate::block::translate_block;
use crate::clause::{ClauseCursor, split_clauses};
use crate::directive::{Declared, parse_directive};
use crate::scanner::scan_to_error;
use crate::selector::SelectorTable;
use crate::token::ParseCtx;

/// Stack size used to test parsing at [`MAX_EXPR_DEPTH`].
pub const PARSER_STACK_BYTES: usize = 512 * 1024 * 1024;

/// Runs `work` on a fresh thread with [`PARSER_STACK_BYTES`] of stack and
/// waits for it to finish. The closure can borrow its inputs from the caller.
///
/// # Errors
///
/// Returns an I/O error if the thread cannot be spawned. A parse error
/// returned by the closure is part of the closure's result.
///
/// # Panics
///
/// Propagates a panic from the worker to the caller.
///
/// # Examples
///
/// ```
/// let line = b"say 'hello'".to_vec();
/// let program = interrexx_parse::with_parser_stack(|| {
///     interrexx_parse::parse_lines(&[&line])
/// })??;
/// assert_eq!(program.main.instructions.len(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn with_parser_stack<T: Send>(work: impl FnOnce() -> T + Send) -> std::io::Result<T> {
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("rexx-parse".into())
            .stack_size(PARSER_STACK_BYTES)
            .spawn_scoped(scope, work)?;
        match worker.join() {
            Ok(result) => Ok(result),
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}

/// A whole program: everything `translate` produces from one source buffer.
#[derive(Debug)]
pub struct Program {
    pub source: ProgramSource,
    /// The main code body. A directive's own body is a `CodeBody` too, held
    /// inside `directives` rather than here.
    pub main: CodeBody,
    /// The `::` directives, in source order, each carrying its own assembled
    /// body.
    pub directives: Vec<Directive>,
    /// Retained because a `SymbolId` is meaningless without it: the executor
    /// resolves names back to text to report them.
    pub symbols: SymbolTable,
}

impl Program {
    /// Checks the arena of every body and every `::CONSTANT` expression
    /// against the roots its instructions name and against the source text
    /// and symbol table its nodes index.
    ///
    /// # Errors
    ///
    /// The first invariant found broken, as [`Exprs::check`] reports it.
    pub fn check(&self) -> Result<(), Inconsistent> {
        let bounds = Bounds {
            text: self.source.parts().0.len(),
            symbols: self.symbols.len(),
        };
        self.main.check(bounds)?;
        for directive in &self.directives {
            let body = match &directive.kind {
                DirectiveKind::Method(method) => method.body.as_ref(),
                DirectiveKind::Attribute(attribute) => attribute.body.as_ref(),
                DirectiveKind::Routine(routine) => routine.body.as_ref(),
                DirectiveKind::Constant(constant) => {
                    constant.value.check(bounds)?;
                    None
                }
                DirectiveKind::Annotate(_)
                | DirectiveKind::Class(_)
                | DirectiveKind::Options(_)
                | DirectiveKind::Requires(_)
                | DirectiveKind::Resource(_) => None,
            };
            if let Some(body) = body {
                body.check(bounds)?;
            }
        }
        Ok(())
    }
}

/// What `INTERPRET` produces: one code body, parsed at *run time* rather than
/// at build time, from the string an `INTERPRET` instruction is about to run.
#[derive(Debug)]
pub struct Fragment {
    pub source: ProgramSource,
    pub body: CodeBody,
    pub symbols: SymbolTable,
}

/// A source that did not parse, with the error reported against it.
#[derive(Debug)]
pub struct Rejected {
    pub error: ParseError,
    pub source: ProgramSource,
    /// The directives whose `EXTERNAL` the parse passed the resolution point
    /// for before it failed, in that order. A failure to resolve one of them is
    /// reported in place of `error`.
    pub binds: Vec<Directive>,
}

/// Parses a whole program from `text`.
pub fn parse_program(text: Vec<u8>) -> Result<Program, ParseError> {
    program_from(ProgramSource::new(text, SourceKind::Program)).map_err(|rejected| rejected.error)
}

/// Parses a program `text` already known to translate, such as the
/// interpreter's own library, without checking its directives against the
/// ones declared before them.
pub fn parse_known_program(text: Vec<u8>) -> Result<Program, ParseError> {
    let source = ProgramSource::new(text, SourceKind::Program);
    match parse(&source, false, None) {
        Ok(parsed) => Ok(Program {
            source,
            main: parsed.main,
            directives: parsed.directives,
            symbols: parsed.symbols,
        }),
        Err((error, _)) => Err(error),
    }
}

/// Parses a program whose physical lines are given one per element.
pub fn parse_lines(lines: &[&[u8]]) -> Result<Program, ParseError> {
    program_from(ProgramSource::from_lines(lines)).map_err(|rejected| rejected.error)
}

/// Parses the string an `INTERPRET` instruction is about to run.
pub fn parse_interpret(text: Vec<u8>) -> Result<Fragment, ParseError> {
    fragment_from(ProgramSource::new(text, SourceKind::Interpret))
        .map_err(|rejected| rejected.error)
}

/// Parses `source` as a whole program, answering it back with the error when
/// it does not parse.
pub fn program_from(source: ProgramSource) -> Result<Program, Rejected> {
    program_from_tracking(source, None)
}

/// The minimum language level of a compiled program using 6.05 or 6.06 features.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum RequiredLanguageLevel {
    #[default]
    V6_05 = 60500,
    V6_06 = 60600,
}

/// Parses a whole program and records which language level its translation requires.
pub fn program_from_with_language_level(
    source: ProgramSource,
) -> Result<(Program, RequiredLanguageLevel), Rejected> {
    let required_level = token::RequiredLanguage::default();
    let program = program_from_tracking(source, Some(&required_level))?;
    Ok((program, required_level.level.get()))
}

fn program_from_tracking(
    source: ProgramSource,
    required_level: Option<&token::RequiredLanguage>,
) -> Result<Program, Rejected> {
    match parse(&source, true, required_level) {
        Ok(parsed) => Ok(Program {
            source,
            main: parsed.main,
            directives: parsed.directives,
            symbols: parsed.symbols,
        }),
        Err((error, binds)) => Err(Rejected {
            error,
            source,
            binds,
        }),
    }
}

/// Parses `source` as an `INTERPRET` fragment, answering it back with the
/// error when it does not parse.
pub fn fragment_from(source: ProgramSource) -> Result<Fragment, Rejected> {
    // `INTERPRET` text carries no directive, so it declares nothing to check.
    let parsed = match parse(&source, false, None) {
        Ok(parsed) => parsed,
        Err((error, binds)) => {
            return Err(Rejected {
                error,
                source,
                binds,
            });
        }
    };
    debug_assert!(
        parsed.directives.is_empty(),
        "INTERPRET text cannot carry a directive: `parse` raises 99.914 first"
    );
    // Measured: a label inside `INTERPRET` text is 47.1, already
    // enforced where `parse_instruction` builds a `Label` node, so this map is
    // always empty rather than load-bearing.
    debug_assert!(
        parsed.main.labels.is_empty(),
        "INTERPRET text cannot carry a label: `parse_instruction` raises 47.1 first"
    );
    Ok(Fragment {
        source,
        body: parsed.main,
        symbols: parsed.symbols,
    })
}

/// What one parse produces, before it is split into `Program` or `Fragment`.
struct Parsed {
    /// The main code body. A directive's body is inside the directive.
    main: CodeBody,
    directives: Vec<Directive>,
    symbols: SymbolTable,
}

/// The composition shared by both entry points. A failure carries
/// [`Rejected::binds`].
fn parse(
    source: &ProgramSource,
    checked: bool,
    required_level: Option<&token::RequiredLanguage>,
) -> Result<Parsed, (ParseError, Vec<Directive>)> {
    let (scanned, scan_error) = scan_to_error(source);
    // Declared ahead of `ctx`, which borrows it, and dropped with the parse.
    // A `Selector` in the arena holds the name's bytes itself, so what the
    // pool's own entry buys is finding a spelling again while the parse is
    // still reading names -- which is exactly as long as it lives.
    let selectors = std::cell::RefCell::new(SelectorTable::new());
    let exprs = std::cell::RefCell::new(Exprs::default());
    let ctx = ParseCtx {
        source,
        tokens: &scanned.tokens,
        symbols: &scanned.symbols,
        selectors: &selectors,
        exprs: &exprs,
        keywords: scanned.keywords,
        resources: &scanned.resources,
        required_level,
    };

    let clauses = split_clauses(ctx.tokens).map_err(|error| (error, Vec::new()))?;
    let mut cursor = ClauseCursor::new(clauses);
    let mut directives = Vec::new();
    let mut binds = Vec::new();
    let parsed = translate(&ctx, &mut cursor, checked, &mut directives, &mut binds);
    let bound = |directives: &[Directive]| -> Vec<Directive> {
        binds
            .iter()
            .map(|&index| directives[index].clone())
            .collect()
    };
    if let Some(error) = scan_error
        && (parsed.is_ok() || cursor.read_past_end())
    {
        return Err((error, bound(&directives)));
    }
    match parsed {
        Ok(main) => Ok(Parsed {
            main,
            directives,
            symbols: scanned.symbols,
        }),
        Err(error) => Err((cursor.echo(error), bound(&directives))),
    }
}

/// The grammar over the clauses `cursor` holds: the main body, then the
/// directives into `directives`, with the index of each one whose `EXTERNAL` is
/// resolved while translating in `binds`.
fn translate<'a>(
    ctx: &'a ParseCtx<'a>,
    cursor: &mut ClauseCursor,
    checked: bool,
    directives: &mut Vec<Directive>,
    binds: &mut Vec<usize>,
) -> Result<CodeBody, ParseError> {
    let main = translate_block(ctx, cursor)?;

    // `translate` raises 99.914 exactly here, once, before the next directive is
    // ever called: `INTERPRET` text may not carry a directive at all, so this is
    // not a per-directive check. Measured via a `signal on syntax` trap around
    // `interpret "::routine r"`: `condition('o')~code` is `99.914`.
    if let Some(clause) = cursor.peek()
        && ctx.source.kind() == SourceKind::Interpret
    {
        return Err(ParseError::raise(E99_914, clause.span.start, []));
    }
    if cursor.peek().is_none() {
        return Ok(main);
    }

    let mut declared = if checked {
        Declared::default()
    } else {
        Declared::unchecked()
    };
    while cursor.peek().is_some() {
        let mut directive = parse_directive(ctx, cursor, &mut declared)?;
        if let Some(required_level) = ctx.required_level {
            // A directive's constant expression has no body-local labels.
            required_level.finish_body(false);
            if let DirectiveKind::Options(options) = &directive.kind
                && options
                    .iter()
                    .any(|option| matches!(option, PackageOption::NumericInherit(_)))
            {
                required_level.level.set(RequiredLanguageLevel::V6_06);
            }
        }
        if declared.bound {
            binds.push(directives.len());
        }
        // Only `::METHOD`/`::ATTRIBUTE`/`::ROUTINE` can carry a body, and each
        // already rejects one with its own specific error when its OWN shape
        // does not allow it (`::CONSTANT`'s body is 99.938, for one). Every
        // other directive kind never sets a body at all, and a clause trailing
        // one of those needs no special case here: the NEXT iteration's
        // `parse_directive` call sees a clause that does not start with `::` and
        // raises 99.916 on its own -- measured for all five kinds that can never
        // have a body (`::CLASS`, `::OPTIONS`, `::REQUIRES`, `::ANNOTATE`,
        // `::RESOURCE`).
        if let Some(slot) = directive_body(&mut directive.kind) {
            *slot = translate_block(ctx, cursor)?;
        }
        directives.push(directive);
    }
    Ok(main)
}

/// The body slot of a directive that carries one, for the assembler to fill.
pub(crate) fn directive_body(kind: &mut DirectiveKind) -> Option<&mut CodeBody> {
    match kind {
        DirectiveKind::Method(method) => method.body.as_deref_mut(),
        DirectiveKind::Attribute(attribute) => attribute.body.as_deref_mut(),
        DirectiveKind::Routine(routine) => routine.body.as_deref_mut(),
        DirectiveKind::Annotate(_)
        | DirectiveKind::Class(_)
        | DirectiveKind::Constant(_)
        | DirectiveKind::Options(_)
        | DirectiveKind::Requires(_)
        | DirectiveKind::Resource(_) => None,
    }
}

// The label table used to be built here, in a pass over the finished
// instruction list. It is built by `Block::add_clause` now, which is where
// labels are added, because a body's labels have to be its own and only the
// assembler knows where one body ends and the next begins.
