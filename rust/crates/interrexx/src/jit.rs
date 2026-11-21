//! Native code for a flat loop's passes, generated with Cranelift.
//!
//! A [`Pass`] describes one loop: the pass boundary, then each clause of the
//! body as an opening, its ops and a close. [`compile`] turns it into a
//! function that runs passes until something needs the driver.
//!
//! **Generated code touches one block of memory it owns and two arrays that
//! block points at, and nothing else.** The block is [`layout::Native`]: the
//! register file, a window onto the running frame's variable slots, the
//! clause countdown and the state of the clause being opened. Rust reads and
//! writes it as an ordinary struct; generated code reads and writes it at the
//! offsets [`layout`] derives from that same struct, so the two cannot come
//! to disagree about where a field is. Everything the interpreter owns is
//! reached through [`Host`], whose calls bracket each step with
//! [`Host::enter`], which writes the block back into the interpreter, and
//! [`Host::leave`], which reads it again. So Rust code only ever sees the
//! interpreter's own state, current, and generated code only ever sees the
//! block.
//!
//! The `unsafe` for this feature is all in this module, and its invariant is
//! local to it:
//!
//! 1. [`Compiled::run`] is the only caller of generated code. It passes a
//!    [`Ctx`] that lives on its own stack for the whole call and whose `host`
//!    is a live `&mut dyn Host`.
//! 2. Generated code reads and writes a register only at an index the
//!    [`Pass`] names, which [`compile`] checks is below [`Pass::registers`],
//!    and `run` checks the frame holds that many. Every bit pattern is a valid
//!    `ObjRef`.
//! 3. Generated code reads and writes a variable slot only at an index below
//!    `Native::slot_count`, through `Native::slots`, which [`Host::leave`]
//!    sets from `RootSet::top_window` after every bracketed call into the host
//!    and which nothing but a call into the host can invalidate; a null window
//!    is never dereferenced. It writes a slot only as `VarSlot::of` would. The
//!    unbracketed calls change nothing [`Host::leave`] reads:
//!    [`Host::push_arg`] appends to the argument stack and touches no frame,
//!    [`Host::quiet_builtin`] runs a builtin that changes nothing but the
//!    heap, given arguments that run no Rexx code (`crate::builtin::quiet`),
//!    and [`Host::quiet_compound`] reads or writes a compound variable whose
//!    tail runs no Rexx code, which changes a stem, the heap and, for a stem
//!    not yet set, the stem's own slot, never the window. Debug builds
//!    check the window after the first and the whole block after the others.
//! 4. The helpers are the only code that turns the `Ctx` pointer back into a
//!    reference, and only while generated code is suspended in that call.
//! 5. The machine code lives in the [`JITModule`] that [`Compiled`] owns,
//!    which frees it when dropped. Its code runs only inside `run`, whose
//!    `&self` keeps the `Compiled` alive for the call, and `entry` is never
//!    copied out of it.

#![allow(unsafe_code)]

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{AbiParam, Block, InstBuilder, MemFlagsData, Signature, Value, types};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module, default_libcall_names};
use interrexx_core::RegFrame;
use std::any::Any;
use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

pub(crate) mod layout;

use layout::Native;

/// A helper's answer that lets the generated code carry on.
pub(crate) const CONTINUE: u32 = 0;
/// Any other answer ends the run; the host has recorded what happened.
pub(crate) const STOP: u32 = 1;
/// An `IF`'s condition did not hold, and the host closed its clause: the
/// generated code goes on at the branch's target.
pub(crate) const BRANCH: u32 = 2;
/// The host ran an op between clauses that answered where the program goes
/// on, `Native::jump_to`; an op with no label in the pass leaves for the
/// driver there, which the host has already recorded.
pub(crate) const JUMP: u32 = 3;
/// [`Host::release`]'s answer: the op ran in a clause generated code opened,
/// and the host handed the clause back with nothing queued at its boundary,
/// so its close is generated code's again. The helper records it in
/// `Native::opened` and answers [`CONTINUE`]; generated code never sees it.
pub(crate) const FAST: u32 = 4;

/// What the generated code calls back into. Each step is bracketed by
/// [`Host::enter`] and [`Host::leave`].
pub(crate) trait Host {
    /// Writes `native`'s state back into the interpreter.
    fn enter(&mut self, native: &mut Native);
    /// Reads the interpreter's state into `native`.
    fn leave(&mut self, native: &mut Native);
    /// The pass boundary: count the header clause and step the loop.
    fn pass(&mut self) -> u32;
    /// Opens the clause whose `Op::Clause` is at `pc`.
    fn open(&mut self, pc: u32) -> u32;
    /// Takes over the clause at `pc` that generated code opened, so that the
    /// interpreter closes it.
    fn adopt(&mut self, pc: u32) -> u32;
    /// Closes the open clause, whose region ends at `next`.
    fn close(&mut self, next: u32) -> u32;
    /// Runs the op at `pc` the way the driver does.
    fn op(&mut self, pc: u32) -> u32;
    /// Hands back the clause [`Host::adopt`] took, answering [`FAST`], where
    /// the driver's hot exit would close it plainly; [`CONTINUE`] otherwise.
    fn release(&mut self) -> u32;
    /// `Op::PushArg`: register `src`, or an omitted argument, onto the
    /// argument stack. Reads nothing the block holds and changes nothing it
    /// does, so it is called with no bracket.
    fn push_arg(&mut self, src: u16);
    /// Whether the block's slot window is still the running frame's: what
    /// [`Host::leave`] would set it to.
    fn window_holds(&mut self, native: &Native) -> bool;
    /// `Op::CallArgs` at `pc` over the registers `args` carries
    /// ([`carry_arg`]), where it still resolves to a quiet builtin
    /// (`crate::builtin::quiet`), the arguments are plain and the call
    /// succeeds: its answer written to the op's register, and `true`. Called
    /// with no bracket, and so it reads nothing the block holds (the clause is
    /// whoever's it was) and changes nothing [`Host::leave`] reads. Anywhere
    /// else it changes nothing at all and answers `false`.
    fn quiet_builtin(&mut self, pc: u32, args: u64) -> bool;
    /// Pushes the arguments `args` carries for `Op::CallArgs` at `pc`, as the
    /// `Op::PushArg`s the pass dropped would have.
    fn stage_args(&mut self, pc: u32, args: u64);
    /// The compound read or write at `pc`, where it runs no Rexx code: under
    /// no trace that shows values, with a tail of plain values, and for a
    /// read, a tail that is set (`Interp::read_compound_quiet`). `true` once
    /// it has run. Called with no bracket, under the same terms as
    /// [`Host::quiet_builtin`]; anywhere else it changes nothing and answers
    /// `false`.
    fn quiet_compound(&mut self, pc: u32) -> bool;
    /// Whether [`Host::leave`] would leave unchanged every field of `native`
    /// it does not take from the interpreter's clause state.
    fn refresh_holds(&mut self, native: &Native) -> bool;
}

/// One step of a pass.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Step {
    /// Opens the clause at `pc`. Generated code opens it itself when the
    /// block says the interpreter is steady and the clause is `tabled`, and
    /// the host reads its instruction, line and indent from the chunk's
    /// table.
    Open {
        pc: u32,
        tabled: bool,
        register_top: u16,
    },
    /// Closes the open clause, whose region ends at `next`.
    Close { next: u32 },
    /// Writes this value's bits to a register.
    Const { dst: u16, bits: u64 },
    /// `+`, `-` or `*` of two registers: done here when both are small
    /// integers whose result is one, else the op at `pc` through the host.
    Arith {
        pc: u32,
        op: Arith,
        lhs: u16,
        rhs: u16,
        dst: u16,
    },
    /// A simple variable's read, from its slot when the window holds it and
    /// it is set, else the op at `pc` through the host.
    Load { pc: u32, slot: u32, dst: u16 },
    /// A simple variable's write, to its slot when the window holds it, else
    /// the op at `pc` through the host.
    Store { pc: u32, slot: u32, src: u16 },
    /// The op at `pc`, through the host.
    Op { pc: u32 },
    /// `Op::CallArgs` at `pc`, compiled where its site resolved to a quiet
    /// builtin: [`Host::quiet_builtin`] over the argument registers `args`
    /// carries ([`carry_arg`]), whose pushes the pass dropped, else those
    /// arguments pushed and the op through the host.
    Builtin { pc: u32, args: u64 },
    /// A compound variable's read or write at `pc` (`Op::Load` of a
    /// compound, `Op::Store` to one): [`Host::quiet_compound`], else the op
    /// through the host.
    Compound { pc: u32 },
    /// A step the pass took out, which is never compiled.
    Dropped,
    /// `Op::PushArg` of register `src`.
    PushArg { src: u16 },
    /// An `IF`'s `Op::ConditionJump` at `pc` on register `reg`, the last op
    /// of its clause: falls through to the clause's close when the value
    /// holds, and otherwise closes the clause and goes on at `target`.
    Branch { pc: u32, reg: u16, target: u32 },
    /// `Op::Jump` to `target`.
    Jump { target: u32 },
    /// `Op::EndBranch` at `pc`.
    EndBranch { pc: u32 },
    /// The `Op::LoopRun` of an unlabelled `do; ... end` block at `pc`: one
    /// clause counted at the line in force, as for [`Step::EndBranch`],
    /// while the block is steady; the op through the host otherwise.
    BlockStart { pc: u32 },
    /// An op between clause regions the host runs, which may answer
    /// [`JUMP`]: a `SELECT`'s own ops.
    Between { pc: u32 },
    /// A comparison of two registers: done here when both are small
    /// integers the driver's own quick comparison takes, else the op at `pc`
    /// through the host.
    Compare {
        pc: u32,
        op: Compare,
        lhs: u16,
        rhs: u16,
        dst: u16,
    },
}

/// The comparisons [`Step::Compare`] does itself, as `small_int_compare`
/// answers them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Compare {
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
}

/// The operators [`Step::Arith`] does itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Arith {
    Add,
    Subtract,
    Multiply,
    /// `%`: the quotient truncated, as `checked_div` answers it.
    IntDiv,
    /// `//`: the remainder with the dividend's sign, as `checked_rem`.
    Remainder,
}

/// One loop's passes: the boundary, then `steps`, then the boundary again.
#[derive(Debug)]
pub(crate) struct Pass {
    pub(crate) steps: Vec<Step>,
    /// How many registers the frame the code runs against holds.
    pub(crate) registers: u16,
    /// The pass boundary's own op, which a branch to it reaches.
    pub(crate) end: u32,
}

/// What a helper is handed: the block, the host it belongs to, and a panic
/// a helper caught, which [`Compiled::run`] resumes once generated code has
/// returned. A panic cannot unwind through generated code, and an
/// `extern "C"` helper that let one escape would abort the process.
#[repr(C)]
struct Ctx<'h> {
    native: Native,
    host: &'h mut dyn Host,
    panic: Option<Box<dyn Any + Send>>,
}

/// Runs `step` with the `Ctx` behind `ctx`, answering [`STOP`] and keeping the
/// panic if it panics.
fn guarded(ctx: *mut Ctx<'_>, step: impl FnOnce(&mut Ctx<'_>) -> u32) -> u32 {
    // SAFETY: property 4 of the module doc.
    let ctx = unsafe { &mut *ctx };
    match catch_unwind(AssertUnwindSafe(|| step(&mut *ctx))) {
        Ok(answer) => answer,
        Err(panic) => {
            ctx.panic = Some(panic);
            STOP
        }
    }
}

/// Runs `step` on the host between [`Host::enter`] and [`Host::leave`]. The
/// clause is the host's from here: `step` opens, closes or steps past it.
fn bracket(ctx: *mut Ctx<'_>, step: impl FnOnce(&mut dyn Host) -> u32) -> u32 {
    guarded(ctx, |ctx| {
        ctx.native.opened = 0;
        ctx.host.enter(&mut ctx.native);
        let answer = step(&mut *ctx.host);
        ctx.host.leave(&mut ctx.native);
        answer
    })
}

extern "C" fn pass_helper(ctx: *mut Ctx<'_>) -> u32 {
    bracket(ctx, |host| host.pass())
}

extern "C" fn open_helper(ctx: *mut Ctx<'_>, pc: u32) -> u32 {
    bracket(ctx, |host| host.open(pc))
}

/// What [`Step::PushArg`] carries for an omitted argument (`Op::ARG_OMITTED`).
pub(crate) const OMITTED: u16 = crate::ir::Op::ARG_OMITTED;

/// `adopt` names no clause: the op runs in one the host already has open.
const NO_ADOPT: u32 = u32::MAX;

extern "C" fn close_helper(ctx: *mut Ctx<'_>, next: u32) -> u32 {
    bracket(ctx, |host| host.close(next))
}

/// The op at `pc`, after the host takes over the clause at `clause` where
/// generated code opened it (`Native::opened`): one call for both. A clause
/// the host hands back is generated code's again, and the call answers
/// [`CONTINUE`] for it.
extern "C" fn op_helper(ctx: *mut Ctx<'_>, clause: u32, pc: u32) -> u32 {
    guarded(ctx, |ctx| run_op(ctx, clause, pc))
}

/// How many arguments a [`Step::Builtin`] carries at most: one register
/// each, in 16 bits of a `u64`.
pub(crate) const CARRIED_ARGS: usize = 4;

/// A [`Step::Builtin`]'s `args` before any is carried.
pub(crate) const NO_ARGS: u64 = u64::MAX;

/// `args` with argument `k`, below [`CARRIED_ARGS`], in register `src`, or
/// omitted where `src` is [`OMITTED`].
pub(crate) const fn carry_arg(args: u64, k: usize, src: u16) -> u64 {
    let shift = 16 * k as u32;
    (args & !(0xffff << shift)) | ((src as u64) << shift)
}

/// The register [`carry_arg`] put at `k` in `args`.
pub(crate) const fn carried_arg(args: u64, k: usize) -> u16 {
    (args >> (16 * k as u32)) as u16
}

/// [`Step::Builtin`]: the quiet path with no bracket (property 3 of the module
/// doc), and otherwise the arguments pushed and the op's own path.
extern "C" fn builtin_helper(ctx: *mut Ctx<'_>, clause: u32, pc: u32, args: u64) -> u32 {
    guarded(ctx, |ctx| {
        if ctx.host.quiet_builtin(pc, args) {
            debug_assert!(
                ctx.host.refresh_holds(&ctx.native),
                "a quiet builtin changed what the block holds"
            );
            return CONTINUE;
        }
        ctx.host.stage_args(pc, args);
        run_op(ctx, clause, pc)
    })
}

/// [`Step::Compound`]: the quiet path with no bracket (property 3 of the
/// module doc), and otherwise the op's own path.
extern "C" fn compound_helper(ctx: *mut Ctx<'_>, clause: u32, pc: u32) -> u32 {
    guarded(ctx, |ctx| {
        if ctx.host.quiet_compound(pc) {
            debug_assert!(
                ctx.host.refresh_holds(&ctx.native),
                "a quiet compound access changed what the block holds"
            );
            return CONTINUE;
        }
        run_op(ctx, clause, pc)
    })
}

/// [`op_helper`]'s body.
fn run_op(ctx: &mut Ctx<'_>, clause: u32, pc: u32) -> u32 {
    let adopt = clause != NO_ADOPT && ctx.native.opened != 0;
    ctx.native.opened = 0;
    ctx.host.enter(&mut ctx.native);
    let host = &mut *ctx.host;
    let answer = match if adopt { host.adopt(clause) } else { CONTINUE } {
        CONTINUE => match host.op(pc) {
            CONTINUE if adopt => host.release(),
            answer => answer,
        },
        answer => answer,
    };
    ctx.host.leave(&mut ctx.native);
    if answer == FAST {
        ctx.native.opened = 1;
        CONTINUE
    } else {
        answer
    }
}

/// A helper that is not bracketed (property 3 of the module doc).
extern "C" fn push_arg_helper(ctx: *mut Ctx<'_>, src: u32) -> u32 {
    guarded(ctx, |ctx| {
        ctx.host.push_arg(src as u16);
        debug_assert!(
            ctx.host.window_holds(&ctx.native),
            "a push onto the argument stack moved the slot window"
        );
        CONTINUE
    })
}

type Entry = extern "C" fn(*mut Ctx<'_>) -> u32;

/// A compiled pass.
pub(crate) struct Compiled {
    /// Owns the code `entry` points into (property 5), and frees it when
    /// dropped.
    module: Option<JITModule>,
    entry: Entry,
    registers: u16,
}

impl Drop for Compiled {
    fn drop(&mut self) {
        if let Some(module) = self.module.take() {
            // SAFETY: property 5 of the module doc: `entry` is the only pointer
            // into the module, it is called only through `run`'s `&self`, and
            // so no code of this module is running and none is called again.
            unsafe { module.free_memory() };
        }
    }
}

impl Compiled {
    /// Runs passes against `registers` until a helper answers [`STOP`].
    pub(crate) fn run(&self, registers: RegFrame<'_>, host: &mut dyn Host) -> u32 {
        assert!(
            registers.len() >= self.registers,
            "a compiled pass run against a frame smaller than the one it was compiled for"
        );
        let mut native = Native::new(registers.as_ptr().cast());
        host.leave(&mut native);
        let mut ctx = Ctx {
            native,
            host,
            panic: None,
        };
        let answer = (self.entry)(&mut ctx);
        if let Some(panic) = ctx.panic.take() {
            resume_unwind(panic);
        }
        // Every exit is a helper's answer, so `enter` has already written
        // the block back; this is the same call made idempotent.
        ctx.host.enter(&mut ctx.native);
        answer
    }
}

/// The code generator's own state while it emits one pass.
struct Emit<'a, 'b> {
    b: FunctionBuilder<'b>,
    pointer: types::Type,
    ctx: Value,
    registers: Value,
    exit: cranelift_codegen::ir::Block,
    bare: cranelift_codegen::ir::SigRef,
    with_pc: cranelift_codegen::ir::SigRef,
    with_two: cranelift_codegen::ir::SigRef,
    /// `with_two`'s, and a `u64` after.
    with_args: cranelift_codegen::ir::SigRef,
    /// Whether the open clause was opened by generated code and not yet
    /// adopted, which is what makes its close free. `Native::opened` carries
    /// it into an op's call into the host and back out.
    fast: Variable,
    _module: std::marker::PhantomData<&'a ()>,
}

impl Emit<'_, '_> {
    fn flags() -> MemFlagsData {
        MemFlagsData::trusted()
    }

    fn load_native(&mut self, ty: types::Type, at: i32) -> Value {
        self.b.ins().load(ty, Self::flags(), self.ctx, at)
    }

    fn store_native(&mut self, value: Value, at: i32) {
        self.b.ins().store(Self::flags(), value, self.ctx, at);
    }

    fn register_at(register: u16) -> i32 {
        i32::from(register) * 8
    }

    /// Calls `helper`, and leaves for `exit` unless it answers CONTINUE.
    fn call(&mut self, helper: *const (), pc: Option<u32>) {
        let address = self.b.ins().iconst(self.pointer, helper as usize as i64);
        let answer = match pc {
            Some(pc) => {
                let pc = self.b.ins().iconst(types::I32, i64::from(pc));
                let call = self
                    .b
                    .ins()
                    .call_indirect(self.with_pc, address, &[self.ctx, pc]);
                self.b.inst_results(call)[0]
            }
            None => {
                let call = self.b.ins().call_indirect(self.bare, address, &[self.ctx]);
                self.b.inst_results(call)[0]
            }
        };
        let on = self.b.create_block();
        self.b
            .ins()
            .brif(answer, self.exit, &[answer.into()], on, &[]);
        self.b.switch_to_block(on);
    }

    /// Calls `op_helper` for the op at `pc`, which adopts the clause at
    /// `clause_pc` first where generated code opened it, and leaves for `exit`
    /// unless the answer is CONTINUE or, with `also`, that answer and block.
    fn call_op(&mut self, clause_pc: Option<u32>, pc: u32, also: Option<(u32, Block)>) {
        let helper = op_helper as *const ();
        self.call_op_with(helper, None, clause_pc, pc, also);
    }

    /// [`Emit::call_op`] through `helper`, which takes the same arguments
    /// and, with `args`, that `u64` after them.
    fn call_op_with(
        &mut self,
        helper: *const (),
        args: Option<u64>,
        clause_pc: Option<u32>,
        pc: u32,
        also: Option<(u32, Block)>,
    ) {
        if clause_pc.is_some() {
            let fast = self.b.use_var(self.fast);
            self.store_native(fast, layout::OPENED);
        }
        let clause = clause_pc.unwrap_or(NO_ADOPT);
        let address = self.b.ins().iconst(self.pointer, helper as usize as i64);
        let clause = self.b.ins().iconst(types::I32, i64::from(clause));
        let pc = self.b.ins().iconst(types::I32, i64::from(pc));
        let call = match args {
            Some(args) => {
                let args = self.b.ins().iconst(types::I64, args as i64);
                self.b
                    .ins()
                    .call_indirect(self.with_args, address, &[self.ctx, clause, pc, args])
            }
            None => self
                .b
                .ins()
                .call_indirect(self.with_two, address, &[self.ctx, clause, pc]),
        };
        let answer = self.b.inst_results(call)[0];
        // The clause is the host's now, unless the op handed it back.
        let fast = if clause_pc.is_some() {
            self.load_native(types::I64, layout::OPENED)
        } else {
            self.b.ins().iconst(types::I64, 0)
        };
        self.b.def_var(self.fast, fast);
        if let Some((code, block)) = also {
            let matches = self
                .b
                .ins()
                .icmp_imm_u(IntCC::Equal, answer, i64::from(code));
            let other = self.b.create_block();
            self.b.ins().brif(matches, block, &[], other, &[]);
            self.b.switch_to_block(other);
        }
        let on = self.b.create_block();
        self.b
            .ins()
            .brif(answer, self.exit, &[answer.into()], on, &[]);
        self.b.switch_to_block(on);
    }

    /// Whether generated code opened the running clause.
    fn opened(&mut self) -> Value {
        self.b.use_var(self.fast)
    }

    /// Records that the running clause is closed.
    fn clear_opened(&mut self) {
        let no = self.b.ins().iconst(types::I64, 0);
        self.b.def_var(self.fast, no);
    }

    fn register(&mut self, register: u16) -> Value {
        self.b.ins().load(
            types::I64,
            Self::flags(),
            self.registers,
            Self::register_at(register),
        )
    }

    fn set_register(&mut self, register: u16, value: Value) {
        self.b.ins().store(
            Self::flags(),
            value,
            self.registers,
            Self::register_at(register),
        );
    }

    /// Whether both handles carry the small-integer tag.
    fn both_small(&mut self, left: Value, right: Value) -> Value {
        let tags = self.b.ins().band(left, right);
        let tags = self.b.ins().band_imm_u(tags, layout::TAG_MASK);
        let either = self.b.ins().bor(left, right);
        let either = self
            .b
            .ins()
            .band_imm_u(either, layout::TAG_MASK & !layout::TAG_INT);
        let tagged = self.b.ins().icmp_imm_u(IntCC::Equal, tags, layout::TAG_INT);
        let clear = self.b.ins().icmp_imm_u(IntCC::Equal, either, 0);
        self.b.ins().band(tagged, clear)
    }

    /// Whether `v` is strictly inside `(-bound, bound)`.
    fn within(&mut self, v: Value, bound: Value) -> Value {
        let negative = self.b.ins().ineg(bound);
        let below = self.b.ins().icmp(IntCC::SignedLessThan, v, bound);
        let above = self.b.ins().icmp(IntCC::SignedGreaterThan, v, negative);
        self.b.ins().band(below, above)
    }

    /// The op at `pc` through the host, from a clause generated code may have
    /// opened itself: adopted first, so the interpreter can close it.
    fn slow_op(&mut self, clause_pc: u32, pc: u32) {
        self.call_op(Some(clause_pc), pc, None);
    }

    /// Branches to `slow` unless the window holds slot `slot`, answering the
    /// slot's address.
    fn slot_address(&mut self, slot: u32, slow: cranelift_codegen::ir::Block) -> Value {
        let base = self.load_native(self.pointer, layout::SLOTS);
        let count = self.load_native(types::I64, layout::SLOT_COUNT);
        let held = self
            .b
            .ins()
            .icmp_imm_u(IntCC::UnsignedGreaterThan, count, i64::from(slot));
        // A null window answers a count of zero, so `held` covers it.
        let on = self.b.create_block();
        self.b.ins().brif(held, on, &[], slow, &[]);
        self.b.switch_to_block(on);
        self.b
            .ins()
            .iadd_imm_u(base, i64::from(slot) * layout::SLOT_SIZE)
    }
}

/// Generates the code for `pass`, or `None` where Cranelift cannot target
/// this host or refuses the function. Never inlined: it runs once per loop,
/// and kept apart it is one frame in a profile.
#[inline(never)]
pub(crate) fn compile(pass: &Pass) -> Option<Compiled> {
    for step in &pass.steps {
        let named = match *step {
            Step::Const { dst, .. } | Step::Load { dst, .. } => Some(dst),
            Step::Store { src, .. } => Some(src),
            Step::Arith { lhs, rhs, dst, .. } | Step::Compare { lhs, rhs, dst, .. } => {
                Some(lhs.max(rhs).max(dst))
            }
            Step::Branch { reg, .. } => Some(reg),
            Step::Open { .. }
            | Step::Close { .. }
            | Step::Op { .. }
            | Step::Jump { .. }
            | Step::Compound { .. }
            | Step::BlockStart { .. }
            | Step::Dropped
            | Step::EndBranch { .. }
            | Step::Between { .. } => None,
            Step::PushArg { src } => (src != OMITTED).then_some(src),
            Step::Builtin { args, .. } => (0..CARRIED_ARGS)
                .map(|k| carried_arg(args, k))
                .filter(|&src| src != OMITTED)
                .max(),
        };
        // Property 2 of the module doc.
        if named.is_some_and(|register| register >= pass.registers) {
            return None;
        }
    }
    let mut flags = settings::builder();
    // Compile time is paid on the first hot entry and has to be earned back
    // by the passes after it. The mid-end finds little in code that is mostly
    // guards and calls; the register allocator is what the code's speed
    // rests on, so it keeps its default.
    flags.set("opt_level", "none").ok()?;
    flags.set("is_pic", "false").ok()?;
    // The verifier checks this generator rather than the program, and costs
    // more than the code it checks; debug builds keep it.
    if !cfg!(debug_assertions) {
        flags.set("enable_verifier", "false").ok()?;
    }
    let isa = cranelift_native::builder()
        .ok()?
        .finish(settings::Flags::new(flags))
        .ok()?;
    let mut module = JITModule::new(JITBuilder::with_isa(isa, default_libcall_names()));
    let pointer = module.target_config().pointer_type();
    if pointer != types::I64 {
        return None;
    }

    let mut signature = module.make_signature();
    signature.params.push(AbiParam::new(pointer));
    signature.returns.push(AbiParam::new(types::I32));
    let id = module
        .declare_function("rexx_pass", Linkage::Local, &signature)
        .ok()?;

    let mut context = module.make_context();
    context.func.signature = signature;
    let mut builder_context = FunctionBuilderContext::new();
    let call_conv = module.target_config().default_call_conv;
    let helper_signature = |with_pc: bool| {
        let mut s = Signature::new(call_conv);
        s.params.push(AbiParam::new(pointer));
        if with_pc {
            s.params.push(AbiParam::new(types::I32));
        }
        s.returns.push(AbiParam::new(types::I32));
        s
    };
    {
        let mut b = FunctionBuilder::new(&mut context.func, &mut builder_context);
        let bare = b.import_signature(helper_signature(false));
        let with_pc = b.import_signature(helper_signature(true));
        let with_two = {
            let mut s = helper_signature(true);
            s.params.push(AbiParam::new(types::I32));
            b.import_signature(s)
        };
        let with_args = {
            let mut s = helper_signature(true);
            s.params.push(AbiParam::new(types::I32));
            s.params.push(AbiParam::new(types::I64));
            b.import_signature(s)
        };

        let entry = b.create_block();
        let head = b.create_block();
        let exit = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.append_block_param(exit, types::I32);

        let fast = b.declare_var(types::I64);
        b.switch_to_block(entry);
        let ctx = b.block_params(entry)[0];
        let registers = b.ins().load(pointer, Emit::flags(), ctx, layout::REGISTERS);
        let no = b.ins().iconst(types::I64, 0);
        b.def_var(fast, no);
        b.ins().jump(head, &[]);

        let mut e = Emit {
            b,
            pointer,
            ctx,
            registers,
            exit,
            bare,
            with_pc,
            with_two,
            with_args,
            fast,
            _module: std::marker::PhantomData,
        };
        // Every op a branch can land on: a clause's opening, an
        // `EndBranch`, and the pass boundary.
        let mut labels: HashMap<u32, Block> = HashMap::new();
        labels.insert(pass.end, head);
        for step in &pass.steps {
            if let Step::Open { pc, .. } | Step::EndBranch { pc } | Step::Between { pc } = *step {
                labels.insert(pc, e.b.create_block());
            }
        }
        for step in &pass.steps {
            if let Step::Branch { target, .. } | Step::Jump { target } = *step
                && !labels.contains_key(&target)
            {
                return None;
            }
        }
        let dispatch = e.b.create_block();
        e.b.switch_to_block(head);
        pass_step(&mut e);
        let mut clause_pc = 0;
        // Whether the block being emitted into can still fall through.
        let mut live = true;
        for step in &pass.steps {
            if let Step::Open { pc, .. } | Step::EndBranch { pc } | Step::Between { pc } = *step {
                let block = labels[&pc];
                if live {
                    e.b.ins().jump(block, &[]);
                }
                e.b.switch_to_block(block);
                live = true;
            } else if !live {
                let unreachable = e.b.create_block();
                e.b.switch_to_block(unreachable);
                live = true;
            }
            match *step {
                Step::Open {
                    pc,
                    tabled,
                    register_top,
                } => {
                    clause_pc = pc;
                    open(&mut e, pc, tabled, register_top, pass.registers);
                }
                Step::Close { next } => {
                    let opened = e.opened();
                    let slow = e.b.create_block();
                    let done = e.b.create_block();
                    let free = e.b.create_block();
                    e.b.ins().brif(opened, free, &[], slow, &[]);
                    e.b.switch_to_block(free);
                    e.clear_opened();
                    e.b.ins().jump(done, &[]);
                    e.b.switch_to_block(slow);
                    e.call(close_helper as *const (), Some(next));
                    e.clear_opened();
                    e.b.ins().jump(done, &[]);
                    e.b.switch_to_block(done);
                }
                Step::Branch { pc, reg, target } => {
                    branch(&mut e, clause_pc, pc, reg, labels[&target], target);
                }
                Step::Jump { target } => {
                    e.b.ins().jump(labels[&target], &[]);
                    live = false;
                }
                Step::EndBranch { pc } => end_branch(&mut e, None, pc),
                Step::BlockStart { pc } => end_branch(&mut e, Some(clause_pc), pc),
                Step::PushArg { src } => {
                    let address =
                        e.b.ins()
                            .iconst(e.pointer, push_arg_helper as *const () as usize as i64);
                    let src = e.b.ins().iconst(types::I32, i64::from(src));
                    e.b.ins().call_indirect(e.with_pc, address, &[e.ctx, src]);
                }
                Step::Between { pc } => {
                    e.call_op(None, pc, Some((JUMP, dispatch)));
                }
                Step::Compare {
                    pc,
                    op,
                    lhs,
                    rhs,
                    dst,
                } => compare(&mut e, clause_pc, pc, op, lhs, rhs, dst),
                Step::Op { pc } => e.slow_op(clause_pc, pc),
                Step::Builtin { pc, args } => e.call_op_with(
                    builtin_helper as *const (),
                    Some(args),
                    Some(clause_pc),
                    pc,
                    None,
                ),
                Step::Compound { pc } => e.call_op_with(
                    compound_helper as *const (),
                    None,
                    Some(clause_pc),
                    pc,
                    None,
                ),
                Step::Dropped => {}
                Step::Const { dst, bits } => {
                    let value = e.b.ins().iconst(types::I64, bits as i64);
                    e.b.ins()
                        .store(Emit::flags(), value, e.registers, Emit::register_at(dst));
                }
                Step::Load { pc, slot, dst } => {
                    let slow = e.b.create_block();
                    let join = e.b.create_block();
                    let address = e.slot_address(slot, slow);
                    let set =
                        e.b.ins()
                            .load(types::I64, Emit::flags(), address, layout::SLOT_SET_AT);
                    let on = e.b.create_block();
                    e.b.ins().brif(set, on, &[], slow, &[]);
                    e.b.switch_to_block(on);
                    let value =
                        e.b.ins()
                            .load(types::I64, Emit::flags(), address, layout::SLOT_VALUE_AT);
                    e.b.ins()
                        .store(Emit::flags(), value, e.registers, Emit::register_at(dst));
                    e.b.ins().jump(join, &[]);
                    e.b.switch_to_block(slow);
                    e.slow_op(clause_pc, pc);
                    e.b.ins().jump(join, &[]);
                    e.b.switch_to_block(join);
                }
                Step::Store { pc, slot, src } => {
                    let slow = e.b.create_block();
                    let join = e.b.create_block();
                    let address = e.slot_address(slot, slow);
                    let value = e.b.ins().load(
                        types::I64,
                        Emit::flags(),
                        e.registers,
                        Emit::register_at(src),
                    );
                    let one = e.b.ins().iconst(types::I64, 1);
                    e.b.ins()
                        .store(Emit::flags(), one, address, layout::SLOT_SET_AT);
                    e.b.ins()
                        .store(Emit::flags(), value, address, layout::SLOT_VALUE_AT);
                    e.b.ins().jump(join, &[]);
                    e.b.switch_to_block(slow);
                    e.slow_op(clause_pc, pc);
                    e.b.ins().jump(join, &[]);
                    e.b.switch_to_block(join);
                }
                Step::Arith {
                    pc,
                    op,
                    lhs,
                    rhs,
                    dst,
                } => arith(&mut e, clause_pc, pc, op, lhs, rhs, dst),
            }
        }
        if live {
            e.b.ins().jump(head, &[]);
        }

        // Where a JUMP answer lands: the label of the op it names, or the
        // driver, at the op the host recorded.
        e.b.switch_to_block(dispatch);
        let to = e.load_native(types::I64, layout::JUMP_TO);
        let mut targets: Vec<(u32, Block)> =
            labels.iter().map(|(&pc, &block)| (pc, block)).collect();
        targets.sort_unstable_by_key(|&(pc, _)| pc);
        for (pc, block) in targets {
            let next = e.b.create_block();
            let here = e.b.ins().icmp_imm_u(IntCC::Equal, to, i64::from(pc));
            e.b.ins().brif(here, block, &[], next, &[]);
            e.b.switch_to_block(next);
        }
        let stop = e.b.ins().iconst(types::I32, i64::from(STOP));
        e.b.ins().jump(exit, &[stop.into()]);

        e.b.switch_to_block(exit);
        let answer = e.b.block_params(exit)[0];
        e.b.ins().return_(&[answer]);
        e.b.seal_all_blocks();
        e.b.finalize(module.target_config());
    }
    // A function Cranelift refuses is this generator's fault, never the
    // program's: loud where the verifier runs, and a loop left to the driver
    // where it does not.
    if let Err(error) = module.define_function(id, &mut context) {
        debug_assert!(false, "Cranelift refused a compiled pass: {error:?}");
        return None;
    }
    module.clear_context(&mut context);
    module.finalize_definitions().ok()?;
    let code = module.get_finalized_function(id);
    // SAFETY: `code` is the function just defined with `signature`, which is
    // `Entry`'s: one pointer in, one `u32` out, the platform's C convention.
    let entry = unsafe { std::mem::transmute::<*const u8, Entry>(code) };
    Some(Compiled {
        module: Some(module),
        entry,
        registers: pass.registers,
    })
}

/// A clause's opening: by generated code when the block is steady, the
/// clause is tabled and the countdown has more than this clause left in it;
/// otherwise by the host.
fn open(e: &mut Emit<'_, '_>, pc: u32, tabled: bool, register_top: u16, registers: u16) {
    let slow = e.b.create_block();
    let opened = e.b.create_block();
    if tabled {
        let steady = e.load_native(types::I64, layout::STEADY);
        let countdown = e.load_native(types::I64, layout::COUNTDOWN);
        let left =
            e.b.ins()
                .icmp_imm_u(IntCC::UnsignedGreaterThan, countdown, 1);
        let ready = e.b.ins().icmp_imm_u(IntCC::NotEqual, steady, 0);
        let ready = e.b.ins().band(ready, left);
        let fast = e.b.create_block();
        e.b.ins().brif(ready, fast, &[], slow, &[]);

        e.b.switch_to_block(fast);
        let countdown = e.b.ins().iadd_imm_s(countdown, -1);
        e.store_native(countdown, layout::COUNTDOWN);
        let at = e.b.ins().iconst(types::I64, i64::from(pc));
        e.store_native(at, layout::CLAUSE_AT);
        clear_registers(e, register_top, registers);
        let yes = e.b.ins().iconst(types::I64, 1);
        e.b.def_var(e.fast, yes);
        e.b.ins().jump(opened, &[]);
    } else {
        e.b.ins().jump(slow, &[]);
    }
    e.b.switch_to_block(slow);
    e.call(open_helper as *const (), Some(pc));
    e.clear_opened();
    e.b.ins().jump(opened, &[]);
    e.b.switch_to_block(opened);
}

/// The pass boundary: the loop's quick pass by generated code when the block
/// offers one, the countdown has more than this clause left in it, and the
/// pass carries on; otherwise the host's, which also ends the loop.
fn pass_step(e: &mut Emit<'_, '_>) {
    let passes = e.load_native(types::I64, layout::PASSES);
    let passes = e.b.ins().iadd_imm_u(passes, 1);
    e.store_native(passes, layout::PASSES);
    let slow = e.b.create_block();
    let body = e.b.create_block();
    let check = e.b.create_block();
    let read = e.b.create_block();
    let decode = e.b.create_block();
    let take = e.b.create_block();

    let ready = e.load_native(types::I64, layout::LOOP_READY);
    let countdown = e.load_native(types::I64, layout::COUNTDOWN);
    let left =
        e.b.ins()
            .icmp_imm_u(IntCC::UnsignedGreaterThan, countdown, 1);
    let ready = e.b.ins().icmp_imm_u(IntCC::NotEqual, ready, 0);
    let ready = e.b.ins().band(ready, left);
    e.b.ins().brif(ready, check, &[], slow, &[]);

    // `DO expr` and `DO FOREVER` share the header clause's tail with the
    // controlled step, which `take` is.
    let counted = e.b.create_block();
    let not_counted = e.b.create_block();
    let forever = e.b.create_block();
    let tail = e.b.create_block();
    e.b.switch_to_block(check);
    let kind = e.load_native(types::I64, layout::LOOP_KIND);
    let is_count =
        e.b.ins()
            .icmp_imm_u(IntCC::Equal, kind, layout::LOOP_COUNT as i64);
    e.b.ins().brif(is_count, counted, &[], not_counted, &[]);
    e.b.switch_to_block(not_counted);
    let is_forever =
        e.b.ins()
            .icmp_imm_u(IntCC::Equal, kind, layout::LOOP_FOREVER as i64);
    let controlled = e.b.create_block();
    e.b.ins().brif(is_forever, forever, &[], controlled, &[]);

    // One fewer pass left; none left is the loop ending, the host's.
    e.b.switch_to_block(counted);
    let left = e.load_native(types::I64, layout::LOOP_VALUE);
    let more = e.b.create_block();
    e.b.ins().brif(left, more, &[], slow, &[]);
    e.b.switch_to_block(more);
    let left = e.b.ins().iadd_imm_s(left, -1);
    e.store_native(left, layout::LOOP_VALUE);
    let one = e.b.ins().iconst(types::I64, 1);
    e.store_native(one, layout::LOOP_DIRTY);
    e.b.ins().jump(tail, &[]);

    e.b.switch_to_block(forever);
    e.b.ins().jump(tail, &[]);

    e.b.switch_to_block(controlled);
    let slot = e.load_native(types::I64, layout::LOOP_SLOT);
    let count = e.load_native(types::I64, layout::SLOT_COUNT);
    let held = e.b.ins().icmp(IntCC::UnsignedLessThan, slot, count);
    e.b.ins().brif(held, read, &[], slow, &[]);

    e.b.switch_to_block(read);
    let base = e.load_native(e.pointer, layout::SLOTS);
    let offset = e.b.ins().imul_imm_u(slot, layout::SLOT_SIZE);
    let address = e.b.ins().iadd(base, offset);
    let set =
        e.b.ins()
            .load(types::I64, Emit::flags(), address, layout::SLOT_SET_AT);
    e.b.ins().brif(set, decode, &[], slow, &[]);

    e.b.switch_to_block(decode);
    let value =
        e.b.ins()
            .load(types::I64, Emit::flags(), address, layout::SLOT_VALUE_AT);
    let tag = e.b.ins().band_imm_u(value, layout::TAG_MASK);
    let tagged = e.b.ins().icmp_imm_u(IntCC::Equal, tag, layout::TAG_INT);
    let running = e.b.ins().sshr_imm_u(value, layout::TAG_BITS);
    let step = e.load_native(types::I64, layout::LOOP_STEP);
    let (sum, overflow) = e.b.ins().sadd_overflow(running, step);
    let bound = e.load_native(types::I64, layout::SMALL_INT_BOUND);
    let negative = e.b.ins().ineg(bound);
    let mut ok = tagged;
    for v in [running, sum] {
        let below = e.b.ins().icmp(IntCC::SignedLessThan, v, bound);
        let above = e.b.ins().icmp(IntCC::SignedGreaterThan, v, negative);
        ok = e.b.ins().band(ok, below);
        ok = e.b.ins().band(ok, above);
    }
    let fine = e.b.ins().icmp_imm_u(IntCC::Equal, overflow, 0);
    ok = e.b.ins().band(ok, fine);
    // `loop_advance_quick`'s own answer: a descending step carries on while
    // the value is at or above the bound, any other while it is at or below.
    let to = e.load_native(types::I64, layout::LOOP_TO);
    let descending = e.b.ins().icmp_imm_u(IntCC::SignedLessThan, step, 0);
    let at_or_above = e.b.ins().icmp(IntCC::SignedGreaterThanOrEqual, sum, to);
    let at_or_below = e.b.ins().icmp(IntCC::SignedLessThanOrEqual, sum, to);
    let carries_on = e.b.ins().select(descending, at_or_above, at_or_below);
    ok = e.b.ins().band(ok, carries_on);
    e.b.ins().brif(ok, take, &[], slow, &[]);

    e.b.switch_to_block(take);
    let one = e.b.ins().iconst(types::I64, 1);
    e.b.ins()
        .store(Emit::flags(), one, address, layout::SLOT_SET_AT);
    let shifted = e.b.ins().ishl_imm_u(sum, layout::TAG_BITS);
    let handle = e.b.ins().bor_imm_u(shifted, layout::TAG_INT);
    e.b.ins()
        .store(Emit::flags(), handle, address, layout::SLOT_VALUE_AT);
    e.store_native(sum, layout::LOOP_VALUE);
    e.store_native(one, layout::LOOP_DIRTY);
    e.b.ins().jump(tail, &[]);

    // The header clause every quick pass runs in: a countdown tick, its line
    // and the indent it leaves.
    e.b.switch_to_block(tail);
    let countdown = e.load_native(types::I64, layout::COUNTDOWN);
    let countdown = e.b.ins().iadd_imm_s(countdown, -1);
    e.store_native(countdown, layout::COUNTDOWN);
    let header = e.b.ins().iconst(types::I64, layout::CLAUSE_HEADER as i64);
    e.store_native(header, layout::CLAUSE_AT);
    e.b.ins().jump(body, &[]);

    e.b.switch_to_block(slow);
    e.call(pass_helper as *const (), None);
    e.b.ins().jump(body, &[]);

    e.b.switch_to_block(body);
}

/// `RegFrame::clear_from(top)`, for a frame of `registers`.
fn clear_registers(e: &mut Emit<'_, '_>, top: u16, registers: u16) {
    let nil = e.b.ins().iconst(types::I64, layout::NIL_BITS as i64);
    for register in top..registers {
        e.b.ins()
            .store(Emit::flags(), nil, e.registers, Emit::register_at(register));
    }
}

fn arith(e: &mut Emit<'_, '_>, clause_pc: u32, pc: u32, op: Arith, lhs: u16, rhs: u16, dst: u16) {
    let left = e.register(lhs);
    let right = e.register(rhs);
    let fast = e.b.create_block();
    let store = e.b.create_block();
    let slow = e.b.create_block();
    let join = e.b.create_block();
    e.b.append_block_param(store, types::I64);
    let both = e.both_small(left, right);
    e.b.ins().brif(both, fast, &[], slow, &[]);

    e.b.switch_to_block(fast);
    let bound = e.load_native(types::I64, layout::SMALL_INT_BOUND);
    let a = e.b.ins().sshr_imm_u(left, layout::TAG_BITS);
    let c = e.b.ins().sshr_imm_u(right, layout::TAG_BITS);
    // A division by zero is the driver's to raise.
    if matches!(op, Arith::IntDiv | Arith::Remainder) {
        let divides = e.b.create_block();
        e.b.ins().brif(c, divides, &[], slow, &[]);
        e.b.switch_to_block(divides);
    }
    let (result, overflow) = match op {
        Arith::Add => (e.b.ins().iadd(a, c), None),
        Arith::Subtract => (e.b.ins().isub(a, c), None),
        Arith::Multiply => {
            let (product, overflow) = e.b.ins().smul_overflow(a, c);
            (product, Some(overflow))
        }
        // Both operands are tagged, so neither is `i64::MIN` and the quotient
        // cannot overflow.
        Arith::IntDiv => (e.b.ins().sdiv(a, c), None),
        Arith::Remainder => (e.b.ins().srem(a, c), None),
    };
    let mut ok = e.within(a, bound);
    let c_ok = e.within(c, bound);
    ok = e.b.ins().band(ok, c_ok);
    let r_ok = e.within(result, bound);
    ok = e.b.ins().band(ok, r_ok);
    if let Some(overflow) = overflow {
        let fine = e.b.ins().icmp_imm_u(IntCC::Equal, overflow, 0);
        ok = e.b.ins().band(ok, fine);
    }
    e.b.ins().brif(ok, store, &[result.into()], slow, &[]);

    e.b.switch_to_block(store);
    let result = e.b.block_params(store)[0];
    let shifted = e.b.ins().ishl_imm_u(result, layout::TAG_BITS);
    let handle = e.b.ins().bor_imm_u(shifted, layout::TAG_INT);
    e.set_register(dst, handle);
    e.b.ins().jump(join, &[]);

    e.b.switch_to_block(slow);
    e.slow_op(clause_pc, pc);
    e.b.ins().jump(join, &[]);

    e.b.switch_to_block(join);
}

/// `small_int_compare`'s answer for two small integers within `DIGITS`, under
/// a `FUZZ` of 0 and no `TRACE` of results; the op through the host otherwise.
fn compare(
    e: &mut Emit<'_, '_>,
    clause_pc: u32,
    pc: u32,
    op: Compare,
    lhs: u16,
    rhs: u16,
    dst: u16,
) {
    let left = e.register(lhs);
    let right = e.register(rhs);
    let fast = e.b.create_block();
    let slow = e.b.create_block();
    let join = e.b.create_block();
    let both = e.both_small(left, right);
    let quick = e.load_native(types::I64, layout::QUICK_VALUES);
    let quick = e.b.ins().icmp_imm_u(IntCC::NotEqual, quick, 0);
    let both = e.b.ins().band(both, quick);
    e.b.ins().brif(both, fast, &[], slow, &[]);

    e.b.switch_to_block(fast);
    let bound = e.load_native(types::I64, layout::SMALL_INT_BOUND);
    let a = e.b.ins().sshr_imm_u(left, layout::TAG_BITS);
    let c = e.b.ins().sshr_imm_u(right, layout::TAG_BITS);
    let a_ok = e.within(a, bound);
    let c_ok = e.within(c, bound);
    let ok = e.b.ins().band(a_ok, c_ok);
    let answer = e.b.create_block();
    e.b.ins().brif(ok, answer, &[], slow, &[]);

    e.b.switch_to_block(answer);
    let cc = match op {
        Compare::Equal => IntCC::Equal,
        Compare::NotEqual => IntCC::NotEqual,
        Compare::Greater => IntCC::SignedGreaterThan,
        Compare::Less => IntCC::SignedLessThan,
        Compare::GreaterEqual => IntCC::SignedGreaterThanOrEqual,
        Compare::LessEqual => IntCC::SignedLessThanOrEqual,
    };
    let holds = e.b.ins().icmp(cc, a, c);
    let yes = e.b.ins().iconst(types::I64, layout::TRUE_BITS);
    let no = e.b.ins().iconst(types::I64, layout::FALSE_BITS);
    let value = e.b.ins().select(holds, yes, no);
    e.set_register(dst, value);
    e.b.ins().jump(join, &[]);

    e.b.switch_to_block(slow);
    e.slow_op(clause_pc, pc);
    e.b.ins().jump(join, &[]);

    e.b.switch_to_block(join);
}

/// An `IF`'s condition: the driver's quick test of a value that is already
/// logical, under no `TRACE` of results; the op through the host otherwise.
/// A condition that does not hold closes the clause as the driver's hot exit
/// does and goes on at `target`.
fn branch(
    e: &mut Emit<'_, '_>,
    clause_pc: u32,
    pc: u32,
    reg: u16,
    target_block: Block,
    target: u32,
) {
    let value = e.register(reg);
    let quick = e.load_native(types::I64, layout::QUICK_VALUES);
    let quick = e.b.ins().icmp_imm_u(IntCC::NotEqual, quick, 0);
    let is_true = e.b.ins().icmp_imm_u(IntCC::Equal, value, layout::TRUE_BITS);
    let is_one = e.b.ins().icmp_imm_u(IntCC::Equal, value, layout::ONE_BITS);
    let holds = e.b.ins().bor(is_true, is_one);
    let is_false =
        e.b.ins()
            .icmp_imm_u(IntCC::Equal, value, layout::FALSE_BITS);
    let is_zero = e.b.ins().icmp_imm_u(IntCC::Equal, value, layout::ZERO_BITS);
    let fails = e.b.ins().bor(is_false, is_zero);
    let holds = e.b.ins().band(holds, quick);
    let fails = e.b.ins().band(fails, quick);

    let fall = e.b.create_block();
    let other = e.b.create_block();
    let take = e.b.create_block();
    let slow = e.b.create_block();
    e.b.ins().brif(holds, fall, &[], other, &[]);
    e.b.switch_to_block(other);
    e.b.ins().brif(fails, take, &[], slow, &[]);

    // The clause closes before the jump: free when generated code opened it,
    // the host's close otherwise.
    e.b.switch_to_block(take);
    let opened = e.opened();
    let closing = e.b.create_block();
    let free = e.b.create_block();
    e.b.ins().brif(opened, free, &[], closing, &[]);
    e.b.switch_to_block(free);
    e.clear_opened();
    e.b.ins().jump(target_block, &[]);
    e.b.switch_to_block(closing);
    e.call(close_helper as *const (), Some(target));
    e.clear_opened();
    e.b.ins().jump(target_block, &[]);

    e.b.switch_to_block(slow);
    let branched = e.b.create_block();
    e.call_op(Some(clause_pc), pc, Some((BRANCH, branched)));
    e.b.ins().jump(fall, &[]);
    e.b.switch_to_block(branched);
    e.b.ins().jump(target_block, &[]);

    e.b.switch_to_block(fall);
}

/// `Op::EndBranch`: a clause counted at the line in force, which while the
/// block is steady and the countdown has more than this in it is the count
/// alone; the host's `end_promoted_branch` otherwise. A block's start
/// ([`Step::BlockStart`]) is the same clause, counted inside the clause at
/// `clause_pc`, and its host path is `flat_block_start`'s.
fn end_branch(e: &mut Emit<'_, '_>, clause_pc: Option<u32>, pc: u32) {
    let steady = e.load_native(types::I64, layout::STEADY);
    let countdown = e.load_native(types::I64, layout::COUNTDOWN);
    let left =
        e.b.ins()
            .icmp_imm_u(IntCC::UnsignedGreaterThan, countdown, 1);
    let ready = e.b.ins().icmp_imm_u(IntCC::NotEqual, steady, 0);
    let ready = e.b.ins().band(ready, left);
    let fast = e.b.create_block();
    let slow = e.b.create_block();
    let join = e.b.create_block();
    e.b.ins().brif(ready, fast, &[], slow, &[]);
    e.b.switch_to_block(fast);
    let countdown = e.b.ins().iadd_imm_s(countdown, -1);
    e.store_native(countdown, layout::COUNTDOWN);
    e.b.ins().jump(join, &[]);
    e.b.switch_to_block(slow);
    e.call_op(clause_pc, pc, None);
    e.b.ins().jump(join, &[]);
    e.b.switch_to_block(join);
}
