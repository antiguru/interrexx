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
//!    sets from `RootSet::top_window` after every call into the host and
//!    which nothing but a call into the host can invalidate; a null window is
//!    never dereferenced. It writes a slot only as `VarSlot::of` would.
//! 4. The helpers are the only code that turns the `Ctx` pointer back into a
//!    reference, and only while generated code is suspended in that call.
//! 5. The machine code lives in the [`JITModule`] that [`Compiled`] owns, and
//!    a module is never freed while its `Compiled` lives.

#![allow(unsafe_code)]

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{AbiParam, InstBuilder, MemFlagsData, Signature, Value, types};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module, default_libcall_names};
use interrexx_core::RegFrame;

pub(crate) mod layout;

use layout::Native;

/// A helper's answer that lets the generated code carry on.
pub(crate) const CONTINUE: u32 = 0;
/// Any other answer ends the run; the host has recorded what happened.
pub(crate) const STOP: u32 = 1;

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
}

/// One step of a pass.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Step {
    /// Opens the clause at `pc`. Generated code opens it itself when the
    /// block says the interpreter is steady and the clause is `tabled`, with
    /// the line and indent the chunk's table gives it.
    Open {
        pc: u32,
        index: u32,
        tabled: bool,
        line: u32,
        indent: u32,
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
}

/// The operators [`Step::Arith`] does itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Arith {
    Add,
    Subtract,
    Multiply,
}

/// One loop's passes: the boundary, then `steps`, then the boundary again.
#[derive(Debug)]
pub(crate) struct Pass {
    pub(crate) steps: Vec<Step>,
    /// How many registers the frame the code runs against holds.
    pub(crate) registers: u16,
}

/// What a helper is handed: the block, and the host it belongs to.
#[repr(C)]
struct Ctx<'h> {
    native: Native,
    host: &'h mut dyn Host,
}

/// Runs `step` on the host between [`Host::enter`] and [`Host::leave`].
fn bracket(ctx: *mut Ctx<'_>, step: impl FnOnce(&mut dyn Host) -> u32) -> u32 {
    // SAFETY: property 4 of the module doc.
    let ctx = unsafe { &mut *ctx };
    ctx.host.enter(&mut ctx.native);
    let answer = step(&mut *ctx.host);
    ctx.host.leave(&mut ctx.native);
    answer
}

extern "C" fn pass_helper(ctx: *mut Ctx<'_>) -> u32 {
    bracket(ctx, |host| host.pass())
}

extern "C" fn open_helper(ctx: *mut Ctx<'_>, pc: u32) -> u32 {
    bracket(ctx, |host| host.open(pc))
}

extern "C" fn adopt_helper(ctx: *mut Ctx<'_>, pc: u32) -> u32 {
    bracket(ctx, |host| host.adopt(pc))
}

extern "C" fn close_helper(ctx: *mut Ctx<'_>, next: u32) -> u32 {
    bracket(ctx, |host| host.close(next))
}

extern "C" fn op_helper(ctx: *mut Ctx<'_>, pc: u32) -> u32 {
    bracket(ctx, |host| host.op(pc))
}

type Entry = extern "C" fn(*mut Ctx<'_>) -> u32;

/// A compiled pass.
pub(crate) struct Compiled {
    /// Owns the code `entry` points into (property 5).
    _module: JITModule,
    entry: Entry,
    registers: u16,
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
        let mut ctx = Ctx { native, host };
        let answer = (self.entry)(&mut ctx);
        // Every exit is a helper's answer, so `enter` has already written
        // the block back; this is the same call made idempotent.
        ctx.host.enter(&mut ctx.native);
        answer
    }
}

/// The block's fields generated code holds in variables between calls into
/// the host: `(offset, written back before a call)`. A field that is not
/// written back is one only the host sets.
const FIELDS: [(i32, bool); 18] = [
    (layout::COUNTDOWN, true),
    (layout::CLAUSE_INDEX, true),
    (layout::CLAUSE_LINE, true),
    (layout::CLAUSE_INDENT, true),
    (layout::CLAUSE_DIRTY, true),
    (layout::LOOP_VALUE, true),
    (layout::LOOP_DIRTY, true),
    (layout::STEADY, false),
    (layout::SLOTS, false),
    (layout::SLOT_COUNT, false),
    (layout::SMALL_INT_BOUND, false),
    (layout::INDENT_BASE, false),
    (layout::LOOP_READY, false),
    (layout::LOOP_SLOT, false),
    (layout::LOOP_STEP, false),
    (layout::LOOP_TO, false),
    (layout::LOOP_LINE, false),
    (layout::LOOP_INDENT, false),
];

/// The code generator's own state while it emits one pass.
///
/// **Between two calls into the host, the block's fields and the registers
/// live in variables.** Each call is preceded by writing the fields generated
/// code changes back to the block, and followed by reading every field and
/// register again, since the host may have changed any of them. A register
/// write also goes to the frame at once, so the registers the collector and
/// the host read are always current.
struct Emit<'b> {
    b: FunctionBuilder<'b>,
    pointer: types::Type,
    ctx: Value,
    registers: Value,
    exit: cranelift_codegen::ir::Block,
    bare: cranelift_codegen::ir::SigRef,
    with_pc: cranelift_codegen::ir::SigRef,
    /// One variable per entry of [`FIELDS`].
    fields: Vec<Variable>,
    /// One variable per register.
    regs: Vec<Variable>,
    /// Whether the open clause was opened by generated code and not yet
    /// adopted, which is what makes its close free.
    fast: Variable,
}

impl Emit<'_> {
    fn flags() -> MemFlagsData {
        MemFlagsData::trusted()
    }

    fn field(&self, offset: i32) -> Variable {
        let at = FIELDS
            .iter()
            .position(|&(field, _)| field == offset)
            .expect("every field generated code reads is in FIELDS");
        self.fields[at]
    }

    fn get(&mut self, offset: i32) -> Value {
        let var = self.field(offset);
        self.b.use_var(var)
    }

    fn set(&mut self, offset: i32, value: Value) {
        debug_assert!(
            FIELDS.iter().any(|&(field, back)| field == offset && back),
            "a field generated code sets is one it writes back"
        );
        let var = self.field(offset);
        self.b.def_var(var, value);
    }

    fn set_const(&mut self, offset: i32, value: i64) {
        let value = self.b.ins().iconst(types::I64, value);
        self.set(offset, value);
    }

    fn reg(&mut self, register: u16) -> Value {
        self.b.use_var(self.regs[usize::from(register)])
    }

    fn set_reg(&mut self, register: u16, value: Value) {
        self.b.def_var(self.regs[usize::from(register)], value);
        self.b.ins().store(
            Self::flags(),
            value,
            self.registers,
            i32::from(register) * 8,
        );
    }

    /// Reads every field and register into its variable.
    fn reload(&mut self) {
        for (at, &(offset, _)) in FIELDS.iter().enumerate() {
            let value = self
                .b
                .ins()
                .load(types::I64, Self::flags(), self.ctx, offset);
            self.b.def_var(self.fields[at], value);
        }
        for register in 0..self.regs.len() {
            let value = self.b.ins().load(
                types::I64,
                Self::flags(),
                self.registers,
                register as i32 * 8,
            );
            self.b.def_var(self.regs[register], value);
        }
    }

    /// Writes the fields generated code changes back to the block.
    fn spill(&mut self) {
        for (at, &(offset, back)) in FIELDS.iter().enumerate() {
            if back {
                let value = self.b.use_var(self.fields[at]);
                self.b.ins().store(Self::flags(), value, self.ctx, offset);
            }
        }
    }

    /// Calls `helper`, and leaves for `exit` unless it answers CONTINUE.
    fn call(&mut self, helper: *const (), pc: Option<u32>) {
        self.spill();
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
        self.reload();
    }

    /// The op at `pc` through the host, from a clause generated code may have
    /// opened itself: adopted first, so the interpreter can close it.
    fn slow_op(&mut self, clause_pc: u32, pc: u32) {
        let fast = self.b.use_var(self.fast);
        let adopt = self.b.create_block();
        let run = self.b.create_block();
        self.b.ins().brif(fast, adopt, &[], run, &[]);
        self.b.switch_to_block(adopt);
        self.call(adopt_helper as *const (), Some(clause_pc));
        let no = self.b.ins().iconst(types::I8, 0);
        self.b.def_var(self.fast, no);
        self.b.ins().jump(run, &[]);
        self.b.switch_to_block(run);
        self.call(op_helper as *const (), Some(pc));
    }

    /// Branches to `slow` unless the window holds slot `slot`, answering the
    /// slot's address.
    fn slot_address(&mut self, slot: Value, slow: cranelift_codegen::ir::Block) -> Value {
        let base = self.get(layout::SLOTS);
        let count = self.get(layout::SLOT_COUNT);
        // A null window has a count of zero, so this covers it.
        let held = self.b.ins().icmp(IntCC::UnsignedLessThan, slot, count);
        let on = self.b.create_block();
        self.b.ins().brif(held, on, &[], slow, &[]);
        self.b.switch_to_block(on);
        let offset = self.b.ins().imul_imm_u(slot, layout::SLOT_SIZE);
        self.b.ins().iadd(base, offset)
    }

    /// Whether `v` is strictly inside `(-bound, bound)`.
    fn within(&mut self, v: Value, bound: Value) -> Value {
        let negative = self.b.ins().ineg(bound);
        let below = self.b.ins().icmp(IntCC::SignedLessThan, v, bound);
        let above = self.b.ins().icmp(IntCC::SignedGreaterThan, v, negative);
        self.b.ins().band(below, above)
    }

    /// Whether `handle` carries the small-integer tag.
    fn small(&mut self, handle: Value) -> Value {
        let tag = self.b.ins().band_imm_u(handle, layout::TAG_MASK);
        self.b.ins().icmp_imm_u(IntCC::Equal, tag, layout::TAG_INT)
    }

    fn tagged(&mut self, value: Value) -> Value {
        let shifted = self.b.ins().ishl_imm_u(value, layout::TAG_BITS);
        self.b.ins().bor_imm_u(shifted, layout::TAG_INT)
    }
}

/// Generates the code for `pass`, or `None` where Cranelift cannot target
/// this host or refuses the function.
pub(crate) fn compile(pass: &Pass) -> Option<Compiled> {
    for step in &pass.steps {
        let named = match *step {
            Step::Const { dst, .. } | Step::Load { dst, .. } => Some(dst),
            Step::Store { src, .. } => Some(src),
            Step::Arith { lhs, rhs, dst, .. } => Some(lhs.max(rhs).max(dst)),
            Step::Open { .. } | Step::Close { .. } | Step::Op { .. } => None,
        };
        // Property 2 of the module doc.
        if named.is_some_and(|register| register >= pass.registers) {
            return None;
        }
    }
    let mut flags = settings::builder();
    flags.set("opt_level", "speed").ok()?;
    flags.set("is_pic", "false").ok()?;
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
        let fast = b.declare_var(types::I8);
        let fields = FIELDS.iter().map(|_| b.declare_var(types::I64)).collect();
        let regs = (0..pass.registers)
            .map(|_| b.declare_var(types::I64))
            .collect();

        let entry = b.create_block();
        let head = b.create_block();
        let exit = b.create_block();
        b.append_block_params_for_function_params(entry);
        b.append_block_param(exit, types::I32);

        b.switch_to_block(entry);
        let ctx = b.block_params(entry)[0];
        let registers = b.ins().load(pointer, Emit::flags(), ctx, layout::REGISTERS);
        let no = b.ins().iconst(types::I8, 0);
        b.def_var(fast, no);

        let mut e = Emit {
            b,
            pointer,
            ctx,
            registers,
            exit,
            bare,
            with_pc,
            fields,
            regs,
            fast,
        };
        e.reload();
        e.b.ins().jump(head, &[]);
        e.b.switch_to_block(head);
        pass_step(&mut e);
        let mut clause_pc = 0;
        for step in &pass.steps {
            match *step {
                Step::Open {
                    pc,
                    index,
                    tabled,
                    line,
                    indent,
                    register_top,
                } => {
                    clause_pc = pc;
                    open(&mut e, pc, index, tabled, line, indent, register_top);
                }
                Step::Close { next } => {
                    let fast = e.b.use_var(e.fast);
                    let slow = e.b.create_block();
                    let done = e.b.create_block();
                    e.b.ins().brif(fast, done, &[], slow, &[]);
                    e.b.switch_to_block(slow);
                    e.call(close_helper as *const (), Some(next));
                    e.b.ins().jump(done, &[]);
                    e.b.switch_to_block(done);
                }
                Step::Op { pc } => e.slow_op(clause_pc, pc),
                Step::Const { dst, bits } => {
                    let value = e.b.ins().iconst(types::I64, bits as i64);
                    e.set_reg(dst, value);
                }
                Step::Load { pc, slot, dst } => {
                    let slow = e.b.create_block();
                    let join = e.b.create_block();
                    let slot = e.b.ins().iconst(types::I64, i64::from(slot));
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
                    e.set_reg(dst, value);
                    e.b.ins().jump(join, &[]);
                    e.b.switch_to_block(slow);
                    e.slow_op(clause_pc, pc);
                    e.b.ins().jump(join, &[]);
                    e.b.switch_to_block(join);
                }
                Step::Store { pc, slot, src } => {
                    let slow = e.b.create_block();
                    let join = e.b.create_block();
                    let slot = e.b.ins().iconst(types::I64, i64::from(slot));
                    let address = e.slot_address(slot, slow);
                    let value = e.reg(src);
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
        e.b.ins().jump(head, &[]);

        e.b.switch_to_block(exit);
        let answer = e.b.block_params(exit)[0];
        e.b.ins().return_(&[answer]);
        e.b.seal_all_blocks();
        e.b.finalize(module.target_config());
    }
    module.define_function(id, &mut context).ok()?;
    module.clear_context(&mut context);
    module.finalize_definitions().ok()?;
    let code = module.get_finalized_function(id);
    // SAFETY: `code` is the function just defined with `signature`, which is
    // `Entry`'s: one pointer in, one `u32` out, the platform's C convention.
    let entry = unsafe { std::mem::transmute::<*const u8, Entry>(code) };
    Some(Compiled {
        _module: module,
        entry,
        registers: pass.registers,
    })
}

/// A clause's opening: by generated code when the block is steady, the
/// clause is tabled and the countdown has more than this clause left in it;
/// otherwise by the host.
fn open(
    e: &mut Emit<'_>,
    pc: u32,
    index: u32,
    tabled: bool,
    line: u32,
    indent: u32,
    register_top: u16,
) {
    let slow = e.b.create_block();
    let opened = e.b.create_block();
    if tabled {
        let steady = e.get(layout::STEADY);
        let countdown = e.get(layout::COUNTDOWN);
        let left =
            e.b.ins()
                .icmp_imm_u(IntCC::UnsignedGreaterThan, countdown, 1);
        let ready = e.b.ins().icmp_imm_u(IntCC::NotEqual, steady, 0);
        let ready = e.b.ins().band(ready, left);
        let fast = e.b.create_block();
        e.b.ins().brif(ready, fast, &[], slow, &[]);

        e.b.switch_to_block(fast);
        let countdown = e.b.ins().iadd_imm_s(countdown, -1);
        e.set(layout::COUNTDOWN, countdown);
        e.set_const(layout::CLAUSE_INDEX, i64::from(index));
        e.set_const(layout::CLAUSE_LINE, i64::from(line));
        let base = e.get(layout::INDENT_BASE);
        let indent = e.b.ins().iadd_imm_u(base, i64::from(indent));
        e.set(layout::CLAUSE_INDENT, indent);
        e.set_const(layout::CLAUSE_DIRTY, 1);
        // `RegFrame::clear_from(register_top)`.
        let nil = e.b.ins().iconst(types::I64, layout::NIL_BITS as i64);
        for register in register_top..e.regs.len() as u16 {
            e.set_reg(register, nil);
        }
        let yes = e.b.ins().iconst(types::I8, 1);
        e.b.def_var(e.fast, yes);
        e.b.ins().jump(opened, &[]);
    } else {
        e.b.ins().jump(slow, &[]);
    }
    e.b.switch_to_block(slow);
    e.call(open_helper as *const (), Some(pc));
    let no = e.b.ins().iconst(types::I8, 0);
    e.b.def_var(e.fast, no);
    e.b.ins().jump(opened, &[]);
    e.b.switch_to_block(opened);
}

/// The pass boundary: the loop's quick pass by generated code when the block
/// offers one, the countdown has more than this clause left in it, and the
/// pass carries on; otherwise the host's, which also ends the loop.
fn pass_step(e: &mut Emit<'_>) {
    let slow = e.b.create_block();
    let body = e.b.create_block();
    let check = e.b.create_block();
    let read = e.b.create_block();
    let take = e.b.create_block();

    let ready = e.get(layout::LOOP_READY);
    let countdown = e.get(layout::COUNTDOWN);
    let left =
        e.b.ins()
            .icmp_imm_u(IntCC::UnsignedGreaterThan, countdown, 1);
    let ready = e.b.ins().icmp_imm_u(IntCC::NotEqual, ready, 0);
    let ready = e.b.ins().band(ready, left);
    e.b.ins().brif(ready, check, &[], slow, &[]);

    e.b.switch_to_block(check);
    let slot = e.get(layout::LOOP_SLOT);
    let address = e.slot_address(slot, slow);
    let set =
        e.b.ins()
            .load(types::I64, Emit::flags(), address, layout::SLOT_SET_AT);
    e.b.ins().brif(set, read, &[], slow, &[]);

    e.b.switch_to_block(read);
    let value =
        e.b.ins()
            .load(types::I64, Emit::flags(), address, layout::SLOT_VALUE_AT);
    let mut ok = e.small(value);
    let running = e.b.ins().sshr_imm_u(value, layout::TAG_BITS);
    let step = e.get(layout::LOOP_STEP);
    let (sum, overflow) = e.b.ins().sadd_overflow(running, step);
    let bound = e.get(layout::SMALL_INT_BOUND);
    for v in [running, sum] {
        let inside = e.within(v, bound);
        ok = e.b.ins().band(ok, inside);
    }
    let fine = e.b.ins().icmp_imm_u(IntCC::Equal, overflow, 0);
    ok = e.b.ins().band(ok, fine);
    // `loop_advance_quick`'s own answer: a descending step carries on while
    // the value is at or above the bound, any other while it is at or below.
    let to = e.get(layout::LOOP_TO);
    let descending = e.b.ins().icmp_imm_u(IntCC::SignedLessThan, step, 0);
    let at_or_above = e.b.ins().icmp(IntCC::SignedGreaterThanOrEqual, sum, to);
    let at_or_below = e.b.ins().icmp(IntCC::SignedLessThanOrEqual, sum, to);
    let carries_on = e.b.ins().select(descending, at_or_above, at_or_below);
    ok = e.b.ins().band(ok, carries_on);
    e.b.ins().brif(ok, take, &[], slow, &[]);

    e.b.switch_to_block(take);
    let countdown = e.b.ins().iadd_imm_s(countdown, -1);
    e.set(layout::COUNTDOWN, countdown);
    let one = e.b.ins().iconst(types::I64, 1);
    e.b.ins()
        .store(Emit::flags(), one, address, layout::SLOT_SET_AT);
    let handle = e.tagged(sum);
    e.b.ins()
        .store(Emit::flags(), handle, address, layout::SLOT_VALUE_AT);
    e.set(layout::LOOP_VALUE, sum);
    e.set_const(layout::LOOP_DIRTY, 1);
    let line = e.get(layout::LOOP_LINE);
    e.set(layout::CLAUSE_LINE, line);
    let indent = e.get(layout::LOOP_INDENT);
    e.set(layout::CLAUSE_INDENT, indent);
    e.set_const(layout::CLAUSE_DIRTY, 1);
    e.b.ins().jump(body, &[]);

    e.b.switch_to_block(slow);
    e.call(pass_helper as *const (), None);
    e.b.ins().jump(body, &[]);

    e.b.switch_to_block(body);
}

fn arith(e: &mut Emit<'_>, clause_pc: u32, pc: u32, op: Arith, lhs: u16, rhs: u16, dst: u16) {
    let left = e.reg(lhs);
    let right = e.reg(rhs);
    let fast = e.b.create_block();
    let store = e.b.create_block();
    let slow = e.b.create_block();
    let join = e.b.create_block();
    e.b.append_block_param(store, types::I64);
    let left_small = e.small(left);
    let right_small = e.small(right);
    let both = e.b.ins().band(left_small, right_small);
    e.b.ins().brif(both, fast, &[], slow, &[]);

    e.b.switch_to_block(fast);
    let bound = e.get(layout::SMALL_INT_BOUND);
    let a = e.b.ins().sshr_imm_u(left, layout::TAG_BITS);
    let c = e.b.ins().sshr_imm_u(right, layout::TAG_BITS);
    let (result, overflow) = match op {
        Arith::Add => (e.b.ins().iadd(a, c), None),
        Arith::Subtract => (e.b.ins().isub(a, c), None),
        Arith::Multiply => {
            let (product, overflow) = e.b.ins().smul_overflow(a, c);
            (product, Some(overflow))
        }
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
    let handle = e.tagged(result);
    e.set_reg(dst, handle);
    e.b.ins().jump(join, &[]);

    e.b.switch_to_block(slow);
    e.slow_op(clause_pc, pc);
    e.b.ins().jump(join, &[]);

    e.b.switch_to_block(join);
}
